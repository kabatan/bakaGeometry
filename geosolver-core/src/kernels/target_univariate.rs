use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, Zero};

use crate::compose::message::{MessageRepresentation, ProjectionMessage, ProjectionStrength};
use crate::graph::projection_dag::ProjectionBlock;
use crate::kernels::traits::{KernelContext, KernelKind, ReplayResult, TargetProjectionKernel};
use crate::planner::admission::{KernelAdmission, KernelAdmissionStatus};
use crate::planner::kernel_plan::{
    planned_failure_behavior, rank_plan, resource_bounds_hash, support_plan_hash, template_plan,
    CertificateRoute, KernelExecutionPlan, KernelSupportPlan, LocalNonfinitePolicy, ResourceBounds,
};
use crate::preprocess::compression::CompressedSystemQ;
use crate::problem::canonicalize::CanonicalRelationQ;
use crate::problem::context::SolverContext;
use crate::result::cost_trace::ProjectionCostTrace;
use crate::result::status::{FailureKind, SolverError, SolverErrorKind, StageId};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{KernelPlanId, PackageId, VariableId};
use crate::types::monomial::normalize_monomial;
use crate::types::polynomial::{normalize_poly, poly_variables, SparsePolynomialQ, TermQ};
use crate::types::rational::{lcm_denominators, one_q, RationalQ};
use crate::types::univariate::{normalize_univariate, squarefree_part_uni, UniPolynomialQ};
use crate::verify::certificates::{
    KernelCertificate, KernelCertificatePayload, TargetOnlySupportCertificate,
};

pub struct TargetUnivariateKernel;

impl TargetProjectionKernel for TargetUnivariateKernel {
    fn kind(&self) -> KernelKind {
        KernelKind::TargetUnivariate
    }

    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission {
        admit_target_univariate_with_messages(
            block,
            &ctx.system,
            &ctx.child_messages,
            &SolverContext::new(Default::default()),
        )
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
            .ok_or_else(|| implementation_bug("target-univariate admission has no plan"))
    }

    fn execute(
        &self,
        plan: &KernelExecutionPlan,
        ctx: &mut KernelContext,
        solver_ctx: &mut SolverContext,
    ) -> Result<ProjectionMessage, SolverError> {
        execute_target_univariate(plan, ctx, solver_ctx)
    }

    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult {
        crate::kernels::traits::exact_replay_result(
            self.kind(),
            "target-univariate-replay",
            message,
            ctx,
        )
    }
}

pub fn admit_target_univariate(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    ctx: &SolverContext,
) -> KernelAdmission {
    admit_target_univariate_with_messages(block, system, &[], ctx)
}

