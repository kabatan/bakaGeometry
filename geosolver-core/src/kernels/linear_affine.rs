use crate::compose::message::{MessageRepresentation, ProjectionMessage, ProjectionStrength};
use crate::graph::projection_dag::ProjectionBlock;
use crate::kernels::traits::{KernelContext, KernelKind, ReplayResult, TargetProjectionKernel};
use crate::planner::admission::{KernelAdmission, KernelAdmissionStatus};
use crate::planner::kernel_plan::{
    affine_elimination_plan, affine_elimination_step, planned_failure_behavior, rank_plan,
    resource_bounds_hash, support_plan_hash, template_plan, AffineEliminationPlan,
    CertificateRoute, KernelExecutionPlan, KernelSupportPlan, LocalNonfinitePolicy, ResourceBounds,
};
use crate::preprocess::compression::{
    affine_parts_in_variable, max_coefficient_height_bits, relation_with_polynomial,
    substitute_rational_and_clear, CompressedSystemQ,
};
use crate::problem::canonicalize::CanonicalRelationQ;
use crate::problem::context::SolverContext;
use crate::result::cost_trace::ProjectionCostTrace;
use crate::result::status::{AlgebraicReason, FailureKind, SolverError, SolverErrorKind, StageId};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{KernelPlanId, PackageId, RelationId, VariableId};
use crate::types::polynomial::{
    clear_denominators_primitive, max_poly_coefficient_height_bits, poly_monomial_count,
    poly_scale, poly_variables, substitute_poly, SparsePolynomialQ, SubstitutionMap,
};
use crate::types::rational::{div_q, int_q, is_zero_q, neg_q};
use crate::verify::certificates::{
    GuardedAffineProjectionCertificate, KernelCertificate, KernelCertificatePayload,
};

pub struct LinearAffineKernel;

impl TargetProjectionKernel for LinearAffineKernel {
    fn kind(&self) -> KernelKind {
        KernelKind::LinearAffine
    }

    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission {
        admit_linear_affine(block, &ctx.system, &SolverContext::new(Default::default()))
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
            .ok_or_else(|| implementation_bug("linear-affine admission has no plan"))
    }

    fn execute(
        &self,
        plan: &KernelExecutionPlan,
        ctx: &mut KernelContext,
        solver_ctx: &mut SolverContext,
    ) -> Result<ProjectionMessage, SolverError> {
        execute_linear_affine(plan, ctx, solver_ctx)
    }

    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult {
        crate::kernels::traits::exact_replay_result(
            self.kind(),
            "linear-affine-replay",
            message,
            ctx,
        )
    }
}

pub fn admit_linear_affine(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    ctx: &SolverContext,
) -> KernelAdmission {
    let Some(order) = find_triangular_affine_order(block, system) else {
        return finish_admission(
            block,
            KernelAdmissionStatus::Declined {
                reason: "no complete safe triangular affine elimination order".to_owned(),
            },
            None,
        );
    };
    match plan_linear_affine(block, system, &order, ctx) {
        Ok(plan) => finish_admission(block, KernelAdmissionStatus::Admitted, Some(plan)),
        Err(_) => finish_admission(
            block,
            KernelAdmissionStatus::Declined {
                reason: "failed to build linear-affine execution plan".to_owned(),
            },
            None,
        ),
    }
}

