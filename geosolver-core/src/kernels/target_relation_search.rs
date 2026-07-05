use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::algebra::linear_solve::{
    solve_homogeneous_modular, MatrixBuilder, ModularNullspaceResult, ModularSolvePlan,
};
use crate::algebra::normal_form::{MembershipCertificate, MembershipTerm};
use crate::compose::message::{MessageRepresentation, ProjectionMessage, ProjectionStrength};
use crate::graph::projection_dag::ProjectionBlock;
use crate::kernels::traits::{
    declined_kernel_admission, KernelContext, KernelKind, ReplayResult, TargetProjectionKernel,
};
use crate::planner::admission::{KernelAdmission, KernelAdmissionStatus};
use crate::planner::kernel_plan::{
    planned_failure_behavior, rank_plan, resource_bounds_hash, support_plan_hash, template_plan,
    CertificateRoute, KernelExecutionPlan, KernelSupportPlan, LocalNonfinitePolicy, ResourceBounds,
};
use crate::problem::canonicalize::CanonicalRelationQ;
use crate::problem::context::SolverContext;
use crate::result::cost_trace::ProjectionCostTrace;
use crate::result::status::{AlgebraicReason, FailureKind, SolverError, SolverErrorKind, StageId};
use crate::solver::options::SolverOptions;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{KernelPlanId, PackageId, VariableId};
use crate::types::matrix::{hash_matrix, matrix_density, SparseMatrixQ, VectorQ};
use crate::types::monomial::{
    monomial_degree, monomial_mul, monomial_to_bytes, normalize_monomial, Monomial,
};
use crate::types::polynomial::{
    clear_denominators_primitive, poly_add, poly_monomial_count, poly_mul, poly_scale, poly_sub,
    poly_total_degree, poly_variables, zero_poly, SparsePolynomialQ, TermQ,
};
use crate::types::rational::{add_q, div_q, int_q, is_zero_q, neg_q, RationalQ};
use crate::verify::certificates::{
    KernelCertificate, KernelCertificatePayload, MembershipProjectionCertificate,
};

pub struct TargetRelationSearchKernel;

impl TargetProjectionKernel for TargetRelationSearchKernel {
    fn kind(&self) -> KernelKind {
        KernelKind::TargetRelationSearch
    }

    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission {
        admit_target_relation_search(block, ctx, &SolverContext::new(Default::default()))
    }

    fn plan(
        &self,
        admission: &KernelAdmission,
        _ctx: &KernelContext,
        _solver_ctx: &SolverContext,
    ) -> Result<KernelExecutionPlan, SolverError> {
        admission
            .execution_plan
            .clone()
            .ok_or_else(|| implementation_bug("target-relation-search admission has no plan"))
    }

