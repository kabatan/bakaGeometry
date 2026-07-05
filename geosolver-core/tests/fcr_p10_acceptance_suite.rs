use std::collections::BTreeSet;

use geosolver_core::api::solve_target;
use geosolver_core::compose::compose::compose_projection_messages;
use geosolver_core::compose::message::{
    hash_projection_message, MessageRepresentation, ProjectionMessage, ProjectionStrength,
};
use geosolver_core::graph::hypergraph::build_relation_variable_hypergraph;
use geosolver_core::graph::influence::build_target_influence_graph;
use geosolver_core::graph::projection_dag::{build_target_projection_dag, TargetProjectionDAG};
use geosolver_core::graph::separators::CostModel;
use geosolver_core::graph::tree_decomposition::build_target_rooted_decomposition;
use geosolver_core::graph::weighted_primal::build_weighted_primal_graph;
use geosolver_core::kernels::traits::KernelKind;
use geosolver_core::preprocess::compression::pre_kernel_compress;
use geosolver_core::problem::canonicalize::canonicalize_system;
use geosolver_core::problem::context::new_context;
use geosolver_core::problem::input::{make_problem, RationalTargetProblem};
use geosolver_core::problem::validate::validate_input;
use geosolver_core::result::cost_trace::ProjectionCostTrace;
use geosolver_core::result::output::TargetSolveResult;
use geosolver_core::result::status::SolverStatus;
use geosolver_core::roots::decode::hash_target_candidate;
use geosolver_core::solver::options::SolverOptions;
use geosolver_core::types::hash::Hash;
use geosolver_core::types::ids::{BlockId, PackageId, RelationId, VariableId};
use geosolver_core::types::polynomial::{
    constant_poly, poly_add, poly_mul, poly_scale, poly_sub, poly_variables, variable_poly,
    SparsePolynomialQ,
};
use geosolver_core::types::rational::{div_q, int_q, RationalQ};
use geosolver_core::verify::certificates::{
    kernel_certificate_binding_hash, KernelCertificate, KernelCertificatePayload,
};
use geosolver_core::verify::replay_run_certificate;

fn public_candidate_cover(problem: RationalTargetProblem) -> TargetSolveResult {
    let result = solve_target(problem.clone(), SolverOptions::default());
    assert_candidate_cover(&result, &problem);
    result
}

fn public_candidate_cover_with_options(
    problem: RationalTargetProblem,
    options: SolverOptions,
) -> TargetSolveResult {
    let result = solve_target(problem.clone(), options);
    assert_candidate_cover(&result, &problem);
    result
}

fn assert_candidate_cover(result: &TargetSolveResult, problem: &RationalTargetProblem) {
    assert_eq!(
        result.status,
        SolverStatus::CertifiedCandidateCover,
        "unexpected status diagnostics={:?}",
        result.diagnostics
    );
    let support = result.support_polynomial.as_ref().expect("support");
    assert!(
        support.coeffs_low_to_high.len() > 1,
        "support must be nonconstant: {support:?}"
    );
    assert!(result.squarefree_support_polynomial.is_some());
    assert!(!result.projection_messages.is_empty());
    assert!(result.certificate.is_some());
    assert_eq!(result.root_isolation.len(), result.decoded_candidates.len());
    let squarefree_hash = result.squarefree_support_polynomial.as_ref().unwrap().hash;
    for (root, candidate) in result.root_isolation.iter().zip(&result.decoded_candidates) {
        assert_eq!(root.support_hash, squarefree_hash);
        assert_eq!(candidate.target, problem.target);
        assert_eq!(candidate.support_hash, squarefree_hash);
        assert_eq!(candidate.root_index, root.root_index);
        assert_eq!(candidate.isolating_interval, root.isolating_interval);
        assert_ne!(candidate.candidate_hash, Hash([0; 32]));
        assert_eq!(
            candidate.candidate_hash,
            hash_target_candidate(
                problem.target,
                squarefree_hash,
                root.root_index,
                &root.isolating_interval
            )
        );
    }
    let replay = replay_run_certificate(result, problem);
    assert!(
        replay.accepted,
        "replay failed; status={:?}; kernels={:?}; support={:?}; certificate={:?}",
        result.status,
        result
            .projection_messages
            .iter()
            .map(|message| message.kernel_kind)
            .collect::<Vec<_>>(),
        result.support_polynomial,
        result.certificate
    );
    assert_eq!(
        result.cost_trace.block_traces.len(),
        result.projection_messages.len()
    );
    assert!(result.cost_trace.verification_trace.checked_relation_count > 0);
}