pub fn find_triangular_affine_order(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
) -> Option<AffineEliminationPlan> {
    let exported = block.exported_variables.iter().copied().collect();
    let mut relations = block_relations(block, system);
    let mut steps = Vec::new();
    let eliminated = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    for variable in eliminated {
        if !relations
            .iter()
            .any(|relation| poly_variables(&relation.polynomial).contains(&variable))
        {
            continue;
        }
        let Some((relation_index, pivot, _rest, guard_hash)) =
            choose_safe_affine_pivot(&relations, variable, system)
        else {
            return None;
        };
        let source_relation_id = relations[relation_index].id;
        steps.push(affine_elimination_step(
            variable,
            source_relation_id,
            pivot.hash,
            guard_hash,
        ));
        relations =
            apply_affine_step_to_relations(relations, variable, source_relation_id, system)?;
    }
    if relations.iter().any(|relation| {
        !relation.polynomial.terms.is_empty()
            && !poly_variables(&relation.polynomial).is_subset(&exported)
    }) {
        return None;
    }
    if !relations.iter().any(|relation| {
        !relation.polynomial.terms.is_empty()
            && poly_variables(&relation.polynomial).is_subset(&exported)
    }) {
        return None;
    }
    if steps.is_empty() {
        None
    } else {
        Some(affine_elimination_plan(steps))
    }
}

pub fn plan_linear_affine(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    order: &AffineEliminationPlan,
    ctx: &SolverContext,
) -> Result<KernelExecutionPlan, SolverError> {
    let mut support_plan = KernelSupportPlan {
        dense_relation_search_schedule: None,
        sparse_relation_search_schedule: None,
        affine_elimination_order: Some(order.clone()),
        template_plan: Some(template_plan(
            block.relation_ids.len().max(1),
            order.steps.len().max(1),
            hash_sequence("linear-affine-row", &[]),
            hash_sequence("linear-affine-column", &[]),
        )),
        rank_plan: Some(rank_plan(order.steps.len().max(1))),
        universal_strategy_sequence: Vec::new(),
        degree_bound: system
            .relations
            .iter()
            .map(|relation| {
                crate::types::polynomial::poly_total_degree(&relation.polynomial) as usize
            })
            .max()
            .unwrap_or(1),
        support_hash: hash_sequence("kernel-support-plan", &[]),
    };
    support_plan.support_hash = support_plan_hash(&support_plan);
    let mut resource_bounds = ResourceBounds {
        max_matrix_rows: Some(block.relation_ids.len().max(1)),
        max_matrix_cols: Some(order.steps.len().max(1)),
        max_export_degree: None,
        max_multiplier_total_degree: None,
        max_local_elimination_steps: Some(order.steps.len()),
        max_memory_bytes: ctx.options.max_memory_bytes,
        bounds_hash: hash_sequence("planner-resource-bounds", &[]),
    };
    resource_bounds.bounds_hash = resource_bounds_hash(&resource_bounds);
    let relation_hashes = block
        .relation_ids
        .iter()
        .filter_map(|id| system.relations.iter().find(|relation| relation.id == *id))
        .map(|relation| relation.hash)
        .collect::<Vec<_>>();
    Ok(KernelExecutionPlan::new(
        KernelPlanId(KernelKind::LinearAffine as u32),
        block.block_id,
        KernelKind::LinearAffine,
        block.authorization_hash,
        block.relation_ids.clone(),
        relation_hashes,
        block.child_block_ids.clone(),
        Vec::new(),
        block.exported_variables.iter().copied().collect(),
        block
            .local_variables
            .difference(&block.exported_variables)
            .copied()
            .collect(),
        support_plan,
        resource_bounds,
        CertificateRoute::GuardedAffineProjectionCertificate,
        planned_failure_behavior(
            vec![
                crate::result::status::SolverStatus::AlgorithmicHardCase,
                crate::result::status::SolverStatus::CertificateDesignGap,
            ],
            LocalNonfinitePolicy::NotApplicable,
        ),
    ))
}