pub fn admit_target_univariate_with_messages(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    child_messages: &[ProjectionMessage],
    ctx: &SolverContext,
) -> KernelAdmission {
    let target = system.target;
    let target_inputs = collect_target_relation_inputs(block, system, child_messages, target);
    if target_inputs.is_empty() {
        return declined(block, "no target-only relation");
    }
    let relation_ids = target_inputs
        .iter()
        .flat_map(|input| input.source_relation_ids.iter().copied())
        .collect::<Vec<_>>();
    let relation_hashes = target_inputs
        .iter()
        .map(|input| input.source_hash)
        .collect::<Vec<_>>();
    let child_message_hashes = dedup_hashes_in_order(
        target_inputs
            .iter()
            .filter_map(|input| input.child_message_hash)
            .collect(),
    );
    let degree_bound = target_inputs
        .iter()
        .map(|input| crate::types::polynomial::poly_total_degree(&input.polynomial) as usize)
        .max()
        .unwrap_or(1);
    let mut support_plan = KernelSupportPlan {
        dense_relation_search_schedule: None,
        sparse_relation_search_schedule: None,
        affine_elimination_order: None,
        template_plan: Some(template_plan(
            1,
            relation_ids.len().max(1),
            hash_sequence("target-univariate-row", &[]),
            hash_sequence("target-univariate-column", &[]),
        )),
        rank_plan: Some(rank_plan(1)),
        universal_strategy_sequence: Vec::new(),
        degree_bound,
        support_hash: hash_sequence("kernel-support-plan", &[]),
    };
    support_plan.support_hash = support_plan_hash(&support_plan);
    let mut resource_bounds = ResourceBounds {
        max_matrix_rows: Some(1),
        max_matrix_cols: Some(relation_ids.len().max(1)),
        max_export_degree: Some(degree_bound),
        max_multiplier_total_degree: None,
        max_local_elimination_steps: Some(0),
        max_memory_bytes: ctx.options.max_memory_bytes,
        bounds_hash: hash_sequence("planner-resource-bounds", &[]),
    };
    resource_bounds.bounds_hash = resource_bounds_hash(&resource_bounds);
    let failure_behavior = planned_failure_behavior(
        vec![crate::result::status::SolverStatus::CertificateDesignGap],
        LocalNonfinitePolicy::NotApplicable,
    );
    let plan = KernelExecutionPlan::new(
        KernelPlanId(KernelKind::TargetUnivariate as u32),
        block.block_id,
        KernelKind::TargetUnivariate,
        block.authorization_hash,
        relation_ids,
        relation_hashes,
        block.child_block_ids.clone(),
        child_message_hashes,
        vec![target],
        block
            .local_variables
            .iter()
            .copied()
            .filter(|var| *var != target)
            .collect(),
        support_plan,
        resource_bounds,
        CertificateRoute::SourceMembershipCertificate,
        failure_behavior,
    );
    finish_admission(block, KernelAdmissionStatus::Admitted, Some(plan))
}

pub fn execute_target_univariate(
    plan: &KernelExecutionPlan,
    ctx: &mut KernelContext,
    solver_ctx: &mut SolverContext,
) -> Result<ProjectionMessage, SolverError> {
    crate::problem::context::check_resource(
        solver_ctx,
        StageId("TargetUnivariate::execute_start".to_owned()),
    )?;
    if plan.kernel_kind != KernelKind::TargetUnivariate {
        return Err(implementation_bug(
            "target-univariate execute received wrong plan kind",
        ));
    }
    if crate::planner::kernel_plan::hash_kernel_execution_plan(plan) != plan.plan_hash {
        return Err(implementation_bug(
            "target-univariate execution plan hash mismatch",
        ));
    }
    validate_target_plan_binding(plan, ctx)?;
    let target = ctx.system.target;
    let relation_inputs = collect_planned_target_inputs(plan, ctx, target);
    validate_planned_target_sources(plan, &relation_inputs)?;
    let relations = relation_inputs
        .iter()
        .map(|input| input.polynomial.clone())
        .collect::<Vec<_>>();
    crate::problem::context::check_resource_work(
        solver_ctx,
        StageId("TargetUnivariate::inputs_materialized".to_owned()),
        relations
            .iter()
            .map(|relation| relation.terms.len())
            .sum::<usize>()
            .max(1) as u128,
    )?;
    let support = target_only_support_from_polynomials(&relations, target)
        .ok_or_else(|| implementation_bug("target-univariate admission invalid"))?;
    crate::problem::context::check_resource_work(
        solver_ctx,
        StageId("TargetUnivariate::support_selected".to_owned()),
        support.terms.len().max(1) as u128,
    )?;
    let certificate_hash = hash_sequence(
        "target-univariate-certificate",
        &[
            plan.plan_hash.0.to_vec(),
            support.hash.0.to_vec(),
            ctx.block.authorization_hash.0.to_vec(),
        ],
    );
    let cost_trace = ProjectionCostTrace {
        block_id: plan.block_id,
        kernel_kind: KernelKind::TargetUnivariate,
        local_variable_count: ctx.block.local_variables.len(),
        exported_variable_count: 1,
        local_relation_count: relations.len(),
        local_monomial_count: relations.iter().map(|relation| relation.terms.len()).sum(),
        estimated_quotient_rank: Some(1),
        matrix_rows: Some(1),
        matrix_cols: Some(relations.len().max(1)),
        matrix_density: None,
        coefficient_height_before_bits: crate::preprocess::compression::max_coefficient_height_bits(
            &collect_planned_block_relations(plan, ctx),
        ),
        coefficient_height_after_bits: crate::types::polynomial::poly_coefficient_height_bits(
            &support,
        ),
        route_cost: Some(ProjectionCostTrace::route_cost_from_plan(plan)),
    };
    let certificate = KernelCertificate::from_execution_plan_with_payload(
        plan,
        std::slice::from_ref(&support),
        certificate_hash,
        KernelCertificatePayload::TargetOnlySupport(TargetOnlySupportCertificate {
            target,
            source_relations: relations.clone(),
            support_relation: support.clone(),
        }),
    );
    let mut message = ProjectionMessage {
        package_id: PackageId(plan.plan_id.0),
        block_id: plan.block_id,
        kernel_kind: KernelKind::TargetUnivariate,
        source_relation_ids: plan.source_relation_ids.clone(),
        eliminated_variables: plan.eliminated_variables.clone(),
        exported_variables: vec![target],
        relation_generators: vec![support],
        representation: MessageRepresentation::PrincipalSupport,
        projection_strength: ProjectionStrength::CandidateCoverStrong,
        certificate,
        compression_trace: ctx.system.compression_trace.clone(),
        cost_trace,
        package_hash: hash_sequence("projection-message-initial", &[]),
    };
    message.package_hash = projection_message_hash(&message);
    Ok(message)
}