fn assert_executed_kernel(result: &TargetSolveResult, kind: KernelKind) {
    assert!(
        result
            .projection_messages
            .iter()
            .any(|message| message.kernel_kind == kind),
        "expected {kind:?}; executed kernels={:?}; diagnostics={:?}",
        result
            .projection_messages
            .iter()
            .map(|message| message.kernel_kind)
            .collect::<Vec<_>>(),
        result.diagnostics
    );
}

fn options_prioritizing(kind: KernelKind) -> SolverOptions {
    let mut options = SolverOptions::default();
    options.kernel_priority = vec![kind];
    options
}

fn problem(
    variables: Vec<VariableId>,
    target: VariableId,
    relations: Vec<SparsePolynomialQ>,
) -> RationalTargetProblem {
    make_problem(variables, target, scale_relations(relations), Vec::new())
}

fn v(id: u32) -> SparsePolynomialQ {
    variable_poly(VariableId(id))
}

fn c(n: i64) -> SparsePolynomialQ {
    constant_poly(int_q(n))
}

fn q(num: i64, den: i64) -> RationalQ {
    div_q(&int_q(num), &int_q(den)).expect("nonzero scale denominator")
}

fn scale_relations(relations: Vec<SparsePolynomialQ>) -> Vec<SparsePolynomialQ> {
    relations
        .into_iter()
        .enumerate()
        .map(|(index, relation)| {
            let factor = match index % 5 {
                0 => q(2, 3),
                1 => q(-5, 2),
                2 => q(7, 4),
                3 => q(-11, 5),
                _ => q(13, 6),
            };
            let scaled = poly_scale(&relation, &factor);
            assert!(
                relation.terms.is_empty() || scaled != relation,
                "P10 anti-hack scaling must change every nonzero input relation"
            );
            scaled
        })
        .collect()
}

fn dag_for_problem(problem: &RationalTargetProblem) -> TargetProjectionDAG {
    let canonical = canonicalize_system(validate_input(problem.clone()).unwrap()).unwrap();
    let mut ctx = new_context(SolverOptions::default());
    let compressed = pre_kernel_compress(canonical, &mut ctx).unwrap();
    let hypergraph = build_relation_variable_hypergraph(&compressed);
    let influence = build_target_influence_graph(&hypergraph, problem.target);
    let weighted = build_weighted_primal_graph(&compressed, &influence);
    let decomposition =
        build_target_rooted_decomposition(&weighted, problem.target, &CostModel::default());
    build_target_projection_dag(&compressed, &influence, &decomposition).unwrap()
}

fn assert_message_composition_is_essential(
    problem: &RationalTargetProblem,
    result: &TargetSolveResult,
) {
    assert!(
        result.projection_messages.len() >= 2,
        "composition stress requires multiple projection messages: {:?}",
        result.projection_messages
    );
    let target_only = BTreeSet::from([problem.target]);
    assert!(
        result.projection_messages.iter().any(|message| message
            .relation_generators
            .iter()
            .any(|relation| !poly_variables(relation).is_subset(&target_only))),
        "composition stress must include separator-bearing message relations"
    );
    let dag = dag_for_problem(problem);
    let mut full_ctx = new_context(SolverOptions::default());
    let full = compose_projection_messages(
        &dag,
        result.projection_messages.clone(),
        problem.target,
        &mut full_ctx,
    )
    .expect("full message composition must succeed");
    assert!(
        !full.separator_elimination_messages.is_empty(),
        "composition must use message-only separator elimination: {:?}",
        full
    );

    for removed_index in 0..result.projection_messages.len() {
        let mut reduced_messages = result.projection_messages.clone();
        reduced_messages.remove(removed_index);
        let mut reduced_ctx = new_context(SolverOptions::default());
        let reduced =
            compose_projection_messages(&dag, reduced_messages, problem.target, &mut reduced_ctx);
        assert!(
            reduced
                .as_ref()
                .map_or(true, |projection| projection.root_relations != full.root_relations),
            "removing projection message {removed_index} must fail or change composed target support"
        );
    }
}