pub fn execute_linear_affine(
    plan: &KernelExecutionPlan,
    ctx: &mut KernelContext,
    solver_ctx: &mut SolverContext,
) -> Result<ProjectionMessage, SolverError> {
    crate::problem::context::check_resource(
        solver_ctx,
        StageId("LinearAffine::execute_start".to_owned()),
    )?;
    if plan.kernel_kind != KernelKind::LinearAffine {
        return Err(implementation_bug(
            "linear-affine execute received wrong plan kind",
        ));
    }
    if crate::planner::kernel_plan::hash_kernel_execution_plan(plan) != plan.plan_hash {
        return Err(implementation_bug(
            "linear-affine execution plan hash mismatch",
        ));
    }
    validate_linear_affine_plan_binding(plan, ctx)?;
    let Some(order) = &plan.support_plan.affine_elimination_order else {
        return Err(implementation_bug("linear-affine plan lacks affine order"));
    };
    let mut relations = block_relations(&ctx.block, &ctx.system);
    let source_relations = relations
        .iter()
        .map(|relation| relation.polynomial.clone())
        .collect::<Vec<_>>();
    for (step_index, step) in order.steps.iter().enumerate() {
        crate::problem::context::check_resource(
            solver_ctx,
            StageId(format!("LinearAffine::step::{step_index}")),
        )?;
        let Some(relation) = relations
            .iter()
            .find(|relation| relation.id == step.source_relation_id)
        else {
            return Err(implementation_bug(
                "linear-affine step source relation missing",
            ));
        };
        let Some((pivot, _rest)) =
            affine_parts_in_variable(&relation.polynomial, step.eliminated_variable)
        else {
            return Err(implementation_bug(
                "linear-affine step source is no longer affine",
            ));
        };
        if pivot.hash != step.pivot_hash {
            return Err(implementation_bug("linear-affine pivot hash mismatch"));
        }
        if !constant_nonzero(&pivot) {
            let guard_matches = step.denominator_guard_hash.is_some_and(|hash| {
                ctx.system
                    .guards
                    .iter()
                    .any(|guard| guard.guard_hash == hash && guard.factor.hash == pivot.hash)
            });
            if !guard_matches {
                return Err(implementation_bug("unsafe affine pivot in plan"));
            }
        }
        relations = apply_affine_step_to_relations(
            relations,
            step.eliminated_variable,
            step.source_relation_id,
            &ctx.system,
        )
        .ok_or_else(|| implementation_bug("linear-affine planned step could not be applied"))?;
    }
    let exported = plan.exported_variables.iter().copied().collect();
    let mut exported_relations = Vec::new();
    for relation in &relations {
        let relation_poly = clear_denominators_primitive(&relation.polynomial);
        if relation_poly.terms.is_empty() {
            continue;
        }
        if poly_variables(&relation_poly).is_subset(&exported) {
            exported_relations.push(relation_poly);
        } else {
            return Err(SolverError {
                target: Some(ctx.system.target),
                kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
                    stage: StageId("LinearAffineKernel".to_owned()),
                    reason: AlgebraicReason(
                        "affine elimination left a nonzero local-variable relation".to_owned(),
                    ),
                    minimal_block_hash: ctx.block.block_hash,
                }),
            });
        }
    }
    if exported_relations.is_empty() {
        return Err(SolverError {
            target: Some(ctx.system.target),
            kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
                stage: StageId("LinearAffineKernel".to_owned()),
                reason: AlgebraicReason(
                    "affine elimination produced no exported relation".to_owned(),
                ),
                minimal_block_hash: ctx.block.block_hash,
            }),
        });
    }
    let certificate_hash = hash_sequence(
        "linear-affine-certificate",
        &[
            plan.plan_hash.0.to_vec(),
            order.order_hash.0.to_vec(),
            ctx.block.authorization_hash.0.to_vec(),
        ],
    );
    let cost_trace = ProjectionCostTrace {
        block_id: plan.block_id,
        kernel_kind: KernelKind::LinearAffine,
        local_variable_count: ctx.block.local_variables.len(),
        exported_variable_count: plan.exported_variables.len(),
        local_relation_count: plan.source_relation_ids.len(),
        local_monomial_count: relations
            .iter()
            .map(|relation| poly_monomial_count(&relation.polynomial))
            .sum(),
        estimated_quotient_rank: Some(order.steps.len().max(1)),
        matrix_rows: Some(plan.source_relation_ids.len().max(1)),
        matrix_cols: Some(order.steps.len().max(1)),
        matrix_density: None,
        coefficient_height_before_bits: max_coefficient_height_bits(&ctx.system.relations),
        coefficient_height_after_bits: max_poly_coefficient_height_bits(&exported_relations),
        route_cost: Some(ProjectionCostTrace::route_cost_from_plan(plan)),
    };
    let certificate = KernelCertificate::from_execution_plan_with_payload(
        plan,
        &exported_relations,
        certificate_hash,
        KernelCertificatePayload::GuardedAffine(GuardedAffineProjectionCertificate {
            source_relation_ids: plan.source_relation_ids.clone(),
            source_relations,
            steps: order.steps.clone(),
            output_relations: exported_relations.clone(),
            affine_order_hash: order.order_hash,
        }),
    );
    let mut message = ProjectionMessage {
        package_id: PackageId(plan.plan_id.0),
        block_id: plan.block_id,
        kernel_kind: KernelKind::LinearAffine,
        source_relation_ids: plan.source_relation_ids.clone(),
        eliminated_variables: plan.eliminated_variables.clone(),
        exported_variables: plan.exported_variables.clone(),
        relation_generators: exported_relations,
        representation: MessageRepresentation::GeneratorSet,
        projection_strength: ProjectionStrength::CandidateCoverStrong,
        certificate,
        compression_trace: ctx.system.compression_trace.clone(),
        cost_trace,
        package_hash: hash_sequence("projection-message-initial", &[]),
    };
    message.package_hash = projection_message_hash(&message);
    Ok(message)
}