pub fn target_only_support_from_relations(
    relations: &[CanonicalRelationQ],
    target: VariableId,
) -> Option<SparsePolynomialQ> {
    target_only_support_from_polynomials(
        &relations
            .iter()
            .map(|relation| relation.polynomial.clone())
            .collect::<Vec<_>>(),
        target,
    )
}

pub fn target_only_support_from_polynomials(
    relations: &[SparsePolynomialQ],
    target: VariableId,
) -> Option<SparsePolynomialQ> {
    let target_set = [target].into_iter().collect();
    let mut support = UniPolynomialQ {
        variable: target,
        coeffs_low_to_high: vec![one_q()],
        hash: hash_sequence("univariate", &[]),
    };
    let mut found = false;
    for relation in relations {
        if relation.terms.is_empty() || !poly_variables(relation).is_subset(&target_set) {
            continue;
        }
        let uni = polynomial_to_univariate(relation, target)?;
        if uni.coeffs_low_to_high.is_empty() {
            continue;
        }
        let squarefree = squarefree_part_uni(&uni);
        support = univariate_mul(&support, &squarefree);
        support = squarefree_part_uni(&support);
        found = true;
    }
    found.then(|| univariate_to_polynomial(&support))
}

#[derive(Debug, Clone)]
struct TargetRelationInput {
    polynomial: SparsePolynomialQ,
    source_relation_ids: Vec<crate::types::ids::RelationId>,
    source_hash: Hash,
    child_message_hash: Option<Hash>,
}

fn collect_target_relation_inputs(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    child_messages: &[ProjectionMessage],
    target: VariableId,
) -> Vec<TargetRelationInput> {
    let target_set = [target].into_iter().collect();
    let mut inputs = block_relations(block, system)
        .into_iter()
        .filter(|relation| {
            !relation.polynomial.terms.is_empty()
                && poly_variables(&relation.polynomial).is_subset(&target_set)
        })
        .map(|relation| TargetRelationInput {
            polynomial: relation.polynomial.clone(),
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
            child_message_hash: None,
        })
        .collect::<Vec<_>>();
    for message in child_messages {
        for relation in &message.relation_generators {
            if relation.terms.is_empty() || !poly_variables(relation).is_subset(&target_set) {
                continue;
            }
            inputs.push(TargetRelationInput {
                polynomial: relation.clone(),
                source_relation_ids: message.source_relation_ids.clone(),
                source_hash: relation.hash,
                child_message_hash: Some(message.package_hash),
            });
        }
    }
    inputs
}