fn assert_multiseparator_message_only_composition() {
    let t = VariableId(71);
    let u = VariableId(72);
    let w = VariableId(73);
    let messages = vec![
        synthetic_message(
            BlockId(1),
            PackageId(101),
            vec![t, u],
            poly_sub(&v(t.0), &v(u.0)),
        ),
        synthetic_message(
            BlockId(2),
            PackageId(102),
            vec![u, w],
            poly_sub(&v(u.0), &v(w.0)),
        ),
        synthetic_message(
            BlockId(3),
            PackageId(103),
            vec![w],
            poly_sub(&poly_mul(&v(w.0), &v(w.0)), &c(2)),
        ),
    ];
    let dag = synthetic_composition_dag(t, u, w);
    let mut ctx = new_context(SolverOptions::default());
    let composed = compose_projection_messages(&dag, messages.clone(), t, &mut ctx)
        .expect("multiseparator message-only composition");
    let separators = composed
        .message_relations
        .iter()
        .flat_map(poly_variables)
        .filter(|var| *var != t)
        .collect::<BTreeSet<_>>();
    assert_eq!(separators, BTreeSet::from([u, w]));
    assert_eq!(composed.separator_elimination_messages.len(), 1);
    assert!(
        composed
            .separator_elimination_messages
            .iter()
            .all(|message| message.kernel_kind == KernelKind::TargetRelationSearch),
        "separator elimination must use production target relation search"
    );

    for removed_index in 0..messages.len() {
        let mut reduced = messages.clone();
        reduced.remove(removed_index);
        let mut reduced_ctx = new_context(SolverOptions::default());
        let reduced_composition = compose_projection_messages(&dag, reduced, t, &mut reduced_ctx);
        assert!(
            reduced_composition
                .as_ref()
                .map_or(true, |projection| projection.root_relations
                    != composed.root_relations),
            "removing multiseparator message {removed_index} must fail or change target support"
        );
    }
}

fn synthetic_message(
    block_id: BlockId,
    package_id: PackageId,
    exported_variables: Vec<VariableId>,
    relation: SparsePolynomialQ,
) -> ProjectionMessage {
    let mut cert = KernelCertificate {
        certificate_hash: hash_projection_message_seed(package_id),
        certificate_route:
            geosolver_core::planner::kernel_plan::CertificateRoute::SourceMembershipCertificate,
        plan_hash: hash_projection_message_seed(PackageId(package_id.0 + 1000)),
        source_relation_hashes: vec![relation.hash],
        output_relation_hashes: vec![relation.hash],
        exported_variables: exported_variables.clone(),
        binding_hash: Hash([0; 32]),
        payload: KernelCertificatePayload::BindingOnly,
    };
    cert.binding_hash = kernel_certificate_binding_hash(&cert);
    let mut message = ProjectionMessage {
        package_id,
        block_id,
        kernel_kind: KernelKind::TargetRelationSearch,
        source_relation_ids: vec![RelationId(package_id.0)],
        eliminated_variables: Vec::new(),
        exported_variables,
        relation_generators: vec![relation],
        representation: MessageRepresentation::GeneratorSet,
        projection_strength: ProjectionStrength::CandidateCoverStrong,
        certificate: cert,
        compression_trace: Default::default(),
        cost_trace: ProjectionCostTrace::default(),
        package_hash: Hash([0; 32]),
    };
    message.package_hash = hash_projection_message(&message);
    message
}

fn hash_projection_message_seed(package_id: PackageId) -> Hash {
    geosolver_core::types::hash::hash_sequence(
        "fcr-p10-synthetic-composition",
        &[package_id.0.to_be_bytes().to_vec()],
    )
}