fn choose_safe_affine_pivot(
    relations: &[CanonicalRelationQ],
    variable: VariableId,
    system: &CompressedSystemQ,
) -> Option<(usize, SparsePolynomialQ, SparsePolynomialQ, Option<Hash>)> {
    relations.iter().enumerate().find_map(|(idx, relation)| {
        let (pivot, rest) = affine_parts_in_variable(&relation.polynomial, variable)?;
        if constant_nonzero(&pivot) {
            return Some((idx, pivot, rest, None));
        }
        let guard = system
            .guards
            .iter()
            .find(|guard| guard.factor.hash == pivot.hash)?;
        Some((idx, pivot, rest, Some(guard.guard_hash)))
    })
}

fn apply_affine_step_to_relations(
    relations: Vec<CanonicalRelationQ>,
    variable: VariableId,
    source_relation_id: RelationId,
    system: &CompressedSystemQ,
) -> Option<Vec<CanonicalRelationQ>> {
    let source = relations
        .iter()
        .find(|relation| relation.id == source_relation_id)?;
    let (pivot, rest) = affine_parts_in_variable(&source.polynomial, variable)?;
    let transformed = if let Some(constant) = constant_value(&pivot) {
        let scale = div_q(&neg_q(&int_q(1)), &constant).ok()?;
        let expression = poly_scale(&rest, &scale);
        let mut subst = SubstitutionMap::new();
        subst.insert(variable, expression);
        relations
            .into_iter()
            .filter(|relation| relation.id != source_relation_id)
            .filter_map(|relation| {
                let poly =
                    clear_denominators_primitive(&substitute_poly(&relation.polynomial, &subst));
                (!poly.terms.is_empty())
                    .then(|| relation_with_polynomial(relation.id, poly, relation.source))
            })
            .collect::<Vec<_>>()
    } else {
        let guard_exists = system
            .guards
            .iter()
            .any(|guard| guard.factor.hash == pivot.hash);
        if !guard_exists {
            return None;
        }
        let numerator = poly_scale(&rest, &int_q(-1));
        relations
            .into_iter()
            .filter(|relation| relation.id != source_relation_id)
            .filter_map(|relation| {
                let poly = clear_denominators_primitive(&substitute_rational_and_clear(
                    &relation.polynomial,
                    variable,
                    &numerator,
                    &pivot,
                ));
                (!poly.terms.is_empty())
                    .then(|| relation_with_polynomial(relation.id, poly, relation.source))
            })
            .collect::<Vec<_>>()
    };
    Some(transformed)
}