fn collect_planned_target_inputs(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
    target: VariableId,
) -> Vec<TargetRelationInput> {
    let target_set = [target].into_iter().collect();
    let relation_ids = plan
        .source_relation_ids
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let source_hashes = plan
        .source_relation_hashes
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let child_message_hashes = plan
        .child_message_hashes
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let mut relations = ctx
        .system
        .relations
        .iter()
        .filter(|relation| {
            relation_ids.contains(&relation.id)
                && source_hashes.contains(&relation.hash)
                && !relation.polynomial.terms.is_empty()
                && poly_variables(&relation.polynomial).is_subset(&target_set)
        })
        .map(|relation| TargetRelationInput {
            polynomial: relation.polynomial.clone(),
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
            child_message_hash: None,
        })
        .collect::<Vec<_>>();
    for message in &ctx.child_messages {
        if !child_message_hashes.contains(&message.package_hash) {
            continue;
        }
        for relation in &message.relation_generators {
            if source_hashes.contains(&relation.hash)
                && !relation.terms.is_empty()
                && poly_variables(relation).is_subset(&target_set)
            {
                relations.push(TargetRelationInput {
                    polynomial: relation.clone(),
                    source_relation_ids: message.source_relation_ids.clone(),
                    source_hash: relation.hash,
                    child_message_hash: Some(message.package_hash),
                });
            }
        }
    }
    relations
}

fn collect_planned_block_relations(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Vec<CanonicalRelationQ> {
    let relation_ids = plan
        .source_relation_ids
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    ctx.system
        .relations
        .iter()
        .filter(|relation| relation_ids.contains(&relation.id))
        .cloned()
        .collect()
}

fn validate_target_plan_binding(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    if plan.block_id != ctx.block.block_id {
        return Err(implementation_bug("target-univariate block id mismatch"));
    }
    if plan.input_block_authorization_hash != ctx.block.authorization_hash {
        return Err(implementation_bug(
            "target-univariate block authorization hash mismatch",
        ));
    }
    let available_child_hashes = ctx
        .child_messages
        .iter()
        .map(|message| message.package_hash)
        .collect::<std::collections::BTreeSet<_>>();
    if plan
        .child_message_hashes
        .iter()
        .any(|hash| !available_child_hashes.contains(hash))
    {
        return Err(implementation_bug(
            "target-univariate child message hash missing",
        ));
    }
    Ok(())
}

fn validate_planned_target_sources(
    plan: &KernelExecutionPlan,
    relations: &[TargetRelationInput],
) -> Result<(), SolverError> {
    let mut expected = plan
        .source_relation_hashes
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let mut actual = relations
        .iter()
        .map(|relation| relation.source_hash)
        .collect::<Vec<_>>();
    expected.sort();
    actual.sort();
    if expected != actual {
        return Err(implementation_bug(
            "target-univariate source relation hash mismatch",
        ));
    }
    Ok(())
}

fn dedup_hashes_in_order(hashes: Vec<Hash>) -> Vec<Hash> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for hash in hashes {
        if seen.insert(hash) {
            out.push(hash);
        }
    }
    out
}

fn polynomial_to_univariate(
    poly: &SparsePolynomialQ,
    target: VariableId,
) -> Option<UniPolynomialQ> {
    let mut coeffs = Vec::<RationalQ>::new();
    for term in &poly.terms {
        let mut degree = 0_usize;
        for (var, exp) in &term.monomial.exponents {
            if *var != target {
                return None;
            }
            degree = *exp as usize;
        }
        if coeffs.len() <= degree {
            coeffs.resize_with(degree + 1, crate::types::rational::zero_q);
        }
        coeffs[degree] = crate::types::rational::add_q(&coeffs[degree], &term.coeff);
    }
    Some(normalize_univariate(UniPolynomialQ {
        variable: target,
        coeffs_low_to_high: coeffs,
        hash: hash_sequence("univariate", &[]),
    }))
}