fn synthetic_composition_dag(t: VariableId, u: VariableId, w: VariableId) -> TargetProjectionDAG {
    TargetProjectionDAG {
        blocks: vec![
            synthetic_block(
                BlockId(0),
                None,
                vec![t, u, w],
                vec![t],
                vec![BlockId(1), BlockId(2), BlockId(3)],
            ),
            synthetic_block(
                BlockId(1),
                Some(BlockId(0)),
                vec![t, u],
                vec![t, u],
                Vec::new(),
            ),
            synthetic_block(
                BlockId(2),
                Some(BlockId(0)),
                vec![u, w],
                vec![u, w],
                Vec::new(),
            ),
            synthetic_block(BlockId(3), Some(BlockId(0)), vec![w], vec![w], Vec::new()),
        ],
        root_block_id: BlockId(0),
        dag_hash: hash_projection_message_seed(PackageId(900)),
    }
}

fn synthetic_block(
    block_id: BlockId,
    parent_block_id: Option<BlockId>,
    local_variables: Vec<VariableId>,
    exported_variables: Vec<VariableId>,
    child_block_ids: Vec<BlockId>,
) -> geosolver_core::graph::projection_dag::ProjectionBlock {
    geosolver_core::graph::projection_dag::ProjectionBlock {
        block_id,
        local_variables: local_variables.into_iter().collect(),
        relation_ids: Vec::new(),
        exported_variables: exported_variables.into_iter().collect(),
        child_block_ids,
        parent_block_id,
        authorization_hash: hash_projection_message_seed(PackageId(block_id.0)),
        duplication_certificates: Vec::new(),
        block_hash: hash_projection_message_seed(PackageId(block_id.0 + 10)),
    }
}

#[test]
fn fcr_p10_a1_public_no_initial_target_only_one_block() {
    let t = VariableId(7);
    let x = VariableId(2);
    let y = VariableId(11);
    let result = public_candidate_cover(problem(
        vec![x, t, y],
        t,
        vec![
            poly_sub(&v(y.0), &v(x.0)),
            poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(2)),
            poly_sub(&v(t.0), &v(y.0)),
        ],
    ));
    assert!(!result.root_isolation.is_empty());
}

#[test]
fn fcr_p10_a2_public_multivariate_quotient_nonlinear_target() {
    let t = VariableId(13);
    let x = VariableId(4);
    let y = VariableId(9);
    let xy = poly_mul(&v(x.0), &v(y.0));
    let mut options = options_prioritizing(KernelKind::TargetActionKrylov);
    options.max_relation_search_export_degree = Some(0);
    let result = public_candidate_cover_with_options(
        problem(
            vec![y, t, x],
            t,
            vec![
                poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(2)),
                poly_sub(&poly_mul(&v(y.0), &v(y.0)), &c(3)),
                poly_sub(&poly_sub(&v(t.0), &xy), &v(x.0)),
            ],
        ),
        options,
    );
    assert!(!result.root_isolation.is_empty());
    assert_executed_kernel(&result, KernelKind::TargetActionKrylov);
}

#[test]
fn fcr_p10_a3_public_multiple_eliminated_variables_and_separators() {
    let t = VariableId(17);
    let x = VariableId(3);
    let u = VariableId(10);
    let v0 = VariableId(6);
    let y = VariableId(14);
    let x2 = poly_mul(&v(x.0), &v(x.0));
    let u2 = poly_mul(&v(u.0), &v(u.0));
    let v2 = poly_mul(&v(v0.0), &v(v0.0));
    let y2 = poly_mul(&v(y.0), &v(y.0));
    let y3 = poly_mul(&y2, &v(y.0));
    let input = problem(
        vec![x, t, y, u, v0],
        t,
        vec![
            poly_sub(&u2, &v(t.0)),
            poly_sub(&v2, &u2),
            poly_sub(&x2, &v2),
            poly_sub(&y2, &x2),
            poly_sub(&y3, &c(2)),
        ],
    );
    let result = public_candidate_cover(input.clone());
    assert!(
        result
            .projection_messages
            .iter()
            .any(|message| message.eliminated_variables.len() >= 2),
        "projection_messages={:?}",
        result.projection_messages
    );
    assert_message_composition_is_essential(&input, &result);
    assert_multiseparator_message_only_composition();
}