fn block_relations(block: &ProjectionBlock, system: &CompressedSystemQ) -> Vec<CanonicalRelationQ> {
    let ids = block
        .relation_ids
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    system
        .relations
        .iter()
        .filter(|relation| ids.contains(&relation.id))
        .cloned()
        .collect()
}

fn validate_linear_affine_plan_binding(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    if plan.block_id != ctx.block.block_id {
        return Err(implementation_bug("linear-affine block id mismatch"));
    }
    if plan.input_block_authorization_hash != ctx.block.authorization_hash {
        return Err(implementation_bug(
            "linear-affine block authorization hash mismatch",
        ));
    }
    if plan.source_relation_ids.len() != plan.source_relation_hashes.len() {
        return Err(implementation_bug(
            "linear-affine source relation identity arity mismatch",
        ));
    }
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
                "linear-affine planned source relation missing",
            ));
        };
        if relation.hash != *expected_hash {
            return Err(implementation_bug(
                "linear-affine source relation hash mismatch",
            ));
        }
    }
    Ok(())
}

fn constant_value(poly: &SparsePolynomialQ) -> Option<crate::types::rational::RationalQ> {
    if poly.terms.len() == 1 && poly.terms[0].monomial.exponents.is_empty() {
        Some(poly.terms[0].coeff.clone())
    } else {
        None
    }
}

fn constant_nonzero(poly: &SparsePolynomialQ) -> bool {
    constant_value(poly)
        .map(|value| !is_zero_q(&value))
        .unwrap_or(false)
}