fn univariate_to_polynomial(poly: &UniPolynomialQ) -> SparsePolynomialQ {
    let lcm = lcm_denominators(poly.coeffs_low_to_high.iter());
    let mut integer_coeffs = poly
        .coeffs_low_to_high
        .iter()
        .map(|coeff| &coeff.num * (&lcm / &coeff.den))
        .collect::<Vec<BigInt>>();
    let content = integer_coeffs
        .iter()
        .fold(BigInt::zero(), |acc, coeff| acc.gcd(&coeff.abs()));
    if !content.is_zero() && content != BigInt::one() {
        for coeff in &mut integer_coeffs {
            *coeff /= &content;
        }
    }
    if integer_coeffs
        .iter()
        .rev()
        .find(|coeff| !coeff.is_zero())
        .map(|coeff| coeff.is_negative())
        .unwrap_or(false)
    {
        for coeff in &mut integer_coeffs {
            *coeff = -coeff.clone();
        }
    }
    normalize_poly(SparsePolynomialQ {
        terms: poly
            .coeffs_low_to_high
            .iter()
            .enumerate()
            .zip(integer_coeffs)
            .filter(|((_, _), coeff)| !coeff.is_zero())
            .map(|((degree, _), coeff)| TermQ {
                coeff: RationalQ {
                    num: coeff,
                    den: BigInt::one(),
                },
                monomial: normalize_monomial(if degree == 0 {
                    Vec::new()
                } else {
                    vec![(poly.variable, degree as u32)]
                }),
            })
            .collect(),
        hash: hash_sequence("poly", &[]),
    })
}

fn univariate_mul(a: &UniPolynomialQ, b: &UniPolynomialQ) -> UniPolynomialQ {
    assert_eq!(a.variable, b.variable, "univariate variable mismatch");
    if a.coeffs_low_to_high.is_empty() || b.coeffs_low_to_high.is_empty() {
        return normalize_univariate(UniPolynomialQ {
            variable: a.variable,
            coeffs_low_to_high: Vec::new(),
            hash: hash_sequence("univariate", &[]),
        });
    }
    let mut coeffs = vec![
        crate::types::rational::zero_q();
        a.coeffs_low_to_high.len() + b.coeffs_low_to_high.len() - 1
    ];
    for (i, ai) in a.coeffs_low_to_high.iter().enumerate() {
        for (j, bj) in b.coeffs_low_to_high.iter().enumerate() {
            coeffs[i + j] = crate::types::rational::add_q(
                &coeffs[i + j],
                &crate::types::rational::mul_q(ai, bj),
            );
        }
    }
    normalize_univariate(UniPolynomialQ {
        variable: a.variable,
        coeffs_low_to_high: coeffs,
        hash: hash_sequence("univariate", &[]),
    })
}

fn block_relations<'a>(
    block: &ProjectionBlock,
    system: &'a CompressedSystemQ,
) -> Vec<&'a CanonicalRelationQ> {
    let ids = block
        .relation_ids
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    system
        .relations
        .iter()
        .filter(|relation| ids.contains(&relation.id))
        .collect()
}