#[test]
fn fcr_p10_a4_public_sparse_resultant_eliminant_without_target_only_input() {
    let t = VariableId(19);
    let x = VariableId(5);
    let x2 = poly_mul(&v(x.0), &v(x.0));
    let x3 = poly_mul(&x2, &v(x.0));
    let result = public_candidate_cover_with_options(
        problem(
            vec![x, t],
            t,
            vec![poly_sub(&x2, &v(t.0)), poly_sub(&x3, &c(2))],
        ),
        options_prioritizing(KernelKind::SparseResultantProjection),
    );
    assert!(!result.root_isolation.is_empty());
    assert_executed_kernel(&result, KernelKind::SparseResultantProjection);
}

#[test]
fn fcr_p10_a5_public_specialization_interpolation_style_multiseparator() {
    let t = VariableId(23);
    let u = VariableId(8);
    let w = VariableId(15);
    let x = VariableId(1);
    let x2 = poly_mul(&v(x.0), &v(x.0));
    let u2 = poly_mul(&v(u.0), &v(u.0));
    let input = problem(
        vec![w, x, t, u],
        t,
        vec![
            poly_sub(&x2, &poly_add(&v(t.0), &u2)),
            poly_sub(&x2, &c(1)),
            poly_sub(&poly_mul(&v(w.0), &v(w.0)), &c(2)),
            poly_sub(&poly_mul(&v(u.0), &v(w.0)), &c(1)),
        ],
    );
    let result = public_candidate_cover_with_options(
        input.clone(),
        options_prioritizing(KernelKind::SpecializationInterpolation),
    );
    assert!(!result.root_isolation.is_empty());
    assert_executed_kernel(&result, KernelKind::SpecializationInterpolation);
    assert_message_composition_is_essential(&input, &result);
}

#[test]
fn fcr_p10_a6_public_guarded_rational_affine_preprocessing_to_support() {
    let t = VariableId(29);
    let x = VariableId(12);
    let y = VariableId(20);
    let s = VariableId(2);
    let denominator = poly_add(&v(x.0), &c(1));
    let witness = poly_sub(&poly_mul(&denominator, &v(s.0)), &c(1));
    let affine = poly_sub(
        &poly_mul(&denominator, &v(y.0)),
        &poly_add(&v(t.0), &v(x.0)),
    );
    let result = public_candidate_cover(problem(
        vec![s, t, y, x],
        t,
        vec![
            witness,
            affine,
            poly_sub(&poly_mul(&v(y.0), &v(y.0)), &c(2)),
            poly_sub(&v(x.0), &c(1)),
        ],
    ));
    assert!(!result.root_isolation.is_empty());
}

#[test]
fn fcr_p10_a7_public_target_independent_component_with_feasibility_obligation() {
    let t = VariableId(31);
    let z = VariableId(18);
    let result = public_candidate_cover(problem(
        vec![z, t],
        t,
        vec![
            poly_sub(&v(t.0), &c(5)),
            poly_sub(&poly_mul(&v(z.0), &v(z.0)), &c(1)),
        ],
    ));
    assert_eq!(result.root_isolation.len(), 1);
}

#[test]
fn fcr_p10_a8_public_one_large_block_no_useful_separator() {
    let t = VariableId(37);
    let x = VariableId(21);
    let y = VariableId(24);
    let mut options = options_prioritizing(KernelKind::UniversalTargetElimination);
    options.max_relation_search_export_degree = Some(0);
    let result = public_candidate_cover_with_options(
        problem(
            vec![y, t, x],
            t,
            vec![
                poly_sub(&poly_add(&poly_mul(&v(x.0), &v(x.0)), &v(y.0)), &c(1)),
                poly_sub(&poly_sub(&poly_mul(&v(y.0), &v(y.0)), &v(x.0)), &c(0)),
                poly_sub(&poly_sub(&v(t.0), &v(x.0)), &v(y.0)),
            ],
        ),
        options,
    );
    assert!(!result.root_isolation.is_empty());
    assert_executed_kernel(&result, KernelKind::UniversalTargetElimination);
}