fn finish_admission(
    block: &ProjectionBlock,
    status: KernelAdmissionStatus,
    execution_plan: Option<KernelExecutionPlan>,
) -> KernelAdmission {
    let mut chunks = vec![
        b"LinearAffine".to_vec(),
        block.block_id.0.to_be_bytes().to_vec(),
        format!("{status:?}").into_bytes(),
    ];
    if let Some(plan) = &execution_plan {
        chunks.push(plan.plan_hash.0.to_vec());
    }
    KernelAdmission {
        kind: KernelKind::LinearAffine,
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
    use crate::compose::message::{MessageRepresentation, ProjectionStrength};
    use crate::kernels::linear_affine::{
        execute_linear_affine, find_triangular_affine_order, plan_linear_affine, LinearAffineKernel,
    };
    use crate::kernels::traits::{KernelKind, TargetProjectionKernel};
    use crate::planner::kernel_plan::hash_kernel_execution_plan;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::result::status::SolverStatus;
    use crate::solver::options::SolverOptions;
    use crate::types::hash::hash_sequence;
    use crate::types::ids::{BlockId, VariableId};
    use crate::types::polynomial::{constant_poly, poly_mul, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    #[test]
    fn p7_linear_affine_exports_only_allowed_variables_with_safe_constant_pivot() {
        let t = VariableId(0);
        let x = VariableId(1);
        let relations = vec![
            poly_sub(&variable_poly(x), &variable_poly(t)),
            poly_sub(
                &poly_mul(&variable_poly(t), &variable_poly(t)),
                &constant_poly(int_q(2)),
            ),
        ];
        let compressed = compressed_system(vec![t, x], t, relations);
        let block = test_block(&compressed, [t, x], [t]);
        let order = find_triangular_affine_order(&block, &compressed).unwrap();
        assert_eq!(order.steps.len(), 1);
        let mut ctx = new_context(SolverOptions::default());
        let plan = plan_linear_affine(&block, &compressed, &order, &ctx).unwrap();
        let mut kctx = crate::kernels::traits::KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let message = execute_linear_affine(&plan, &mut kctx, &mut ctx).unwrap();
        assert_eq!(message.kernel_kind, KernelKind::LinearAffine);
        assert_eq!(message.representation, MessageRepresentation::GeneratorSet);
        assert_eq!(
            message.projection_strength,
            ProjectionStrength::CandidateCoverStrong
        );
        let exported = [t].into_iter().collect();
        assert!(message
            .relation_generators
            .iter()
            .all(|poly| crate::types::polynomial::poly_variables(poly).is_subset(&exported)));
        let kernel = LinearAffineKernel;
        assert!(kernel.replay(&message, &kctx).accepted);
        let mut tampered_message = message;
        tampered_message.package_hash = hash_sequence("tampered-package", &[]);
        assert!(!kernel.replay(&tampered_message, &kctx).accepted);
    }

    #[test]
    fn p7_linear_affine_rejects_unsafe_nonconstant_pivot() {
        let t = VariableId(0);
        let x = VariableId(1);
        let a = VariableId(2);
        let relations = vec![poly_sub(
            &poly_mul(&variable_poly(a), &variable_poly(x)),
            &variable_poly(t),
        )];
        let compressed = compressed_system(vec![t, x, a], t, relations);
        let block = test_block(&compressed, [t, x, a], [t]);
        assert!(find_triangular_affine_order(&block, &compressed).is_none());
    }

    #[test]
    fn p7_linear_affine_reports_stale_incomplete_plan_as_hard_case() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let relations = vec![
            poly_sub(&variable_poly(x), &variable_poly(t)),
            poly_sub(&variable_poly(y), &variable_poly(x)),
            poly_sub(
                &poly_mul(&variable_poly(t), &variable_poly(t)),
                &constant_poly(int_q(2)),
            ),
        ];
        let compressed = compressed_system(vec![t, x, y], t, relations);
        let block = test_block(&compressed, [t, x, y], [t]);
        let mut order = find_triangular_affine_order(&block, &compressed).unwrap();
        order.steps.pop();
        order.order_hash =
            crate::planner::kernel_plan::affine_elimination_plan(order.steps.clone()).order_hash;
        let mut ctx = new_context(SolverOptions::default());
        let plan = plan_linear_affine(&block, &compressed, &order, &ctx).unwrap();
        let mut kctx = crate::kernels::traits::KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let err = execute_linear_affine(&plan, &mut kctx, &mut ctx).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::AlgorithmicHardCase);
    }

    #[test]
    fn p7_linear_affine_rejects_auth_and_source_hash_tamper() {
        let t = VariableId(0);
        let x = VariableId(1);
        let relations = vec![
            poly_sub(&variable_poly(x), &variable_poly(t)),
            poly_sub(
                &poly_mul(&variable_poly(t), &variable_poly(t)),
                &constant_poly(int_q(2)),
            ),
        ];
        let compressed = compressed_system(vec![t, x], t, relations);
        let block = test_block(&compressed, [t, x], [t]);
        let order = find_triangular_affine_order(&block, &compressed).unwrap();
        let mut ctx = new_context(SolverOptions::default());
        let mut plan = plan_linear_affine(&block, &compressed, &order, &ctx).unwrap();
        let mut kctx = crate::kernels::traits::KernelContext {
            block: block.clone(),
            system: compressed.clone(),
            child_messages: Vec::new(),
        };

        let mut bad_auth_ctx = kctx.clone();
        bad_auth_ctx.block.authorization_hash = hash_sequence("tampered-auth", &[]);
        let err = execute_linear_affine(&plan, &mut bad_auth_ctx, &mut ctx).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);

        plan.source_relation_hashes[0] = hash_sequence("tampered-source", &[]);
        plan.plan_hash = hash_kernel_execution_plan(&plan);
        let err = execute_linear_affine(&plan, &mut kctx, &mut ctx).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    fn compressed_system(
        variables: Vec<VariableId>,
        target: VariableId,
        relations: Vec<crate::types::polynomial::SparsePolynomialQ>,
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