    fn execute(
        &self,
        plan: &KernelExecutionPlan,
        ctx: &mut KernelContext,
        solver_ctx: &mut SolverContext,
    ) -> Result<ProjectionMessage, SolverError> {
        execute_target_relation_search(plan, ctx, solver_ctx)
    }

    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult {
        crate::kernels::traits::exact_replay_result(
            self.kind(),
            "target-relation-search-replay",
            message,
            ctx,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationSearchBound {
    pub export_degree: usize,
    pub multiplier_total_degree: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationSearchStage {
    pub export_degree: usize,
    pub multiplier_total_degree: usize,
    pub export_support_hash: Hash,
    pub multiplier_support_hashes: Vec<Hash>,
    pub row_monomial_hash: Hash,
    pub row_monomial_count: usize,
    pub matrix_rows: usize,
    pub matrix_cols: usize,
    pub stage_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DenseRelationSearchSchedule {
    pub eliminated_variables: Vec<VariableId>,
    pub exported_variables: Vec<VariableId>,
    pub z_seed: usize,
    pub e_cap: usize,
    pub d_max: usize,
    pub stages: Vec<RelationSearchStage>,
    pub schedule_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipMatrixTrace {
    pub stage_hash: Hash,
    pub matrix_hash: Hash,
    pub row_monomial_hash: Hash,
    pub export_support_hash: Hash,
    pub multiplier_support_hashes: Vec<Hash>,
    pub rows: usize,
    pub cols: usize,
    pub trace_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipMatrixBuilder {
    pub bound: RelationSearchBound,
    pub exported_variables: Vec<VariableId>,
    pub eliminated_variables: Vec<VariableId>,
    pub export_support: Vec<Monomial>,
    pub multiplier_supports: Vec<Vec<Monomial>>,
    pub row_monomials: Vec<Monomial>,
    pub matrix: SparseMatrixQ,
    pub builder_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedRelationSearchCandidate {
    pub relation: SparsePolynomialQ,
    pub multipliers: Vec<SparsePolynomialQ>,
    pub exact_identity_hash: Hash,
    pub candidate_hash: Hash,
}

#[derive(Debug, Clone)]
struct RelationSearchCandidate {
    relation: SparsePolynomialQ,
    multipliers: Vec<SparsePolynomialQ>,
}

pub fn admit_target_relation_search(
    block: &ProjectionBlock,
    ctx: &KernelContext,
    solver_ctx: &SolverContext,
) -> KernelAdmission {
    let relations = block_relations(block, &ctx.system);
    if relations.is_empty() {
        return declined_kernel_admission(
            KernelKind::TargetRelationSearch,
            block,
            "no authorized local relations for dense relation search",
        );
    }
    let relation_polys = relations
        .iter()
        .map(|relation| relation.polynomial.clone())
        .collect::<Vec<_>>();
    let eliminated_variables = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    let exported_variables =
        sorted_variables(&block.exported_variables.iter().copied().collect::<Vec<_>>());
    let schedule = build_dense_relation_search_schedule(
        &relation_polys,
        &eliminated_variables,
        &exported_variables,
        &solver_ctx.options,
    );
    let first_stage = schedule.stages.first();
    let mut support_plan = KernelSupportPlan {
        dense_relation_search_schedule: Some(schedule.clone()),
        affine_elimination_order: None,
        template_plan: Some(template_plan(
            first_stage.map_or(0, |stage| stage.matrix_rows).max(1),
            first_stage.map_or(0, |stage| stage.matrix_cols).max(1),
            first_stage.map_or_else(
                || hash_sequence("target-relation-empty-row", &[]),
                |stage| stage.row_monomial_hash,
            ),
            first_stage.map_or_else(
                || hash_sequence("target-relation-empty-col", &[]),
                |stage| stage.export_support_hash,
            ),
        )),
        rank_plan: Some(rank_plan(first_stage.map_or(1, |stage| stage.matrix_cols))),
        universal_strategy_sequence: Vec::new(),
        degree_bound: schedule.e_cap,
        support_hash: hash_sequence("kernel-support-plan", &[]),
    };
    support_plan.support_hash = support_plan_hash(&support_plan);
    let mut resource_bounds = ResourceBounds {
        max_matrix_rows: first_stage.map(|stage| stage.matrix_rows),
        max_matrix_cols: first_stage.map(|stage| stage.matrix_cols),
        max_export_degree: Some(schedule.e_cap),
        max_multiplier_total_degree: schedule
            .stages
            .last()
            .map(|stage| stage.multiplier_total_degree),
        max_local_elimination_steps: Some(0),
        max_memory_bytes: solver_ctx.options.max_memory_bytes,
        bounds_hash: hash_sequence("planner-resource-bounds", &[]),
    };
    resource_bounds.bounds_hash = resource_bounds_hash(&resource_bounds);
    let plan = KernelExecutionPlan::new(
        KernelPlanId(KernelKind::TargetRelationSearch as u32),
        block.block_id,
        KernelKind::TargetRelationSearch,
        block.authorization_hash,
        relations.iter().map(|relation| relation.id).collect(),
        relations.iter().map(|relation| relation.hash).collect(),
        block.child_block_ids.clone(),
        Vec::new(),
        exported_variables,
        eliminated_variables,
        support_plan,
        resource_bounds,
        CertificateRoute::DenseRelationSearchMembership,
        planned_failure_behavior(
            vec![
                crate::result::status::SolverStatus::AlgorithmicHardCase,
                crate::result::status::SolverStatus::FiniteResourceFailure,
                crate::result::status::SolverStatus::CertificateDesignGap,
            ],
            LocalNonfinitePolicy::NotApplicable,
        ),
    );
    finish_admission(block, KernelAdmissionStatus::Admitted, Some(plan))
}

pub fn execute_target_relation_search(
    plan: &KernelExecutionPlan,
    ctx: &mut KernelContext,
    solver_ctx: &mut SolverContext,
) -> Result<ProjectionMessage, SolverError> {
    validate_relation_search_plan_binding(plan, ctx)?;
    let relations = planned_relations(plan, ctx)?;
    if relations.is_empty() {
        return Err(algorithmic_hard_case(
            ctx,
            "target relation search has no authorized local relations",
        ));
    }
    let relation_polys = relations
        .iter()
        .map(|relation| relation.polynomial.clone())
        .collect::<Vec<_>>();
    let Some(schedule) = &plan.support_plan.dense_relation_search_schedule else {
        return Err(implementation_bug(
            "target relation search plan lacks dense schedule",
        ));
    };
    let recomputed = build_dense_relation_search_schedule(
        &relation_polys,
        &plan.eliminated_variables,
        &plan.exported_variables,
        &solver_ctx.options,
    );
    if &recomputed != schedule
        || hash_dense_relation_search_schedule(schedule) != schedule.schedule_hash
    {
        return Err(implementation_bug(
            "target relation search schedule is not reproducible from J,Y,Z,options",
        ));
    }
    let exported = plan
        .exported_variables
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut traces = Vec::new();
    for stage in &schedule.stages {
        let bound = RelationSearchBound {
            export_degree: stage.export_degree,
            multiplier_total_degree: stage.multiplier_total_degree,
        };
        let export_support = build_export_monomial_support(&plan.exported_variables, &bound);
        let multiplier_supports = build_multiplier_supports(
            &relation_polys,
            &plan.eliminated_variables,
            &plan.exported_variables,
            &bound,
        );
        let row_monomials =
            build_row_monomial_support(&relation_polys, &export_support, &multiplier_supports);
        verify_stage_recomputes(stage, &export_support, &multiplier_supports, &row_monomials)?;
        let matrix_builder = build_membership_matrix_builder_with_supports(
            &relation_polys,
            &plan.eliminated_variables,
            &plan.exported_variables,
            &bound,
            export_support,
            multiplier_supports,
            row_monomials,
        );
        let matrix = matrix_builder.matrix.clone();
        enforce_matrix_limits(plan, solver_ctx, &matrix)?;
        let trace = membership_matrix_trace(stage, &matrix);
        let nullspace = solve_homogeneous_modular(
            MatrixBuilder {
                matrix: matrix.clone(),
            },
            ModularSolvePlan {
                seed: 101,
                max_primes: 4,
                stable_rank_after: 2,
                reconstruction_height_bound: solver_ctx.options.max_coefficient_height_bits,
            },
        );
        for verified in reconstruct_and_verify_relation_from_builder(
            &nullspace,
            &matrix_builder,
            &relation_polys,
            &exported,
        )? {
            let relation = verified.relation;
            let multipliers = verified.multipliers;
            let certificate_hash = target_relation_search_certificate_hash(
                plan,
                stage,
                &trace,
                &nullspace
                    .traces
                    .iter()
                    .map(|trace| trace.prime)
                    .collect::<Vec<_>>(),
                &relation,
                &multipliers,
            );
            let cost_trace = ProjectionCostTrace {
                block_id: plan.block_id,
                kernel_kind: KernelKind::TargetRelationSearch,
                local_variable_count: ctx.block.local_variables.len(),
                exported_variable_count: plan.exported_variables.len(),
                local_relation_count: relations.len(),
                local_monomial_count: relation_polys.iter().map(poly_monomial_count).sum(),
                estimated_quotient_rank: Some(nullspace.rank),
                matrix_rows: Some(matrix.rows),
                matrix_cols: Some(matrix.cols),
                matrix_density: Some(matrix_density(&matrix)),
                coefficient_height_before_bits: 0,
                coefficient_height_after_bits: poly_monomial_count(&relation),
            };
            let membership = MembershipCertificate {
                combination_terms: multipliers
                    .iter()
                    .enumerate()
                    .filter(|(_, multiplier)| !multiplier.terms.is_empty())
                    .map(|(relation_id, multiplier)| MembershipTerm {
                        relation_id,
                        multiplier: multiplier.clone(),
                    })
                    .collect(),
            };
            let certificate = KernelCertificate::from_execution_plan_with_payload(
                plan,
                std::slice::from_ref(&relation),
                certificate_hash,
                KernelCertificatePayload::Membership(MembershipProjectionCertificate {
                    source_relations: relation_polys.clone(),
                    output_memberships: vec![membership],
                }),
            );
            let mut message = ProjectionMessage {
                package_id: PackageId(plan.plan_id.0),
                block_id: plan.block_id,
                kernel_kind: KernelKind::TargetRelationSearch,
                source_relation_ids: plan.source_relation_ids.clone(),
                eliminated_variables: plan.eliminated_variables.clone(),
                exported_variables: plan.exported_variables.clone(),
                relation_generators: vec![relation],
                representation: MessageRepresentation::GeneratorSet,
                projection_strength: ProjectionStrength::CandidateCoverStrong,
                certificate,
                compression_trace: ctx.system.compression_trace.clone(),
                cost_trace,
                package_hash: hash_sequence("projection-message-initial", &[]),
            };
            message.package_hash = projection_message_hash(&message);
            return Ok(message);
        }
        traces.push(trace);
    }
    Err(algorithmic_hard_case_with_traces(
        ctx,
        "no target/separator relation found within declared dense schedule",
        &traces,
    ))
}

pub fn relation_search_default_export_degree_cap(
    j: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
) -> usize {
    let d_max = j
        .iter()
        .map(|relation| poly_total_degree(relation) as usize)
        .max()
        .unwrap_or(0);
    let z_seed = relation_search_z_seed(j, exported);
    z_seed.max(
        2_usize
            .saturating_mul(d_max)
            .saturating_add(eliminated.len())
            .saturating_add(exported.len()),
    )
}

pub fn build_dense_relation_search_schedule(
    j: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    options: &SolverOptions,
) -> DenseRelationSearchSchedule {
    let eliminated_variables = sorted_variables(eliminated);
    let exported_variables = sorted_variables(exported);
    let z_seed = relation_search_z_seed(j, &exported_variables);
    let d_max = j
        .iter()
        .map(|relation| poly_total_degree(relation) as usize)
        .max()
        .unwrap_or(0);
    let default_cap =
        relation_search_default_export_degree_cap(j, &eliminated_variables, &exported_variables);
    let e_cap = options
        .max_relation_search_export_degree
        .unwrap_or(default_cap);
    let mut stages = Vec::new();
    for e in z_seed..=e_cap {
        let bound = RelationSearchBound {
            export_degree: e,
            multiplier_total_degree: e.saturating_add(d_max),
        };
        let export_support = build_export_monomial_support(&exported_variables, &bound);
        let multiplier_supports =
            build_multiplier_supports(j, &eliminated_variables, &exported_variables, &bound);
        let row_monomials = build_row_monomial_support(j, &export_support, &multiplier_supports);
        let export_support_hash = hash_monomials("rgq042-export-support", &export_support);
        let multiplier_support_hashes = multiplier_supports
            .iter()
            .map(|support| hash_monomials("rgq042-multiplier-support", support))
            .collect::<Vec<_>>();
        let row_monomial_hash = hash_monomials("rgq042-row-monomials", &row_monomials);
        let matrix_rows = row_monomials.len();
        let matrix_cols = export_support.len()
            + multiplier_supports
                .iter()
                .map(|support| support.len())
                .sum::<usize>();
        let mut stage = RelationSearchStage {
            export_degree: e,
            multiplier_total_degree: bound.multiplier_total_degree,
            export_support_hash,
            multiplier_support_hashes,
            row_monomial_hash,
            row_monomial_count: matrix_rows,
            matrix_rows,
            matrix_cols,
            stage_hash: hash_sequence("rgq042-relation-search-stage", &[]),
        };
        stage.stage_hash = hash_relation_search_stage(&stage);
        stages.push(stage);
    }
    let mut schedule = DenseRelationSearchSchedule {
        eliminated_variables,
        exported_variables,
        z_seed,
        e_cap,
        d_max,
        stages,
        schedule_hash: hash_sequence("rgq042-dense-relation-search-schedule", &[]),
    };
    schedule.schedule_hash = hash_dense_relation_search_schedule(&schedule);
    schedule
}

pub fn hash_dense_relation_search_schedule(schedule: &DenseRelationSearchSchedule) -> Hash {
    let mut chunks = vec![
        schedule.z_seed.to_be_bytes().to_vec(),
        schedule.e_cap.to_be_bytes().to_vec(),
        schedule.d_max.to_be_bytes().to_vec(),
    ];
    for variable in &schedule.eliminated_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for variable in &schedule.exported_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for stage in &schedule.stages {
        chunks.push(stage.stage_hash.0.to_vec());
        chunks.push(hash_relation_search_stage(stage).0.to_vec());
    }
    hash_sequence("rgq042-dense-relation-search-schedule", &chunks)
}

pub fn hash_relation_search_stage(stage: &RelationSearchStage) -> Hash {
    let mut chunks = vec![
        stage.export_degree.to_be_bytes().to_vec(),
        stage.multiplier_total_degree.to_be_bytes().to_vec(),
        stage.export_support_hash.0.to_vec(),
    ];
    for hash in &stage.multiplier_support_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    chunks.push(stage.row_monomial_hash.0.to_vec());
    chunks.push(stage.row_monomial_count.to_be_bytes().to_vec());
    chunks.push(stage.matrix_rows.to_be_bytes().to_vec());
    chunks.push(stage.matrix_cols.to_be_bytes().to_vec());
    hash_sequence("rgq042-relation-search-stage", &chunks)
}

pub fn build_export_monomial_support(
    exported: &[VariableId],
    bound: &RelationSearchBound,
) -> Vec<Monomial> {
    monomials_total_degree_leq(&sorted_variables(exported), bound.export_degree)
}

pub fn build_multiplier_supports(
    relations: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    bound: &RelationSearchBound,
) -> Vec<Vec<Monomial>> {
    let variables = sorted_union(eliminated, exported);
    relations
        .iter()
        .map(|relation| {
            let relation_degree = poly_total_degree(relation) as usize;
            let multiplier_degree = bound
                .multiplier_total_degree
                .saturating_sub(relation_degree);
            monomials_total_degree_leq(&variables, multiplier_degree)
        })
        .collect()
}

fn relation_search_z_seed(j: &[SparsePolynomialQ], exported: &[VariableId]) -> usize {
    let exported_set = exported.iter().copied().collect::<BTreeSet<_>>();
    j.iter()
        .flat_map(|relation| relation.terms.iter())
        .map(|term| {
            term.monomial
                .exponents
                .iter()
                .filter(|(var, _)| exported_set.contains(var))
                .map(|(_, exp)| *exp as usize)
                .sum::<usize>()
        })
        .max()
        .unwrap_or(0)
        .max(1)
}

fn build_row_monomial_support(
    relations: &[SparsePolynomialQ],
    export_support: &[Monomial],
    multiplier_supports: &[Vec<Monomial>],
) -> Vec<Monomial> {
    let mut row_monomials = export_support.iter().cloned().collect::<BTreeSet<_>>();
    for (relation, support) in relations.iter().zip(multiplier_supports.iter()) {
        for multiplier in support {
            for term in &relation.terms {
                row_monomials.insert(monomial_mul(multiplier, &term.monomial));
            }
        }
    }
    row_monomials.into_iter().collect()
}

fn monomials_total_degree_leq(variables: &[VariableId], max_degree: usize) -> Vec<Monomial> {
    let mut out = Vec::new();
    let mut current = Vec::new();
    enumerate_monomials(variables, 0, max_degree as u32, &mut current, &mut out);
    out.sort_by(|a, b| (monomial_degree(a), a).cmp(&(monomial_degree(b), b)));
    out
}

fn enumerate_monomials(
    variables: &[VariableId],
    index: usize,
    remaining: u32,
    current: &mut Vec<(VariableId, u32)>,
    out: &mut Vec<Monomial>,
) {
    if index == variables.len() {
        out.push(normalize_monomial(current.clone()));
        return;
    }
    let variable = variables[index];
    for exponent in 0..=remaining {
        if exponent > 0 {
            current.push((variable, exponent));
        }
        enumerate_monomials(variables, index + 1, remaining - exponent, current, out);
        if exponent > 0 {
            current.pop();
        }
    }
}

fn sorted_variables(vars: &[VariableId]) -> Vec<VariableId> {
    vars.iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sorted_union(a: &[VariableId], b: &[VariableId]) -> Vec<VariableId> {
    a.iter()
        .chain(b.iter())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn hash_monomials(tag: &str, monomials: &[Monomial]) -> Hash {
    hash_sequence(
        tag,
        &monomials.iter().map(monomial_to_bytes).collect::<Vec<_>>(),
    )
}

pub fn build_membership_matrix(
    relations: &[SparsePolynomialQ],
    export_support: &[Monomial],
    multiplier_supports: &[Vec<Monomial>],
    row_monomials: &[Monomial],
) -> SparseMatrixQ {
    let row_index = row_monomials
        .iter()
        .cloned()
        .enumerate()
        .map(|(idx, monomial)| (monomial, idx))
        .collect::<BTreeMap<_, _>>();
    let mut entries = BTreeMap::<(usize, usize), RationalQ>::new();
    for (col, monomial) in export_support.iter().enumerate() {
        if let Some(row) = row_index.get(monomial) {
            add_matrix_entry(&mut entries, *row, col, int_q(1));
        }
    }
    let mut col_offset = export_support.len();
    for (relation, support) in relations.iter().zip(multiplier_supports.iter()) {
        for (support_idx, multiplier) in support.iter().enumerate() {
            let col = col_offset + support_idx;
            for term in &relation.terms {
                let row_monomial = monomial_mul(multiplier, &term.monomial);
                if let Some(row) = row_index.get(&row_monomial) {
                    add_matrix_entry(&mut entries, *row, col, neg_q(&term.coeff));
                }
            }
        }
        col_offset += support.len();
    }
    SparseMatrixQ {
        rows: row_monomials.len(),
        cols: export_support.len()
            + multiplier_supports
                .iter()
                .map(|support| support.len())
                .sum::<usize>(),
        entries: entries
            .into_iter()
            .filter(|(_, coeff)| !is_zero_q(coeff))
            .map(|((row, col), coeff)| (row, col, coeff))
            .collect(),
    }
}

pub fn build_membership_matrix_builder(
    j: &[SparsePolynomialQ],
    bound: &RelationSearchBound,
) -> MembershipMatrixBuilder {
    let exported = j.iter().flat_map(poly_variables).collect::<Vec<_>>();
    build_membership_matrix_builder_for_variables(j, &[], &exported, bound)
}

pub fn build_membership_matrix_builder_for_variables(
    j: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    bound: &RelationSearchBound,
) -> MembershipMatrixBuilder {
    let export_support = build_export_monomial_support(exported, bound);
    let multiplier_supports = build_multiplier_supports(j, eliminated, exported, bound);
    let row_monomials = build_row_monomial_support(j, &export_support, &multiplier_supports);
    build_membership_matrix_builder_with_supports(
        j,
        eliminated,
        exported,
        bound,
        export_support,
        multiplier_supports,
        row_monomials,
    )
}

fn build_membership_matrix_builder_with_supports(
    j: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    bound: &RelationSearchBound,
    export_support: Vec<Monomial>,
    multiplier_supports: Vec<Vec<Monomial>>,
    row_monomials: Vec<Monomial>,
) -> MembershipMatrixBuilder {
    let matrix = build_membership_matrix(j, &export_support, &multiplier_supports, &row_monomials);
    let mut builder = MembershipMatrixBuilder {
        bound: *bound,
        exported_variables: sorted_variables(exported),
        eliminated_variables: sorted_variables(eliminated),
        export_support,
        multiplier_supports,
        row_monomials,
        matrix,
        builder_hash: hash_sequence("target-relation-membership-builder", &[]),
    };
    builder.builder_hash = membership_matrix_builder_hash(&builder);
    builder
}

fn membership_matrix_builder_hash(builder: &MembershipMatrixBuilder) -> Hash {
    let mut chunks = vec![
        builder.bound.export_degree.to_be_bytes().to_vec(),
        builder.bound.multiplier_total_degree.to_be_bytes().to_vec(),
        hash_monomials("rgq042-export-support", &builder.export_support)
            .0
            .to_vec(),
        hash_monomials("rgq042-row-monomials", &builder.row_monomials)
            .0
            .to_vec(),
        hash_matrix(&builder.matrix).0.to_vec(),
    ];
    for support in &builder.multiplier_supports {
        chunks.push(
            hash_monomials("rgq042-multiplier-support", support)
                .0
                .to_vec(),
        );
    }
    hash_sequence("target-relation-membership-builder", &chunks)
}

fn add_matrix_entry(
    entries: &mut BTreeMap<(usize, usize), RationalQ>,
    row: usize,
    col: usize,
    value: RationalQ,
) {
    let next = entries
        .remove(&(row, col))
        .map_or(value.clone(), |old| add_q(&old, &value));
    if !is_zero_q(&next) {
        entries.insert((row, col), next);
    }
}

fn verify_stage_recomputes(
    stage: &RelationSearchStage,
    export_support: &[Monomial],
    multiplier_supports: &[Vec<Monomial>],
    row_monomials: &[Monomial],
) -> Result<(), SolverError> {
    let export_hash = hash_monomials("rgq042-export-support", export_support);
    let multiplier_hashes = multiplier_supports
        .iter()
        .map(|support| hash_monomials("rgq042-multiplier-support", support))
        .collect::<Vec<_>>();
    let row_hash = hash_monomials("rgq042-row-monomials", row_monomials);
    let matrix_cols = export_support.len()
        + multiplier_supports
            .iter()
            .map(|support| support.len())
            .sum::<usize>();
    if stage.export_support_hash != export_hash
        || stage.multiplier_support_hashes != multiplier_hashes
        || stage.row_monomial_hash != row_hash
        || stage.row_monomial_count != row_monomials.len()
        || stage.matrix_rows != row_monomials.len()
        || stage.matrix_cols != matrix_cols
        || hash_relation_search_stage(stage) != stage.stage_hash
    {
        return Err(implementation_bug(
            "target relation search stage is not reproducible from J,Y,Z,bound",
        ));
    }
    Ok(())
}

fn relation_search_candidate_from_vector(
    vector: &VectorQ,
    export_support: &[Monomial],
    multiplier_supports: &[Vec<Monomial>],
) -> Option<RelationSearchCandidate> {
    if vector.entries.len()
        != export_support.len()
            + multiplier_supports
                .iter()
                .map(|support| support.len())
                .sum::<usize>()
    {
        return None;
    }
    let relation =
        polynomial_from_coefficients(export_support, &vector.entries[..export_support.len()]);
    if relation.terms.is_empty() {
        return None;
    }
    let mut offset = export_support.len();
    let mut multipliers = Vec::new();
    for support in multiplier_supports {
        let end = offset + support.len();
        multipliers.push(polynomial_from_coefficients(
            support,
            &vector.entries[offset..end],
        ));
        offset = end;
    }
    Some(RelationSearchCandidate {
        relation,
        multipliers,
    })
}

pub fn reconstruct_and_verify_relation(
    ns: &ModularNullspaceResult,
    bound: &RelationSearchBound,
    j: &[SparsePolynomialQ],
) -> Result<Vec<VerifiedRelationSearchCandidate>, SolverError> {
    let builder = build_membership_matrix_builder(j, bound);
    let exported = builder
        .exported_variables
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    reconstruct_and_verify_relation_from_builder(ns, &builder, j, &exported)
}

fn reconstruct_and_verify_relation_from_builder(
    ns: &ModularNullspaceResult,
    builder: &MembershipMatrixBuilder,
    j: &[SparsePolynomialQ],
    exported: &BTreeSet<VariableId>,
) -> Result<Vec<VerifiedRelationSearchCandidate>, SolverError> {
    if membership_matrix_builder_hash(builder) != builder.builder_hash {
        return Err(implementation_bug(
            "target relation search membership matrix builder hash mismatch",
        ));
    }
    let mut verified = Vec::new();
    for vector in deterministic_candidate_vectors(&ns.reconstructed_basis_candidates) {
        let Some(candidate) = relation_search_candidate_from_vector(
            &vector,
            &builder.export_support,
            &builder.multiplier_supports,
        ) else {
            continue;
        };
        let Some((relation, multipliers)) =
            primitive_relation_with_scaled_multipliers(&candidate.relation, &candidate.multipliers)
        else {
            continue;
        };
        if relation.terms.is_empty() || !poly_variables(&relation).is_subset(exported) {
            continue;
        }
        if !verify_membership_exact(&relation, &multipliers, j) {
            continue;
        }
        let exact_identity_hash = hash_sequence(
            "target-relation-search-exact-q-identity",
            &[
                relation.hash.0.to_vec(),
                builder.builder_hash.0.to_vec(),
                hash_sequence(
                    "target-relation-search-multiplier-hashes",
                    &multipliers
                        .iter()
                        .map(|multiplier| multiplier.hash.0.to_vec())
                        .collect::<Vec<_>>(),
                )
                .0
                .to_vec(),
            ],
        );
        let candidate_hash = hash_sequence(
            "target-relation-search-verified-candidate",
            &[relation.hash.0.to_vec(), exact_identity_hash.0.to_vec()],
        );
        verified.push(VerifiedRelationSearchCandidate {
            relation,
            multipliers,
            exact_identity_hash,
            candidate_hash,
        });
    }
    verified.sort_by(|a, b| a.candidate_hash.cmp(&b.candidate_hash));
    Ok(verified)
}

fn deterministic_candidate_vectors(vectors: &[VectorQ]) -> Vec<VectorQ> {
    let mut out = vectors.to_vec();
    out.sort_by(|a, b| rational_vector_bytes(a).cmp(&rational_vector_bytes(b)));
    out
}

fn rational_vector_bytes(vector: &VectorQ) -> Vec<u8> {
    vector
        .entries
        .iter()
        .flat_map(crate::types::rational::rational_to_bytes)
        .collect()
}

fn polynomial_from_coefficients(monomials: &[Monomial], coeffs: &[RationalQ]) -> SparsePolynomialQ {
    crate::types::polynomial::normalize_poly(SparsePolynomialQ {
        terms: monomials
            .iter()
            .zip(coeffs.iter())
            .filter(|(_, coeff)| !is_zero_q(coeff))
            .map(|(monomial, coeff)| TermQ {
                coeff: coeff.clone(),
                monomial: monomial.clone(),
            })
            .collect(),
        hash: hash_sequence("poly", &[]),
    })
}

fn primitive_relation_with_scaled_multipliers(
    relation: &SparsePolynomialQ,
    multipliers: &[SparsePolynomialQ],
) -> Option<(SparsePolynomialQ, Vec<SparsePolynomialQ>)> {
    let primitive = clear_denominators_primitive(relation);
    if primitive.terms.is_empty() {
        return None;
    }
    let source_term = relation.terms.iter().find(|term| !is_zero_q(&term.coeff))?;
    let primitive_term = primitive
        .terms
        .iter()
        .find(|term| term.monomial == source_term.monomial)?;
    let scale = div_q(&primitive_term.coeff, &source_term.coeff).ok()?;
    let scaled_multipliers = multipliers
        .iter()
        .map(|multiplier| poly_scale(multiplier, &scale))
        .collect::<Vec<_>>();
    Some((primitive, scaled_multipliers))
}

pub fn verify_membership_exact(
    relation: &SparsePolynomialQ,
    multipliers: &[SparsePolynomialQ],
    source_relations: &[SparsePolynomialQ],
) -> bool {
    if multipliers.len() != source_relations.len() {
        return false;
    }
    let mut sum = zero_poly();
    for (multiplier, source) in multipliers.iter().zip(source_relations.iter()) {
        sum = poly_add(&sum, &poly_mul(multiplier, source));
    }
    poly_sub(relation, &sum).terms.is_empty()
}

fn membership_matrix_trace(
    stage: &RelationSearchStage,
    matrix: &SparseMatrixQ,
) -> MembershipMatrixTrace {
    let matrix_hash = hash_matrix(matrix);
    let mut trace = MembershipMatrixTrace {
        stage_hash: stage.stage_hash,
        matrix_hash,
        row_monomial_hash: stage.row_monomial_hash,
        export_support_hash: stage.export_support_hash,
        multiplier_support_hashes: stage.multiplier_support_hashes.clone(),
        rows: matrix.rows,
        cols: matrix.cols,
        trace_hash: hash_sequence("target-relation-search-matrix-trace", &[]),
    };
    trace.trace_hash = hash_sequence(
        "target-relation-search-matrix-trace",
        &[
            trace.stage_hash.0.to_vec(),
            trace.matrix_hash.0.to_vec(),
            trace.row_monomial_hash.0.to_vec(),
            trace.export_support_hash.0.to_vec(),
            trace.rows.to_be_bytes().to_vec(),
            trace.cols.to_be_bytes().to_vec(),
        ],
    );
    trace
}

fn target_relation_search_certificate_hash(
    plan: &KernelExecutionPlan,
    stage: &RelationSearchStage,
    trace: &MembershipMatrixTrace,
    primes: &[crate::algebra::modular::Prime],
    relation: &SparsePolynomialQ,
    multipliers: &[SparsePolynomialQ],
) -> Hash {
    let mut chunks = vec![
        plan.plan_hash.0.to_vec(),
        stage.stage_hash.0.to_vec(),
        trace.trace_hash.0.to_vec(),
        relation.hash.0.to_vec(),
    ];
    for prime in primes {
        chunks.push(prime.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for multiplier in multipliers {
        chunks.push(multiplier.hash.0.to_vec());
    }
    let exact_identity_hash = hash_sequence(
        "target-relation-search-exact-identity",
        &[relation.hash.0.to_vec(), trace.matrix_hash.0.to_vec()],
    );
    chunks.push(exact_identity_hash.0.to_vec());
    hash_sequence("target-relation-search-certificate", &chunks)
}

fn validate_relation_search_plan_binding(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    if plan.kernel_kind != KernelKind::TargetRelationSearch {
        return Err(implementation_bug(
            "target relation search execute received wrong plan kind",
        ));
    }
    if crate::planner::kernel_plan::hash_kernel_execution_plan(plan) != plan.plan_hash {
        return Err(implementation_bug(
            "target relation search execution plan hash mismatch",
        ));
    }
    if plan.block_id != ctx.block.block_id {
        return Err(implementation_bug(
            "target relation search block id mismatch",
        ));
    }
    if plan.input_block_authorization_hash != ctx.block.authorization_hash {
        return Err(implementation_bug(
            "target relation search block authorization hash mismatch",
        ));
    }
    Ok(())
}

fn planned_relations(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<Vec<CanonicalRelationQ>, SolverError> {
    if plan.source_relation_ids.len() != plan.source_relation_hashes.len() {
        return Err(implementation_bug(
            "target relation search source relation identity arity mismatch",
        ));
    }
    let mut out = Vec::new();
    for (id, expected_hash) in plan
        .source_relation_ids
        .iter()
        .zip(plan.source_relation_hashes.iter())
    {
        let Some(relation) = ctx
            .system
            .relations
            .iter()
            .find(|relation| relation.id == *id)
        else {
            return Err(implementation_bug(
                "target relation search planned source relation missing",
            ));
        };
        if relation.hash != *expected_hash {
            return Err(implementation_bug(
                "target relation search source relation hash mismatch",
            ));
        }
        out.push(relation.clone());
    }
    Ok(out)
}

fn block_relations(
    block: &ProjectionBlock,
    system: &crate::preprocess::compression::CompressedSystemQ,
) -> Vec<CanonicalRelationQ> {
    let ids = block.relation_ids.iter().copied().collect::<BTreeSet<_>>();
    system
        .relations
        .iter()
        .filter(|relation| ids.contains(&relation.id))
        .cloned()
        .collect()
}

fn enforce_matrix_limits(
    plan: &KernelExecutionPlan,
    ctx: &SolverContext,
    matrix: &SparseMatrixQ,
) -> Result<(), SolverError> {
    if ctx
        .options
        .max_matrix_rows
        .is_some_and(|limit| matrix.rows > limit)
        || ctx
            .options
            .max_matrix_cols
            .is_some_and(|limit| matrix.cols > limit)
    {
        return Err(SolverError {
            target: None,
            kind: SolverErrorKind::Failure(FailureKind::FiniteResourceFailure {
                stage: StageId("TargetRelationSearchKernel".to_owned()),
                block_id: Some(plan.block_id),
                matrix_rows: Some(matrix.rows),
                matrix_cols: Some(matrix.cols),
                matrix_density: Some(matrix_density(matrix)),
                quotient_rank_estimate: None,
                coefficient_height_bits: None,
                memory_bytes: ctx.options.max_memory_bytes,
            }),
        });
    }
    Ok(())
}

fn algorithmic_hard_case(ctx: &KernelContext, reason: &str) -> SolverError {
    SolverError {
        target: Some(ctx.system.target),
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId("TargetRelationSearchKernel".to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash: ctx.block.block_hash,
        }),
    }
}

fn algorithmic_hard_case_with_traces(
    ctx: &KernelContext,
    reason: &str,
    traces: &[MembershipMatrixTrace],
) -> SolverError {
    let trace_hash = hash_sequence(
        "target-relation-search-exhaustion-trace",
        &traces
            .iter()
            .map(|trace| trace.trace_hash.0.to_vec())
            .collect::<Vec<_>>(),
    );
    SolverError {
        target: Some(ctx.system.target),
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId("TargetRelationSearchKernel".to_owned()),
            reason: AlgebraicReason(format!("{reason}; trace={trace_hash:?}")),
            minimal_block_hash: ctx.block.block_hash,
        }),
    }
}

fn finish_admission(
    block: &ProjectionBlock,
    status: KernelAdmissionStatus,
    execution_plan: Option<KernelExecutionPlan>,
) -> KernelAdmission {
    let mut chunks = vec![
        b"TargetRelationSearch".to_vec(),
        block.block_id.0.to_be_bytes().to_vec(),
        format!("{status:?}").into_bytes(),
    ];
    if let Some(plan) = &execution_plan {
        chunks.push(plan.plan_hash.0.to_vec());
    }
    KernelAdmission {
        kind: KernelKind::TargetRelationSearch,
        block_id: block.block_id,
        status,
        exported_variables: block.exported_variables.iter().copied().collect(),
        eliminated_variables: block
            .local_variables
            .difference(&block.exported_variables)
            .copied()
            .collect(),
        execution_plan,
        admission_hash: hash_sequence("kernel-admission", &chunks),
    }
}

fn projection_message_hash(message: &ProjectionMessage) -> Hash {
    let mut chunks = vec![
        message.package_id.0.to_be_bytes().to_vec(),
        message.block_id.0.to_be_bytes().to_vec(),
        format!("{:?}", message.kernel_kind).into_bytes(),
        message.certificate.certificate_hash.0.to_vec(),
    ];
    for relation in &message.relation_generators {
        chunks.push(relation.hash.0.to_vec());
    }
    hash_sequence("projection-message", &chunks)
}

fn implementation_bug(message: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::ImplementationBug {
            invariant_violated: message.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::algebra::linear_solve::{
        solve_homogeneous_modular, MatrixBuilder, ModularSolvePlan,
    };
    use crate::kernels::target_relation_search::{
        build_dense_relation_search_schedule, build_export_monomial_support,
        build_membership_matrix_builder, build_multiplier_supports, execute_target_relation_search,
        reconstruct_and_verify_relation, relation_search_default_export_degree_cap,
        RelationSearchBound,
    };
    use crate::kernels::traits::KernelKind;
    use crate::planner::admission::{collect_kernel_admissions, KernelAdmissionStatus};
    use crate::planner::probes::run_cost_probes;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::result::status::SolverStatus;
    use crate::solver::options::SolverOptions;
    use crate::types::hash::{hash_sequence, Hash};
    use crate::types::ids::{BlockId, VariableId};
    use crate::types::monomial::{
        monomial_degree, monomial_to_bytes, normalize_monomial, Monomial,
    };
    use crate::types::polynomial::{
        clear_denominators_primitive, constant_poly, poly_add, poly_mul, poly_sub, poly_variables,
        variable_poly, SparsePolynomialQ,
    };
    use crate::types::rational::int_q;

    #[test]
    fn rgq042_dense_schedule_is_recomputable_from_j_y_z_and_options() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let j = vec![
            poly_add(
                &poly_mul(&variable_poly(x), &variable_poly(x)),
                &variable_poly(t),
            ),
            poly_add(
                &poly_mul(&variable_poly(x), &variable_poly(y)),
                &variable_poly(t),
            ),
        ];
        let eliminated = vec![x, y];
        let exported = vec![t];
        let options = SolverOptions {
            max_relation_search_export_degree: Some(4),
            ..SolverOptions::default()
        };

        let cap = relation_search_default_export_degree_cap(&j, &eliminated, &exported);
        assert!(cap >= 2);

        let schedule_a = build_dense_relation_search_schedule(&j, &eliminated, &exported, &options);
        let schedule_b = build_dense_relation_search_schedule(&j, &eliminated, &exported, &options);
        assert_eq!(schedule_a, schedule_b);
        assert_eq!(schedule_a.stages.first().unwrap().export_degree, 1);
        assert_eq!(schedule_a.stages.last().unwrap().export_degree, 4);
        assert!(schedule_a
            .stages
            .windows(2)
            .all(|pair| pair[0].export_degree + 1 == pair[1].export_degree));

        let bound = RelationSearchBound {
            export_degree: 2,
            multiplier_total_degree: 4,
        };
        assert_eq!(build_export_monomial_support(&exported, &bound).len(), 3);
        let multiplier_supports = build_multiplier_supports(&j, &eliminated, &exported, &bound);
        assert_eq!(multiplier_supports.len(), j.len());
        assert!(multiplier_supports
            .iter()
            .all(|support| !support.is_empty()));
    }

    #[test]
    fn rgq042_option_cap_is_not_silently_widened() {
        let t = VariableId(0);
        let x = VariableId(1);
        let j = vec![poly_add(
            &poly_mul(&variable_poly(t), &variable_poly(t)),
            &variable_poly(x),
        )];
        let schedule = build_dense_relation_search_schedule(
            &j,
            &[x],
            &[t],
            &SolverOptions {
                max_relation_search_export_degree: Some(1),
                ..SolverOptions::default()
            },
        );

        assert_eq!(schedule.z_seed, 2);
        assert_eq!(schedule.e_cap, 1);
        assert!(schedule.stages.is_empty());
    }

    #[test]
    fn rgq055_schedule_reproducibility_covers_three_local_shapes() {
        let t = VariableId(0);
        let s = VariableId(1);
        let u = VariableId(2);
        let x = VariableId(3);
        let y = VariableId(4);
        let z = VariableId(5);

        let cases = vec![
            (
                vec![poly_add(
                    &poly_mul(&variable_poly(x), &variable_poly(x)),
                    &variable_poly(t),
                )],
                vec![x],
                vec![t],
                1,
                3,
                2,
            ),
            (
                vec![poly_add(
                    &poly_mul(
                        &poly_mul(&variable_poly(x), &variable_poly(x)),
                        &variable_poly(y),
                    ),
                    &poly_mul(&variable_poly(t), &variable_poly(s)),
                )],
                vec![x, y],
                vec![t, s],
                2,
                4,
                3,
            ),
            (
                vec![poly_add(
                    &poly_mul(
                        &poly_mul(
                            &poly_mul(&variable_poly(x), &variable_poly(x)),
                            &variable_poly(y),
                        ),
                        &variable_poly(z),
                    ),
                    &poly_mul(
                        &poly_mul(&variable_poly(t), &variable_poly(s)),
                        &variable_poly(u),
                    ),
                )],
                vec![x, y, z],
                vec![t, s, u],
                3,
                5,
                4,
            ),
        ];

        for (relations, eliminated, exported, expected_z_seed, cap, expected_d_max) in cases {
            let schedule = build_dense_relation_search_schedule(
                &relations,
                &eliminated,
                &exported,
                &SolverOptions {
                    max_relation_search_export_degree: Some(cap),
                    ..SolverOptions::default()
                },
            );
            assert_eq!(schedule.z_seed, expected_z_seed);
            assert_eq!(schedule.e_cap, cap);
            assert_eq!(schedule.d_max, expected_d_max);
            assert_eq!(
                schedule
                    .stages
                    .iter()
                    .map(|stage| stage.export_degree)
                    .collect::<Vec<_>>(),
                (expected_z_seed..=cap).collect::<Vec<_>>()
            );

            let all_variables = test_sorted_union(&eliminated, &exported);
            for stage in &schedule.stages {
                let export_support =
                    test_monomials_total_degree_leq(&exported, stage.export_degree);
                assert_eq!(
                    stage.export_support_hash,
                    test_hash_monomials("rgq042-export-support", &export_support)
                );

                let multiplier_supports = relations
                    .iter()
                    .map(|relation| {
                        let multiplier_degree = stage
                            .multiplier_total_degree
                            .saturating_sub(test_poly_total_degree(relation));
                        test_monomials_total_degree_leq(&all_variables, multiplier_degree)
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    stage.multiplier_support_hashes,
                    multiplier_supports
                        .iter()
                        .map(|support| test_hash_monomials("rgq042-multiplier-support", support))
                        .collect::<Vec<_>>()
                );

                let row_monomials =
                    test_row_monomials(&relations, &export_support, &multiplier_supports);
                let matrix_cols = export_support.len()
                    + multiplier_supports
                        .iter()
                        .map(|support| support.len())
                        .sum::<usize>();
                assert_eq!(
                    stage.row_monomial_hash,
                    test_hash_monomials("rgq042-row-monomials", &row_monomials)
                );
                assert_eq!(stage.row_monomial_count, row_monomials.len());
                assert_eq!(stage.matrix_rows, row_monomials.len());
                assert_eq!(stage.matrix_cols, matrix_cols);
            }
        }
    }

    #[test]
    fn p8a_target_relation_search_produces_multi_separator_relation() {
        let t = VariableId(0);
        let s = VariableId(1);
        let x = VariableId(2);
        let relations = vec![
            variable_poly(x),
            poly_sub(
                &variable_poly(x),
                &poly_add(&variable_poly(t), &variable_poly(s)),
            ),
        ];
        let (message, _ctx) = execute_case(vec![t, s, x], t, relations, [t, s, x], [t, s], Some(2));
        assert_eq!(message.kernel_kind, KernelKind::TargetRelationSearch);
        assert_eq!(message.exported_variables, vec![t, s]);
        let exported = [t, s].into_iter().collect();
        assert!(message
            .relation_generators
            .iter()
            .all(|relation| poly_variables(relation).is_subset(&exported)));
        assert_eq!(
            message.relation_generators[0],
            poly_add(&variable_poly(t), &variable_poly(s))
        );
    }

    #[test]
    fn p8a_target_relation_search_produces_bilinear_quadratic_relation() {
        let t = VariableId(0);
        let s = VariableId(1);
        let x = VariableId(2);
        let relations = vec![
            poly_sub(&variable_poly(x), &variable_poly(t)),
            poly_sub(
                &poly_mul(&variable_poly(x), &variable_poly(s)),
                &constant_poly(int_q(1)),
            ),
        ];
        let (message, _ctx) = execute_case(vec![t, s, x], t, relations, [t, s, x], [t, s], Some(3));
        assert_eq!(
            message.relation_generators[0],
            clear_denominators_primitive(&poly_sub(
                &poly_mul(&variable_poly(t), &variable_poly(s)),
                &constant_poly(int_q(1))
            ))
        );
    }

    #[test]
    fn p8a_target_relation_search_produces_one_large_block_relation() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let relations = vec![
            poly_sub(&variable_poly(x), &variable_poly(y)),
            poly_sub(&variable_poly(y), &variable_poly(t)),
            poly_sub(
                &poly_mul(&variable_poly(x), &variable_poly(x)),
                &constant_poly(int_q(2)),
            ),
        ];
        let (message, _ctx) = execute_case(vec![t, x, y], t, relations, [t, x, y], [t], Some(3));
        assert_eq!(
            message.relation_generators[0],
            clear_denominators_primitive(&poly_sub(
                &poly_mul(&variable_poly(t), &variable_poly(t)),
                &constant_poly(int_q(2))
            ))
        );
    }

    #[test]
    fn p8a_target_relation_search_exhaustion_is_hardcase_not_nonfinite() {
        let t = VariableId(0);
        let x = VariableId(1);
        let relations = vec![poly_sub(&variable_poly(x), &variable_poly(t))];
        let compressed = compressed_system(vec![t, x], t, relations);
        let block = test_block(&compressed, [t, x], [t]);
        let mut ctx = new_context(SolverOptions {
            max_relation_search_export_degree: Some(1),
            ..SolverOptions::default()
        });
        let probes = run_cost_probes(&block, &compressed, &mut ctx);
        let admissions = collect_kernel_admissions(&block, &compressed, &probes, &ctx);
        let plan = admissions
            .into_iter()
            .find(|admission| admission.kind == KernelKind::TargetRelationSearch)
            .and_then(|admission| admission.execution_plan)
            .unwrap();
        let mut kctx = crate::kernels::traits::KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let err = execute_target_relation_search(&plan, &mut kctx, &mut ctx).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::AlgorithmicHardCase);
    }

    #[test]
    fn p8a_exact_membership_verification_rejects_tampered_multiplier() {
        let t = VariableId(0);
        let s = VariableId(1);
        let x = VariableId(2);
        let relations = vec![
            variable_poly(x),
            poly_sub(
                &variable_poly(x),
                &poly_add(&variable_poly(t), &variable_poly(s)),
            ),
        ];
        let relation = poly_add(&variable_poly(t), &variable_poly(s));
        let multipliers = vec![constant_poly(int_q(1)), constant_poly(int_q(-1))];
        assert!(super::verify_membership_exact(
            &relation,
            &multipliers,
            &relations
        ));
        let tampered = vec![constant_poly(int_q(1)), constant_poly(int_q(1))];
        assert!(!super::verify_membership_exact(
            &relation, &tampered, &relations
        ));
    }

    #[test]
    fn p8a_required_rgq042_public_api_reconstructs_only_verified_candidates() {
        let t = VariableId(0);
        let s = VariableId(1);
        let x = VariableId(2);
        let relations = vec![
            variable_poly(x),
            poly_sub(
                &variable_poly(x),
                &poly_add(&variable_poly(t), &variable_poly(s)),
            ),
        ];
        let bound = RelationSearchBound {
            export_degree: 1,
            multiplier_total_degree: 2,
        };
        let builder = build_membership_matrix_builder(&relations, &bound);
        let ns = solve_homogeneous_modular(
            MatrixBuilder {
                matrix: builder.matrix.clone(),
            },
            ModularSolvePlan {
                seed: 101,
                max_primes: 4,
                stable_rank_after: 2,
                reconstruction_height_bound: None,
            },
        );
        let verified = reconstruct_and_verify_relation(&ns, &bound, &relations).unwrap();
        assert!(!verified.is_empty());
        assert!(verified
            .iter()
            .all(|candidate| super::verify_membership_exact(
                &candidate.relation,
                &candidate.multipliers,
                &relations
            )));
    }

    fn test_poly_total_degree(poly: &SparsePolynomialQ) -> usize {
        poly.terms
            .iter()
            .map(|term| {
                term.monomial
                    .exponents
                    .iter()
                    .map(|(_, exp)| *exp as usize)
                    .sum::<usize>()
            })
            .max()
            .unwrap_or(0)
    }

    fn test_sorted_union(a: &[VariableId], b: &[VariableId]) -> Vec<VariableId> {
        a.iter()
            .chain(b.iter())
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn test_monomials_total_degree_leq(
        variables: &[VariableId],
        max_degree: usize,
    ) -> Vec<Monomial> {
        let variables = variables
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let mut out = Vec::new();
        let mut current = Vec::new();
        test_enumerate_monomials(&variables, 0, max_degree as u32, &mut current, &mut out);
        out.sort_by(|a, b| (monomial_degree(a), a).cmp(&(monomial_degree(b), b)));
        out
    }

    fn test_enumerate_monomials(
        variables: &[VariableId],
        index: usize,
        remaining: u32,
        current: &mut Vec<(VariableId, u32)>,
        out: &mut Vec<Monomial>,
    ) {
        if index == variables.len() {
            out.push(normalize_monomial(current.clone()));
            return;
        }
        for exponent in 0..=remaining {
            if exponent > 0 {
                current.push((variables[index], exponent));
            }
            test_enumerate_monomials(variables, index + 1, remaining - exponent, current, out);
            if exponent > 0 {
                current.pop();
            }
        }
    }

    fn test_row_monomials(
        relations: &[SparsePolynomialQ],
        export_support: &[Monomial],
        multiplier_supports: &[Vec<Monomial>],
    ) -> Vec<Monomial> {
        let mut rows = export_support.iter().cloned().collect::<BTreeSet<_>>();
        for (relation, support) in relations.iter().zip(multiplier_supports.iter()) {
            for multiplier in support {
                for term in &relation.terms {
                    rows.insert(test_monomial_mul(multiplier, &term.monomial));
                }
            }
        }
        rows.into_iter().collect()
    }

    fn test_monomial_mul(a: &Monomial, b: &Monomial) -> Monomial {
        let mut exponents = BTreeMap::<VariableId, u32>::new();
        for (var, exp) in a.exponents.iter().chain(b.exponents.iter()) {
            *exponents.entry(*var).or_default() += *exp;
        }
        normalize_monomial(exponents.into_iter().collect())
    }

    fn test_hash_monomials(tag: &str, monomials: &[Monomial]) -> Hash {
        hash_sequence(
            tag,
            &monomials.iter().map(monomial_to_bytes).collect::<Vec<_>>(),
        )
    }

    fn execute_case<const N: usize, const M: usize>(
        variables: Vec<VariableId>,
        target: VariableId,
        relations: Vec<SparsePolynomialQ>,
        local_variables: [VariableId; N],
        exported_variables: [VariableId; M],
        cap: Option<usize>,
    ) -> (
        crate::compose::message::ProjectionMessage,
        crate::kernels::traits::KernelContext,
    ) {
        let compressed = compressed_system(variables.clone(), target, relations);
        let block = test_block(&compressed, local_variables, exported_variables);
        let mut ctx = new_context(SolverOptions {
            max_relation_search_export_degree: cap,
            ..SolverOptions::default()
        });
        let probes = run_cost_probes(&block, &compressed, &mut ctx);
        let admissions = collect_kernel_admissions(&block, &compressed, &probes, &ctx);
        let admission = admissions
            .into_iter()
            .find(|admission| admission.kind == KernelKind::TargetRelationSearch)
            .unwrap();
        assert!(matches!(admission.status, KernelAdmissionStatus::Admitted));
        let plan = admission.execution_plan.unwrap();
        let mut kctx = crate::kernels::traits::KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let message = execute_target_relation_search(&plan, &mut kctx, &mut ctx).unwrap();
        (message, kctx)
    }

    fn compressed_system(
        variables: Vec<VariableId>,
        target: VariableId,
        relations: Vec<SparsePolynomialQ>,
    ) -> crate::preprocess::compression::CompressedSystemQ {
        let canonical = canonicalize_system(
            validate_input(make_problem(variables, target, relations, Vec::new())).unwrap(),
        )
        .unwrap();
        CompressionState::from_system(canonical).to_compressed_system()
    }

    fn test_block<const N: usize, const M: usize>(
        compressed: &crate::preprocess::compression::CompressedSystemQ,
        local_variables: [VariableId; N],
        exported_variables: [VariableId; M],
    ) -> crate::graph::projection_dag::ProjectionBlock {
        let mut block = crate::graph::projection_dag::ProjectionBlock {
            block_id: BlockId(0),
            local_variables: local_variables.into_iter().collect(),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: exported_variables.into_iter().collect(),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash =
            crate::graph::projection_dag::authorize_block_relations(&block, compressed);
        block
    }
}