fn finish_admission(
    block: &ProjectionBlock,
    status: KernelAdmissionStatus,
    execution_plan: Option<KernelExecutionPlan>,
) -> KernelAdmission {
    let mut chunks = vec![
        b"TargetUnivariate".to_vec(),
        block.block_id.0.to_be_bytes().to_vec(),
        format!("{status:?}").into_bytes(),
    ];
    if let Some(plan) = &execution_plan {
        chunks.push(plan.plan_hash.0.to_vec());
    }
    KernelAdmission {
        kind: KernelKind::TargetUnivariate,
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

fn declined(block: &ProjectionBlock, reason: &str) -> KernelAdmission {
    finish_admission(
        block,
        KernelAdmissionStatus::Declined {
            reason: reason.to_owned(),
        },
        None,
    )
}

fn projection_message_hash(message: &ProjectionMessage) -> Hash {
    crate::compose::message::hash_projection_message(message)
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
    use crate::compose::message::{MessageRepresentation, ProjectionMessage, ProjectionStrength};
    use crate::kernels::target_univariate::{
        admit_target_univariate, admit_target_univariate_with_messages, execute_target_univariate,
        target_only_support_from_relations, TargetUnivariateKernel,
    };
    use crate::kernels::traits::{KernelKind, TargetProjectionKernel};
    use crate::planner::admission::{collect_kernel_admissions, KernelAdmissionStatus};
    use crate::planner::kernel_plan::hash_kernel_execution_plan;
    use crate::planner::probes::run_cost_probes;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::result::cost_trace::ProjectionCostTrace;
    use crate::result::status::SolverStatus;
    use crate::solver::options::SolverOptions;
    use crate::types::hash::hash_sequence;
    use crate::types::ids::{BlockId, PackageId, VariableId};
    use crate::types::polynomial::{constant_poly, poly_mul, poly_sub, variable_poly};
    use crate::types::rational::int_q;
    use crate::verify::certificates::KernelCertificate;

    #[test]
    fn p7_target_univariate_support_is_primitive_lcm_squarefree() {
        let t = VariableId(0);
        let x = VariableId(1);
        let target_relation = poly_mul(
            &poly_sub(&variable_poly(t), &constant_poly(int_q(1))),
            &poly_sub(&variable_poly(t), &constant_poly(int_q(1))),
        );
        let non_target_relation = poly_sub(&variable_poly(x), &variable_poly(t));
        let compressed = compressed_system(t, vec![target_relation, non_target_relation]);
        let block = test_block(&compressed, [t, x], [t]);
        let mut ctx = new_context(SolverOptions::default());
        let admission = admit_target_univariate(&block, &compressed, &ctx);
        assert!(matches!(admission.status, KernelAdmissionStatus::Admitted));
        let plan = admission.execution_plan.clone().unwrap();
        let mut kctx = crate::kernels::traits::KernelContext {
            block,
            system: compressed.clone(),
            child_messages: Vec::new(),
        };
        let message = execute_target_univariate(&plan, &mut kctx, &mut ctx).unwrap();
        assert_eq!(message.kernel_kind, KernelKind::TargetUnivariate);
        assert_eq!(
            message.representation,
            MessageRepresentation::PrincipalSupport
        );
        assert_eq!(
            message.projection_strength,
            ProjectionStrength::CandidateCoverStrong
        );
        assert_eq!(message.exported_variables, vec![t]);
        assert_eq!(message.relation_generators.len(), 1);
        assert_eq!(
            message.relation_generators[0],
            poly_sub(&variable_poly(t), &constant_poly(int_q(1)))
        );
        assert_eq!(
            target_only_support_from_relations(&compressed.relations, t).unwrap(),
            message.relation_generators[0]
        );
        let kernel = TargetUnivariateKernel;
        assert!(kernel.replay(&message, &kctx).accepted);
        let mut tampered_message = message;
        tampered_message.package_hash = hash_sequence("tampered-package", &[]);
        assert!(!kernel.replay(&tampered_message, &kctx).accepted);
    }

    #[test]
    fn p7_target_univariate_uses_child_message_target_relation() {
        let t = VariableId(0);
        let x = VariableId(1);
        let non_target_relation = poly_sub(&variable_poly(x), &variable_poly(t));
        let compressed = compressed_system(t, vec![non_target_relation]);
        let mut block = test_block(&compressed, [t, x], [t]);
        block.child_block_ids = vec![BlockId(7)];
        block.authorization_hash =
            crate::graph::projection_dag::authorize_block_relations(&block, &compressed);
        let child_relation = poly_sub(&variable_poly(t), &constant_poly(int_q(3)));
        let child_message = child_projection_message(&compressed, child_relation.clone());
        let mut ctx = new_context(SolverOptions::default());
        let admission = admit_target_univariate_with_messages(
            &block,
            &compressed,
            std::slice::from_ref(&child_message),
            &ctx,
        );
        assert!(matches!(admission.status, KernelAdmissionStatus::Admitted));
        let plan = admission.execution_plan.clone().unwrap();
        assert_eq!(plan.child_message_hashes, vec![child_message.package_hash]);
        assert_eq!(plan.source_relation_hashes, vec![child_relation.hash]);
        let mut kctx = crate::kernels::traits::KernelContext {
            block,
            system: compressed,
            child_messages: vec![child_message],
        };
        let message = execute_target_univariate(&plan, &mut kctx, &mut ctx).unwrap();
        assert_eq!(message.relation_generators, vec![child_relation]);
    }

    #[test]
    fn p7_target_univariate_rejects_separator_only_planner_admission() {
        let t = VariableId(0);
        let s = VariableId(1);
        let relation = poly_sub(&variable_poly(s), &constant_poly(int_q(5)));
        let compressed = compressed_system(t, vec![relation]);
        let block = test_block(&compressed, [t, s], [t, s]);
        let mut ctx = new_context(SolverOptions::default());
        let probes = run_cost_probes(&block, &compressed, &mut ctx);
        let admissions = collect_kernel_admissions(&block, &compressed, &probes, &ctx);
        let target_admission = admissions
            .iter()
            .find(|admission| admission.kind == KernelKind::TargetUnivariate)
            .unwrap();
        assert!(matches!(
            target_admission.status,
            KernelAdmissionStatus::Declined { .. }
        ));
    }

    #[test]
    fn p7_target_univariate_rejects_auth_source_and_child_message_tamper() {
        let t = VariableId(0);
        let target_relation = poly_sub(&variable_poly(t), &constant_poly(int_q(2)));
        let compressed = compressed_system(t, vec![target_relation]);
        let block = test_block(&compressed, [t], [t]);
        let mut ctx = new_context(SolverOptions::default());
        let admission = admit_target_univariate(&block, &compressed, &ctx);
        let mut plan = admission.execution_plan.clone().unwrap();
        let mut kctx = crate::kernels::traits::KernelContext {
            block: block.clone(),
            system: compressed.clone(),
            child_messages: Vec::new(),
        };

        let mut bad_auth_ctx = kctx.clone();
        bad_auth_ctx.block.authorization_hash = hash_sequence("tampered-auth", &[]);
        let err = execute_target_univariate(&plan, &mut bad_auth_ctx, &mut ctx).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);

        plan.source_relation_hashes[0] = hash_sequence("tampered-source", &[]);
        plan.plan_hash = hash_kernel_execution_plan(&plan);
        let err = execute_target_univariate(&plan, &mut kctx, &mut ctx).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);

        let child_relation = poly_sub(&variable_poly(t), &constant_poly(int_q(4)));
        let child_message = child_projection_message(&compressed, child_relation);
        let child_admission = admit_target_univariate_with_messages(
            &block,
            &compressed,
            std::slice::from_ref(&child_message),
            &ctx,
        );
        let child_plan = child_admission.execution_plan.clone().unwrap();
        let mut child_ctx = crate::kernels::traits::KernelContext {
            block,
            system: compressed,
            child_messages: vec![child_message],
        };
        child_ctx.child_messages[0].package_hash = hash_sequence("tampered-child", &[]);
        let err = execute_target_univariate(&child_plan, &mut child_ctx, &mut ctx).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    fn compressed_system(
        target: VariableId,
        relations: Vec<crate::types::polynomial::SparsePolynomialQ>,
    ) -> crate::preprocess::compression::CompressedSystemQ {
        let mut variables = vec![target];
        variables.extend(
            relations
                .iter()
                .flat_map(crate::types::polynomial::poly_variables)
                .filter(|var| *var != target),
        );
        variables.sort();
        variables.dedup();
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

    fn child_projection_message(
        compressed: &crate::preprocess::compression::CompressedSystemQ,
        relation: crate::types::polynomial::SparsePolynomialQ,
    ) -> ProjectionMessage {
        let certificate_hash = hash_sequence("test-child-certificate", &[relation.hash.0.to_vec()]);
        let mut message = ProjectionMessage {
            package_id: PackageId(700),
            block_id: BlockId(7),
            kernel_kind: KernelKind::TargetUnivariate,
            source_relation_ids: Vec::new(),
            eliminated_variables: Vec::new(),
            exported_variables: vec![compressed.target],
            relation_generators: vec![relation],
            representation: MessageRepresentation::PrincipalSupport,
            projection_strength: ProjectionStrength::CandidateCoverStrong,
            certificate: KernelCertificate::synthetic_for_tests(certificate_hash),
            compression_trace: compressed.compression_trace.clone(),
            cost_trace: ProjectionCostTrace::default(),
            package_hash: hash_sequence("projection-message-initial", &[]),
        };
        message.package_hash = hash_sequence(
            "test-child-message",
            &[message.relation_generators[0].hash.0.to_vec()],
        );
        message
    }
}