#[test]
fn fcr_p10_a9_public_regular_chain_style_projection() {
    let t = VariableId(41);
    let y = VariableId(27);
    let result = public_candidate_cover_with_options(
        problem(
            vec![y, t],
            t,
            vec![
                poly_sub(&v(y.0), &v(t.0)),
                poly_sub(&poly_mul(&v(y.0), &v(y.0)), &c(2)),
            ],
        ),
        options_prioritizing(KernelKind::RegularChainProjection),
    );
    assert_eq!(result.root_isolation.len(), 2);
    assert_executed_kernel(&result, KernelKind::RegularChainProjection);
}

#[test]
fn fcr_p10_a10_public_norm_trace_two_step_tower() {
    let t = VariableId(43);
    let a = VariableId(28);
    let b = VariableId(33);
    let result = public_candidate_cover_with_options(
        problem(
            vec![b, t, a],
            t,
            vec![
                poly_sub(&poly_mul(&v(a.0), &v(a.0)), &c(2)),
                poly_sub(&poly_mul(&v(b.0), &v(b.0)), &v(a.0)),
                poly_sub(&poly_mul(&v(t.0), &v(b.0)), &c(1)),
            ],
        ),
        options_prioritizing(KernelKind::NormTraceProjection),
    );
    assert!(!result.root_isolation.is_empty());
    assert_executed_kernel(&result, KernelKind::NormTraceProjection);
}

#[test]
fn fcr_p10_a11_public_nonreal_support_empty_candidate_cover() {
    let t = VariableId(47);
    let result = public_candidate_cover(problem(
        vec![t],
        t,
        vec![poly_add(&poly_mul(&v(t.0), &v(t.0)), &c(1))],
    ));
    assert!(result.root_isolation.is_empty());
    assert!(result.decoded_candidates.is_empty());
}

#[test]
fn fcr_p10_a12_public_certified_nonfinite_requires_positive_proof() {
    let t = VariableId(53);
    let x = VariableId(35);
    let result = solve_target(
        problem(vec![x, t], t, vec![poly_sub(&v(x.0), &c(1))]),
        SolverOptions::default(),
    );
    assert_eq!(result.status, SolverStatus::CertifiedNonFiniteTargetImage);
    assert!(result.support_polynomial.is_none());
    assert!(result.projection_messages.is_empty() || result.certificate.is_none());
}

#[test]
fn fcr_p10_a13_public_resource_bounded_hard_case_has_spec_status() {
    let t = VariableId(59);
    let x = VariableId(39);
    let mut options = options_prioritizing(KernelKind::TargetRelationSearch);
    options.max_relation_search_export_degree = Some(2);
    options.max_matrix_rows = Some(1);
    options.max_matrix_cols = Some(1);
    let result = solve_target(
        problem(
            vec![x, t],
            t,
            vec![poly_sub(&poly_mul(&v(x.0), &v(x.0)), &v(t.0))],
        ),
        options,
    );
    assert!(matches!(
        result.status,
        SolverStatus::AlgorithmicHardCase
            | SolverStatus::FiniteResourceFailure
            | SolverStatus::CertificateDesignGap
    ));
    assert_eq!(result.target, t);
    assert!(
        !result.cost_trace.block_traces.is_empty(),
        "resource/hard-case result must retain a cost trace: {:?}",
        result.cost_trace
    );
    assert!(
        result
            .cost_trace
            .block_traces
            .iter()
            .any(|trace| trace.matrix_rows.is_some() || trace.matrix_cols.is_some()),
        "bounded failure cost trace must identify matrix dimensions: {:?}",
        result.cost_trace
    );
    assert!(
        result
            .cost_trace
            .block_traces
            .iter()
            .any(|trace| trace.kernel_kind == KernelKind::TargetRelationSearch),
        "bounded TargetRelationSearch failure must retain kernel identity: {:?}",
        result.cost_trace
    );
}
