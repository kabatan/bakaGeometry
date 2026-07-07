#!/usr/bin/env python3
"""Static conformance audit for the scoped RGDTPK-Q v4 candidate-cover repair.

This script is intentionally conservative. It is not an acceptance proof; it only
makes missing files, forbidden production markers, and obvious scope violations
visible for Guardian phase review.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src"
TESTS = ROOT / "tests"


REQUIRED_FILES = [
    "lib.rs",
    "api.rs",
    "types/mod.rs",
    "types/ids.rs",
    "types/rational.rs",
    "types/monomial.rs",
    "types/polynomial.rs",
    "types/univariate.rs",
    "types/matrix.rs",
    "types/interval.rs",
    "types/hash.rs",
    "problem/mod.rs",
    "problem/input.rs",
    "problem/semantic.rs",
    "problem/validate.rs",
    "problem/canonicalize.rs",
    "problem/context.rs",
    "algebra/mod.rs",
    "algebra/monomial_order.rs",
    "algebra/polynomial_ops.rs",
    "algebra/modular.rs",
    "algebra/crt.rs",
    "algebra/rational_reconstruction.rs",
    "algebra/sparse_matrix.rs",
    "algebra/dense_matrix.rs",
    "algebra/linear_solve.rs",
    "algebra/normal_form.rs",
    "algebra/groebner.rs",
    "algebra/f4.rs",
    "algebra/elimination.rs",
    "algebra/resultant.rs",
    "algebra/interpolation.rs",
    "algebra/quotient.rs",
    "algebra/krylov.rs",
    "algebra/regular_chain.rs",
    "algebra/norm_trace.rs",
    "algebra/real_root.rs",
    "algebra/sign.rs",
    "preprocess/mod.rs",
    "preprocess/compression.rs",
    "preprocess/linear_affine.rs",
    "preprocess/definitional.rs",
    "preprocess/binomial.rs",
    "preprocess/saturation.rs",
    "preprocess/independent.rs",
    "graph/mod.rs",
    "graph/hypergraph.rs",
    "graph/influence.rs",
    "graph/weighted_primal.rs",
    "graph/separators.rs",
    "graph/tree_decomposition.rs",
    "graph/projection_dag.rs",
    "graph/metrics.rs",
    "planner/mod.rs",
    "planner/cost_model.rs",
    "planner/probes.rs",
    "planner/admission.rs",
    "planner/kernel_plan.rs",
    "planner/ladder.rs",
    "planner/planner.rs",
    "kernels/mod.rs",
    "kernels/traits.rs",
    "kernels/target_univariate.rs",
    "kernels/linear_affine.rs",
    "kernels/target_relation_search.rs",
    "kernels/sparse_resultant.rs",
    "kernels/action_krylov.rs",
    "kernels/universal_elimination.rs",
    "kernels/regular_chain_projection.rs",
    "kernels/norm_trace_projection.rs",
    "kernels/specialization_interpolation.rs",
    "compose/mod.rs",
    "compose/message.rs",
    "compose/compose.rs",
    "compose/separator_elimination.rs",
    "compose/final_support.rs",
    "verify/mod.rs",
    "verify/certificates.rs",
    "verify/verify_message.rs",
    "verify/verify_support.rs",
    "verify/replay.rs",
    "verify/run_certificate.rs",
    "roots/mod.rs",
    "roots/squarefree.rs",
    "roots/isolate.rs",
    "roots/decode.rs",
    "roots/algebraic_number.rs",
    "result/mod.rs",
    "result/status.rs",
    "result/diagnostics.rs",
    "result/cost_trace.rs",
    "result/output.rs",
    "solver/mod.rs",
    "solver/options.rs",
    "solver/pipeline.rs",
    "solver/orchestrator.rs",
]


REQUIRED_SYMBOLS = {
    "api.rs": ["solve_target"],
    "problem/validate.rs": ["validate_input"],
    "problem/canonicalize.rs": ["canonicalize_system"],
    "preprocess/compression.rs": ["compress_system"],
    "graph/projection_dag.rs": ["build_target_projection_dag", "validate_projection_dag"],
    "planner/planner.rs": ["plan_projection_messages"],
    "kernels/mod.rs": ["all_kernels"],
    "kernels/target_univariate.rs": ["TargetUnivariateKernel"],
    "kernels/linear_affine.rs": ["LinearAffineKernel"],
    "kernels/target_relation_search.rs": ["TargetRelationSearchKernel"],
    "kernels/sparse_resultant.rs": ["SparseResultantProjectionKernel"],
    "kernels/action_krylov.rs": ["TargetActionKrylovKernel"],
    "kernels/universal_elimination.rs": ["UniversalTargetEliminationKernel"],
    "kernels/regular_chain_projection.rs": ["RegularChainProjectionKernel"],
    "kernels/norm_trace_projection.rs": ["NormTraceProjectionKernel"],
    "kernels/specialization_interpolation.rs": ["SpecializationInterpolationKernel"],
    "compose/compose.rs": ["compose_projection_messages"],
    "compose/final_support.rs": ["build_global_support_polynomial"],
    "verify/verify_message.rs": ["verify_projection_message"],
    "verify/verify_support.rs": ["verify_global_support"],
    "verify/replay.rs": ["replay_run_certificate"],
    "roots/squarefree.rs": ["squarefree_support"],
    "roots/isolate.rs": ["isolate_real_roots"],
    "roots/decode.rs": ["decode_candidates"],
    "solver/pipeline.rs": ["run_pipeline"],
    "solver/orchestrator.rs": ["solve_target"],
}


FORBIDDEN_PATTERNS = [
    ("todo_macro", re.compile(r"\btodo!\s*\(")),
    ("unimplemented_macro", re.compile(r"\bunimplemented!\s*\(")),
    ("todo_panic", re.compile(r'panic!\s*\(\s*"TODO')),
    ("unsupported_status_or_diagnostic", re.compile(r"\bUnsupported\b")),
    ("kernel_not_ready_error", re.compile(r"\bkernel_not_ready_error\b")),
    ("qe_cad_fallback", re.compile(r"\b(QE|CAD)\b")),
    ("fixture_dispatch", re.compile(r"\b(fixture|expected_answer|problem_id)\b", re.IGNORECASE)),
]


GEOMETRY_DISPATCH_PATTERN = re.compile(
    r"\b(geometry_name|problem_name|semantic_label|dsl_label)\b", re.IGNORECASE
)
COORDINATE_SOLVER_PATTERN = re.compile(
    r"\b(full_coordinate|coordinate_solution|coordinate_roots|global_lex|rur)\b",
    re.IGNORECASE,
)
EXACT_IMAGE_SUCCESS_PATTERN = re.compile(
    r"\b(CertifiedExactTargetImage|CertifiedEmptyRealTargetImage)\b"
)


def is_allowed_no_coordinate_export_marker(line: str) -> bool:
    lower = line.lower()
    return (
        "no_coordinate" in line
        or "must not expose coordinate roots" in line
        or "coordinate-exporting quotient handle" in line
        or "coordinate root or rur export" in lower
        or "coordinate roots or rur" in lower
        or "no coordinate" in lower
    )


def is_allowed_exact_image_reference(phase: str | None, rel_path: str, line: str) -> bool:
    return False


@dataclass
class Finding:
    severity: str
    code: str
    path: str
    line: int
    message: str


def production_rust_files() -> list[Path]:
    return sorted(p for p in SRC.rglob("*.rs") if p.is_file())


def all_test_files() -> list[Path]:
    return sorted(TESTS.rglob("*.rs")) if TESTS.exists() else []


def rel(path: Path) -> str:
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def lines(path: Path) -> list[str]:
    try:
        return path.read_text(encoding="utf-8").splitlines()
    except UnicodeDecodeError:
        return path.read_text().splitlines()


def strip_cfg_test_blocks(text: str) -> str:
    out: list[str] = []
    skip_depth: int | None = None
    pending_cfg = False
    for line in text.splitlines():
        if "#[cfg(test)]" in line or "#[cfg(any(test" in line:
            pending_cfg = True
            continue
        if pending_cfg:
            if "mod tests" in line or line.lstrip().startswith("mod "):
                skip_depth = line.count("{") - line.count("}")
                pending_cfg = False
                if skip_depth <= 0:
                    skip_depth = 1
                continue
            pending_cfg = False
        if skip_depth is not None:
            skip_depth += line.count("{") - line.count("}")
            if skip_depth <= 0:
                skip_depth = None
            continue
        out.append(line)
    return "\n".join(out)


def symbol_regex(symbol: str) -> re.Pattern[str]:
    if symbol.endswith("Kernel"):
        return re.compile(rf"\b(struct|enum|trait|impl)\s+{re.escape(symbol)}\b|\b{re.escape(symbol)}\b")
    return re.compile(
        rf"\b(struct|enum|trait|type|fn|pub\s+fn)\s+{re.escape(symbol)}\b|\b{re.escape(symbol)}\s*\(|\b{re.escape(symbol)}\b"
    )


P1_FILES = [
    "lib.rs",
    "api.rs",
    "types/ids.rs",
    "types/rational.rs",
    "types/monomial.rs",
    "types/polynomial.rs",
    "types/univariate.rs",
    "types/matrix.rs",
    "types/interval.rs",
    "types/hash.rs",
    "result/status.rs",
    "result/diagnostics.rs",
    "result/cost_trace.rs",
    "result/output.rs",
]


P1_SYMBOLS = {
    "api.rs": ["solve_target"],
    "types/ids.rs": [
        "IdCounter",
        "fresh_variable_id",
        "fresh_relation_id",
        "fresh_block_id",
        "fresh_package_id",
        "fresh_kernel_plan_id",
        "stable_id_from_name",
    ],
    "types/rational.rs": ["normalize_q", "div_q"],
    "types/polynomial.rs": ["normalize_poly", "clear_denominators_primitive", "substitute_poly"],
    "types/univariate.rs": ["degree_uni", "gcd_uni", "squarefree_part_uni", "eval_uni_q"],
    "types/matrix.rs": ["matrix_shape", "matrix_density", "hash_matrix"],
    "types/interval.rs": ["interval_new", "interval_contains_q", "interval_disjoint"],
}


P2_FILES = [
    "problem/input.rs",
    "problem/semantic.rs",
    "problem/validate.rs",
    "problem/canonicalize.rs",
    "problem/context.rs",
    "preprocess/compression.rs",
]


P2_SYMBOLS = {
    "problem/input.rs": [
        "RationalTargetProblem",
        "VariableRoleRecord",
        "make_problem",
        "make_problem_with_roles",
        "hash_problem_input",
    ],
    "problem/semantic.rs": [
        "RealConstraintKind",
        "RealConstraintEncoding",
        "register_slack_encoding",
        "semantic_relations",
        "verify_semantic_references",
    ],
    "problem/validate.rs": ["ValidatedProblem", "validate_input"],
    "problem/canonicalize.rs": [
        "CanonicalSystemQ",
        "canonicalize_system",
        "canonicalize_relation",
        "canonical_variable_order",
    ],
    "problem/context.rs": [
        "SolverContext",
        "ResourceMeter",
        "ActiveRouteBudget",
        "new_context",
        "begin_route_budget",
        "check_resource",
        "check_resource_work",
        "push_diagnostic",
    ],
    "preprocess/compression.rs": ["compress_system"],
}


P3_FILES = [
    "algebra/monomial_order.rs",
    "algebra/polynomial_ops.rs",
    "algebra/modular.rs",
    "algebra/crt.rs",
    "algebra/rational_reconstruction.rs",
    "algebra/sparse_matrix.rs",
    "algebra/dense_matrix.rs",
    "algebra/linear_solve.rs",
    "algebra/normal_form.rs",
    "types/matrix.rs",
]


P3_SYMBOLS = {
    "algebra/monomial_order.rs": [
        "MonomialOrder",
        "elimination_order",
        "grevlex_order",
        "lex_order",
        "block_order",
    ],
    "algebra/polynomial_ops.rs": [
        "leading_term",
        "s_polynomial",
        "reduce_by_set",
        "content_primitive_part",
    ],
    "algebra/modular.rs": [
        "choose_prime_avoiding_denominators",
        "reduce_q_to_fp",
        "reduce_rational_coeff",
        "next_prime_at_or_after",
    ],
    "algebra/crt.rs": [
        "try_crt_combine",
        "try_crt_vector_combine",
        "crt_combine",
        "crt_vector_combine",
    ],
    "algebra/rational_reconstruction.rs": [
        "reconstruct_rational",
        "reconstruct_polynomial",
    ],
    "algebra/sparse_matrix.rs": [
        "row_echelon_sparse_fp",
        "nullspace_sparse_fp",
        "rank_sparse_fp",
    ],
    "algebra/dense_matrix.rs": [
        "row_echelon_dense_fp",
        "nullspace_dense_fp",
        "rank_dense_fp",
    ],
    "algebra/linear_solve.rs": [
        "solve_homogeneous_modular",
        "solve_inhomogeneous_modular",
        "ModularProofStatus",
    ],
    "algebra/normal_form.rs": [
        "normal_form",
        "verify_membership_by_certificate",
        "MembershipCertificate",
    ],
    "types/matrix.rs": ["matrix_shape", "matrix_density", "hash_matrix"],
}


P4_FILES = [
    "algebra/groebner.rs",
    "algebra/f4.rs",
    "algebra/elimination.rs",
    "algebra/normal_form.rs",
    "result/cost_trace.rs",
]


P4_SYMBOLS = {
    "algebra/groebner.rs": [
        "groebner_elimination_basis",
        "reduce_with_certified_basis",
        "extract_certified_elimination_generators",
        "CertifiedPolynomialQ",
    ],
    "algebra/f4.rs": [
        "F4Options",
        "F4BatchReductionResult",
        "f4_reduce_batch",
        "f4_elimination_local",
    ],
    "algebra/elimination.rs": [
        "EliminationStrategy",
        "EliminationGroebnerLocal",
        "F4EliminationLocal",
        "TargetRelationSearchEscalated",
        "ResultantIfSquareOrOverdetermined",
        "SpecializeProjectInterpolateVerify",
        "eliminate_to_keep_variables",
        "validate_local_elimination_result",
    ],
    "algebra/normal_form.rs": ["verify_membership_by_certificate"],
    "result/cost_trace.rs": ["ProjectionCostTrace", "GlobalCostTrace"],
}


P5_FILES = [
    "preprocess/compression.rs",
    "preprocess/definitional.rs",
    "preprocess/linear_affine.rs",
    "preprocess/binomial.rs",
    "preprocess/saturation.rs",
    "preprocess/independent.rs",
    "problem/semantic.rs",
    "result/diagnostics.rs",
]


P5_SYMBOLS = {
    "preprocess/compression.rs": [
        "CompressionState",
        "CompressionTrace",
        "pre_kernel_compress",
        "CompressionSubstitution",
        "GuardRecord",
        "SaturationRecord",
        "FeasibilityObligation",
    ],
    "preprocess/definitional.rs": [
        "DefinitionalCandidate",
        "find_definitional_relations",
        "apply_definitional_elimination",
    ],
    "preprocess/linear_affine.rs": [
        "LinearAffineCandidate",
        "select_safe_affine_pivots",
        "eliminate_linear_affine_variables",
    ],
    "preprocess/binomial.rs": ["BinomialCandidate", "simplify_binomial_relations"],
    "preprocess/saturation.rs": [
        "ExplicitNonzeroWitness",
        "apply_explicit_saturations",
        "is_explicit_nonzero_factor",
        "explicit_nonzero_witnesses",
    ],
    "preprocess/independent.rs": [
        "mark_target_independent_components",
        "compute_component_feasibility_obligations",
    ],
    "problem/semantic.rs": ["RealConstraintKind", "RealConstraintEncoding"],
    "result/diagnostics.rs": ["DiagnosticRecord"],
}


P6_FILES = [
    "graph/hypergraph.rs",
    "graph/influence.rs",
    "graph/weighted_primal.rs",
    "graph/separators.rs",
    "graph/tree_decomposition.rs",
    "graph/projection_dag.rs",
    "graph/metrics.rs",
    "planner/cost_model.rs",
]


P6_SYMBOLS = {
    "graph/hypergraph.rs": [
        "RelationVariableHypergraph",
        "build_relation_variable_hypergraph",
        "connected_components",
    ],
    "graph/influence.rs": ["TargetInfluenceGraph", "build_target_influence_graph"],
    "graph/weighted_primal.rs": ["WeightedPrimalGraph", "build_weighted_primal_graph"],
    "graph/separators.rs": [
        "articulation_variable_candidates",
        "min_fill_separator_candidates",
        "bounded_min_cut_separator_candidates",
        "score_separator",
    ],
    "graph/tree_decomposition.rs": [
        "DecompositionTree",
        "build_target_rooted_decomposition",
    ],
    "graph/projection_dag.rs": [
        "TargetProjectionDAG",
        "ProjectionBlock",
        "RelationDuplicationCertificate",
        "build_target_projection_dag",
        "validate_projection_dag",
        "authorize_block_relations",
    ],
    "graph/metrics.rs": [
        "structural_metrics",
        "estimate_local_quotient_rank",
        "estimate_sparse_template_size",
        "estimate_coefficient_growth",
    ],
    "planner/cost_model.rs": ["estimate_kernel_cost", "classify_route_cost"],
}


P7_FILES = [
    "planner/cost_model.rs",
    "planner/probes.rs",
    "planner/admission.rs",
    "planner/kernel_plan.rs",
    "planner/ladder.rs",
    "planner/planner.rs",
    "kernels/mod.rs",
    "kernels/traits.rs",
    "solver/options.rs",
]


P7_SYMBOLS = {
    "planner/cost_model.rs": ["estimate_kernel_cost", "estimate_kernel_cost_for_admission", "compare_cost"],
    "planner/probes.rs": [
        "run_cost_probes",
        "modular_rank_probe",
        "local_macaulay_size_probe",
        "mixed_support_probe",
        "probe_hash",
    ],
    "planner/admission.rs": [
        "KernelAdmissionEvidence",
        "KernelAdmission",
        "KernelAdmissionStatus",
        "all_planner_kernel_kinds",
        "collect_kernel_admissions",
    ],
    "planner/kernel_plan.rs": [
        "KernelPlan",
        "KernelExecutionPlan",
        "RouteBudget",
        "require_declared_kernel_plan",
        "hash_kernel_execution_plan",
    ],
    "planner/ladder.rs": ["build_declared_ladder", "has_enforceable_route_budget"],
    "planner/planner.rs": [
        "plan_all_blocks",
        "ensure_universal_admitted_for_relation_block",
        "record_structural_empty_block",
    ],
    "kernels/mod.rs": ["all_kernels", "kernel_by_kind"],
    "kernels/traits.rs": ["KernelKind", "TargetProjectionKernel"],
    "solver/options.rs": ["kernel_priority"],
}


P7_KERNEL_ORDER = [
    "TargetUnivariate",
    "LinearAffine",
    "TargetRelationSearch",
    "SparseResultantProjection",
    "TargetActionKrylov",
    "NormTraceProjection",
    "RegularChainProjection",
    "SpecializationInterpolation",
    "UniversalTargetElimination",
]


P8_FILES = [
    "kernels/target_univariate.rs",
    "kernels/linear_affine.rs",
    "verify/certificates.rs",
    "verify/verify_message.rs",
]


P8_SYMBOLS = {
    "kernels/target_univariate.rs": [
        "TargetUnivariateKernel",
        "admit_target_univariate_with_messages",
        "execute_target_univariate",
        "collect_target_relation_inputs",
    ],
    "kernels/linear_affine.rs": [
        "LinearAffineKernel",
        "find_triangular_affine_order",
        "plan_linear_affine",
        "execute_linear_affine",
    ],
    "verify/certificates.rs": [
        "MembershipProjectionCertificate",
        "GuardedAffineProjectionCertificate",
    ],
    "verify/verify_message.rs": ["verify_projection_message"],
}


P9_FILES = [
    "kernels/target_relation_search.rs",
    "planner/relation_schedule.rs",
    "algebra/linear_solve.rs",
    "verify/certificates.rs",
    "verify/verify_message.rs",
]


P9_SYMBOLS = {
    "kernels/target_relation_search.rs": [
        "TargetRelationSearchKernel",
        "admit_target_relation_search",
        "execute_target_relation_search",
        "build_membership_matrix",
        "reconstruct_and_verify_relation",
        "verify_membership_exact",
    ],
    "planner/relation_schedule.rs": [
        "SupportDescriptor",
        "DenseTotalDegree",
        "SparseFootprint",
        "SpecializedInterpolationFootprint",
        "build_dense_relation_search_schedule",
        "build_sparse_relation_search_schedule",
    ],
    "algebra/linear_solve.rs": ["solve_homogeneous_modular", "ModularProofStatus"],
    "verify/certificates.rs": [
        "TargetRelationSearchCertificate",
        "target_relation_exact_identity_hash",
        "target_relation_multipliers_hash",
    ],
    "verify/verify_message.rs": [
        "verify_projection_message",
        "verify_target_relation_search_certificate",
    ],
}


P10_FILES = [
    "algebra/resultant.rs",
    "kernels/sparse_resultant.rs",
    "verify/certificates.rs",
    "verify/verify_message.rs",
]


P10_SYMBOLS = {
    "algebra/resultant.rs": [
        "ResultantInput",
        "ResultantTemplate",
        "SparseResultantCertificate",
        "ResultantBackendKind",
        "build_sparse_resultant_template",
        "compute_resultant_relation",
        "verify_resultant_certificate",
    ],
    "kernels/sparse_resultant.rs": [
        "SparseResultantProjectionKernel",
        "admit_sparse_resultant",
        "plan_sparse_resultant_with_messages",
        "execute_sparse_resultant",
        "probe_sparse_resultant_plan",
    ],
    "verify/certificates.rs": ["SparseResultantProjectionCertificate"],
    "verify/verify_message.rs": ["verify_sparse_resultant_payload"],
}


P11_FILES = [
    "algebra/quotient.rs",
    "algebra/krylov.rs",
    "kernels/action_krylov.rs",
    "verify/certificates.rs",
    "verify/verify_message.rs",
]


P11_SYMBOLS = {
    "algebra/quotient.rs": [
        "ProductionQuotientHandleInput",
        "ProductionProvenancedTargetQuotientHandle",
        "build_production_target_relevant_quotient_handle",
        "build_production_target_relevant_quotient_input_from_relations",
        "normal_form_basis_certificate",
        "make_action_column_certificate",
    ],
    "algebra/krylov.rs": [
        "block_krylov_sequence",
        "recover_recurrence",
        "certify_krylov_coverage",
        "verify_annihilator",
        "CoverageKind::VerifiedCharacteristicSupportCoverage",
    ],
    "kernels/action_krylov.rs": [
        "TargetActionKrylovKernel",
        "admit_target_action_krylov",
        "plan_target_action_krylov_with_messages",
        "execute_target_action_krylov",
        "build_target_action_krylov_trace",
    ],
    "verify/certificates.rs": ["TargetActionProjectionCertificate"],
    "verify/verify_message.rs": ["verify_target_action_payload"],
}


P12_FILES = [
    "kernels/universal_elimination.rs",
    "algebra/elimination.rs",
    "algebra/f4.rs",
    "kernels/target_relation_search.rs",
    "kernels/sparse_resultant.rs",
    "kernels/specialization_interpolation.rs",
    "planner/kernel_plan.rs",
    "verify/certificates.rs",
    "verify/verify_message.rs",
]


P12_SYMBOLS = {
    "kernels/universal_elimination.rs": [
        "UniversalTargetEliminationKernel",
        "admit_universal_elimination",
        "plan_universal_elimination_with_messages",
        "execute_universal_elimination_with_solver_ctx",
        "fixed_universal_strategy_sequence",
        "validate_fixed_strategy_sequence",
        "wrap_stage_message",
        "extract_verified_export_generators",
    ],
    "algebra/elimination.rs": [
        "EliminationStrategy",
        "EliminationGroebnerLocal",
        "F4EliminationLocal",
        "validate_local_elimination_result",
        "verify_membership_by_certificate",
    ],
    "algebra/f4.rs": ["f4_elimination_local"],
    "planner/kernel_plan.rs": ["UniversalStrategy", "universal_strategy_step_with_cost"],
    "verify/certificates.rs": ["UniversalProjectionCertificate", "UniversalStrategyTraceRecord"],
    "verify/verify_message.rs": [
        "verify_universal_strategy_trace",
        "universal_strategy_for_inner_payload",
        "verify_membership_outputs",
    ],
}


P13_FILES = [
    "algebra/regular_chain.rs",
    "kernels/regular_chain_projection.rs",
    "algebra/norm_trace.rs",
    "kernels/norm_trace_projection.rs",
    "algebra/interpolation.rs",
    "kernels/specialization_interpolation.rs",
    "verify/certificates.rs",
    "verify/verify_message.rs",
]


P13_SYMBOLS = {
    "algebra/regular_chain.rs": [
        "RegularChainDAG",
        "RegularChain",
        "ProjectionGenerators",
        "RegularityEvidence",
        "GuardConditionEvidence",
        "local_regular_chain_decomposition",
        "verify_regular_chain_dag_evidence",
        "project_chain_to_variables",
        "combine_chain_projections",
    ],
    "kernels/regular_chain_projection.rs": [
        "RegularChainProjectionKernel",
        "execute_regular_chain_projection",
        "build_regular_chain_trace",
    ],
    "algebra/norm_trace.rs": [
        "detect_explicit_tower_plan",
        "norm_relation_for_tower_plan",
        "verify_norm_tower_plan_relation",
    ],
    "kernels/norm_trace_projection.rs": [
        "NormTraceProjectionKernel",
        "execute_norm_trace_projection",
        "build_norm_trace_trace",
    ],
    "algebra/interpolation.rs": [
        "choose_multiseparator_specialization_points",
        "interpolate_sparse_coefficients_with_support",
        "verify_interpolated_relation",
    ],
    "kernels/specialization_interpolation.rs": [
        "SpecializationInterpolationKernel",
        "execute_specialization_interpolation",
        "verify_interpolated_relation_by_elimination",
    ],
    "verify/certificates.rs": [
        "RegularChainProjectionCertificate",
        "NormTraceProjectionCertificate",
        "SpecializationInterpolationProjectionCertificate",
    ],
    "verify/verify_message.rs": [
        "verify_regular_chain_payload",
        "verify_norm_tower_plan_relation",
        "verify_interpolated_relation",
    ],
}


P14_FILES = [
    "compose/message.rs",
    "compose/compose.rs",
    "compose/separator_elimination.rs",
    "compose/final_support.rs",
    "verify/certificates.rs",
    "verify/verify_message.rs",
    "verify/verify_support.rs",
    "verify/replay.rs",
    "verify/run_certificate.rs",
    "result/cost_trace.rs",
]


P14_SYMBOLS = {
    "compose/message.rs": ["MessageIdeal", "message_to_relations", "merge_messages"],
    "compose/compose.rs": [
        "compose_projection_messages",
        "message_to_relations",
        "merge_messages",
        "eliminate_separators_from_message_relations",
        "message_relations_have_target_eliminant",
    ],
    "compose/separator_elimination.rs": [
        "eliminate_separators_from_message_relations",
        "message_only_system",
        "message_only_block",
        "verify_separator_elimination_message",
    ],
    "compose/final_support.rs": [
        "build_global_support_polynomial",
        "build_final_support_or_nonfinite_with_system",
        "support_from_target_only_relations",
        "support_from_composed_ideal_membership",
        "certify_nonfinite_target_image_with_system",
        "RealNonFiniteSemanticGuardSaturationCertificate",
    ],
    "verify/verify_support.rs": [
        "verify_global_support",
        "GlobalSupportProofRoute",
        "ComposedIdealMembership",
        "verify_composed_ideal_membership_support_certificate",
        "verify_separator_elimination_message",
    ],
    "verify/replay.rs": ["replay_run_certificate", "replay_target_projection_dag", "verify_support_from_messages"],
    "verify/run_certificate.rs": [
        "CoreRunCertificate",
        "CoreInvariantFlags",
        "FinalDagReplayEvidence",
        "build_core_run_certificate",
        "hash_core_run_certificate",
    ],
}


P15_FILES = [
    "algebra/real_root.rs",
    "algebra/sign.rs",
    "roots/squarefree.rs",
    "roots/isolate.rs",
    "roots/decode.rs",
    "roots/algebraic_number.rs",
]


P15_SYMBOLS = {
    "algebra/real_root.rs": [
        "sturm_sequence",
        "isolate_real_roots_sturm",
        "isolate_real_roots_descartes",
        "isolate_descartes_interval",
        "descartes_variations_on_interval",
        "pow_linear",
        "convolve_coeffs",
    ],
    "algebra/sign.rs": ["sign_at_algebraic_root", "thom_encoding"],
    "roots/squarefree.rs": ["squarefree_support"],
    "roots/isolate.rs": ["RootIsolationOptions", "RealRootRecord", "isolate_real_roots"],
    "roots/decode.rs": ["TargetCandidate", "decode_candidates", "hash_target_candidate"],
    "roots/algebraic_number.rs": [
        "AlgebraicRootRecord",
        "algebraic_roots_from_support",
        "refine_algebraic_root",
        "refine_algebraic_root_to_width",
        "compare_algebraic_roots",
        "hash_algebraic_root",
    ],
}


P16_FILES = [
    "solver/options.rs",
    "solver/pipeline.rs",
    "solver/orchestrator.rs",
    "result/status.rs",
    "result/output.rs",
    "problem/semantic.rs",
    "verify/run_certificate.rs",
    "verify/replay.rs",
]


P16_SYMBOLS = {
    "solver/options.rs": ["exact_image_mode", "SolverOptions"],
    "solver/orchestrator.rs": [
        "ExactImageOutOfScope",
        "P16ExactImageScopeGuard",
        "exact_image_out_of_scope_diagnostic",
        "exact_image_out_of_scope_nonfinite_diagnostic",
        "SolverStatus::CertificateDesignGap",
    ],
    "result/status.rs": ["CertificateDesignGap"],
    "result/output.rs": ["TargetSolveResult"],
    "problem/semantic.rs": ["RealConstraintKind", "RealConstraintEncoding"],
    "verify/run_certificate.rs": [
        "exact_image_certificate_hash",
        "hash_solver_options",
        "options.exact_image_mode",
    ],
    "verify/replay.rs": ["exact_image_certificate_hash"],
}


P17_FILES = [
    "solver/options.rs",
    "solver/pipeline.rs",
    "solver/orchestrator.rs",
    "result/output.rs",
    "result/cost_trace.rs",
    "verify/run_certificate.rs",
    "verify/replay.rs",
]


P17_SYMBOLS = {
    "solver/options.rs": [
        "finite_candidate_cover_mode",
        "exact_image_mode",
        "max_matrix_rows",
        "max_matrix_cols",
        "max_coefficient_height_bits",
        "root_isolation_method",
        "certificate_level",
    ],
    "solver/pipeline.rs": [
        "step_validate",
        "step_canonicalize",
        "step_compress",
        "step_build_graphs",
        "step_build_dag",
        "step_plan",
        "step_execute",
        "step_verify_messages",
        "step_compose",
        "step_support",
        "step_roots",
        "step_core_certificate",
        "step_cost_trace",
    ],
    "solver/orchestrator.rs": ["solve_with_context", "finalize_pipeline_error", "verify_global_support"],
    "result/output.rs": ["TargetSolveResult", "from_solver_error_for_target_with_cost_trace"],
    "result/cost_trace.rs": [
        "GlobalCostTrace",
        "ProjectionCostTrace",
        "RouteCostTrace",
        "final_support_degree",
        "certificate_size",
    ],
    "verify/run_certificate.rs": ["CoreRunCertificate", "hash_solver_options", "finite_candidate_cover_mode"],
    "verify/replay.rs": ["replay_run_certificate", "verify_support_from_messages"],
}


def scan_required_files(findings: list[Finding], phase: str | None) -> None:
    if phase == "P1":
        required = P1_FILES
    elif phase == "P2":
        required = P2_FILES
    elif phase == "P3":
        required = P3_FILES
    elif phase == "P4":
        required = P4_FILES
    elif phase == "P5":
        required = P5_FILES
    elif phase == "P6":
        required = P6_FILES
    elif phase == "P7":
        required = P7_FILES
    elif phase == "P8":
        required = P8_FILES
    elif phase == "P9":
        required = P9_FILES
    elif phase == "P10":
        required = P10_FILES
    elif phase == "P11":
        required = P11_FILES
    elif phase == "P12":
        required = P12_FILES
    elif phase == "P13":
        required = P13_FILES
    elif phase == "P14":
        required = P14_FILES
    elif phase == "P15":
        required = P15_FILES
    elif phase == "P16":
        required = P16_FILES
    elif phase == "P17":
        required = P17_FILES
    else:
        required = REQUIRED_FILES
    for file_name in required:
        if not (SRC / file_name).exists():
            findings.append(
                Finding("error", "missing_required_file", file_name, 0, "Required production file is missing")
            )


def scan_symbols(findings: list[Finding], phase: str | None) -> None:
    if phase == "P1":
        required_symbols = P1_SYMBOLS
    elif phase == "P2":
        required_symbols = P2_SYMBOLS
    elif phase == "P3":
        required_symbols = P3_SYMBOLS
    elif phase == "P4":
        required_symbols = P4_SYMBOLS
    elif phase == "P5":
        required_symbols = P5_SYMBOLS
    elif phase == "P6":
        required_symbols = P6_SYMBOLS
    elif phase == "P7":
        required_symbols = P7_SYMBOLS
    elif phase == "P8":
        required_symbols = P8_SYMBOLS
    elif phase == "P9":
        required_symbols = P9_SYMBOLS
    elif phase == "P10":
        required_symbols = P10_SYMBOLS
    elif phase == "P11":
        required_symbols = P11_SYMBOLS
    elif phase == "P12":
        required_symbols = P12_SYMBOLS
    elif phase == "P13":
        required_symbols = P13_SYMBOLS
    elif phase == "P14":
        required_symbols = P14_SYMBOLS
    elif phase == "P15":
        required_symbols = P15_SYMBOLS
    elif phase == "P16":
        required_symbols = P16_SYMBOLS
    elif phase == "P17":
        required_symbols = P17_SYMBOLS
    else:
        required_symbols = REQUIRED_SYMBOLS
    test_text = "\n".join(path.read_text(encoding="utf-8", errors="ignore") for path in all_test_files())
    for file_name, symbols in required_symbols.items():
        path = SRC / file_name
        prod_text = strip_cfg_test_blocks(path.read_text(encoding="utf-8", errors="ignore")) if path.exists() else ""
        full_text = path.read_text(encoding="utf-8", errors="ignore") if path.exists() else ""
        for symbol in symbols:
            pattern = symbol_regex(symbol)
            in_prod = bool(pattern.search(prod_text))
            in_full = bool(pattern.search(full_text))
            in_tests = bool(pattern.search(test_text))
            if not in_prod:
                code = "test_only_required_symbol" if in_full or in_tests else "missing_required_symbol"
                findings.append(
                    Finding(
                        "error",
                        code,
                        file_name,
                        0,
                        f"In-scope required symbol `{symbol}` is not visible in production code",
                    )
                )


def scan_forbidden_patterns(findings: list[Finding], phase: str | None) -> None:
    if phase == "P1":
        files = [SRC / file_name for file_name in P1_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P2":
        files = [SRC / file_name for file_name in P2_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P3":
        files = [SRC / file_name for file_name in P3_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P4":
        files = [SRC / file_name for file_name in P4_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P5":
        files = [SRC / file_name for file_name in P5_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P6":
        files = [SRC / file_name for file_name in P6_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P7":
        files = [SRC / file_name for file_name in P7_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P8":
        files = [SRC / file_name for file_name in P8_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P9":
        files = [SRC / file_name for file_name in P9_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P10":
        files = [SRC / file_name for file_name in P10_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P11":
        files = [SRC / file_name for file_name in P11_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P12":
        files = [SRC / file_name for file_name in P12_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P13":
        files = [SRC / file_name for file_name in P13_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P14":
        files = [SRC / file_name for file_name in P14_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P15":
        files = [SRC / file_name for file_name in P15_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P16":
        files = [SRC / file_name for file_name in P16_FILES]
        files = [path for path in files if path.exists()]
    elif phase == "P17":
        files = [SRC / file_name for file_name in P17_FILES]
        files = [path for path in files if path.exists()]
    else:
        files = production_rust_files()
    for path in files:
        for line_no, line in enumerate(lines(path), start=1):
            for code, pattern in FORBIDDEN_PATTERNS:
                if pattern.search(line):
                    findings.append(Finding("error", code, rel(path), line_no, line.strip()))
            if phase != "P1" and GEOMETRY_DISPATCH_PATTERN.search(line):
                findings.append(Finding("error", "possible_geometry_or_label_dispatch", rel(path), line_no, line.strip()))
            if phase != "P1" and COORDINATE_SOLVER_PATTERN.search(line):
                if not is_allowed_no_coordinate_export_marker(line):
                    findings.append(Finding("error", "possible_coordinate_rur_path", rel(path), line_no, line.strip()))
            if phase != "P1" and EXACT_IMAGE_SUCCESS_PATTERN.search(line):
                rel_path = rel(path)
                if is_allowed_exact_image_reference(phase, rel_path, line):
                    continue
                findings.append(
                    Finding(
                        "error",
                        "exact_image_success_in_scoped_repair",
                        rel_path,
                        line_no,
                        line.strip(),
                    )
                )
            if phase == "P4" and re.search(r"\b(NotProduction|NonProduction|for_tests)\b", line):
                findings.append(
                    Finding(
                        "error",
                        "p4_nonproduction_f4_marker",
                        rel(path),
                        line_no,
                        line.strip(),
                    )
                )


def scan_p5_specific(findings: list[Finding]) -> None:
    compression = SRC / "preprocess" / "compression.rs"
    if compression.exists():
        text = compression.read_text(encoding="utf-8", errors="ignore")
        body_match = re.search(
            r"pub fn pre_kernel_compress\b(?P<body>.*?pub fn compress_system\b)",
            text,
            flags=re.DOTALL,
        )
        body = body_match.group("body") if body_match else text
        required_order = [
            "apply_definitional_elimination",
            "eliminate_linear_affine_variables",
            "simplify_binomial_relations",
            "apply_explicit_saturations",
            "mark_target_independent_components",
        ]
        positions = [body.find(name) for name in required_order]
        if any(pos < 0 for pos in positions) or positions != sorted(positions):
            findings.append(
                Finding(
                    "error",
                    "p5_compression_order_mismatch",
                    "preprocess/compression.rs",
                    0,
                    "pre_kernel_compress does not show the required source compression order",
                )
            )
    saturation = SRC / "preprocess" / "saturation.rs"
    if saturation.exists():
        text = saturation.read_text(encoding="utf-8", errors="ignore")
        for needle in ["RealConstraintKind::NonZero", "encoded_witnesses", "extract_nonzero_witness"]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p5_saturation_semantic_binding_missing",
                        "preprocess/saturation.rs",
                        0,
                        f"saturation path does not contain `{needle}`",
                    )
                )
    linear = SRC / "preprocess" / "linear_affine.rs"
    if linear.exists():
        text = linear.read_text(encoding="utf-8", errors="ignore")
        if "UnsafeAffinePivotRejected" not in text:
            findings.append(
                Finding(
                    "error",
                    "p5_unsafe_affine_diagnostic_missing",
                    "preprocess/linear_affine.rs",
                    0,
                    "unsafe affine pivot rejection diagnostic is missing",
                )
            )
    independent = SRC / "preprocess" / "independent.rs"
    if independent.exists():
        text = independent.read_text(encoding="utf-8", errors="ignore")
        for needle in ["FeasibilityObligation", "contains_target", "filter(|relation|"]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p5_independent_obligation_missing",
                        "preprocess/independent.rs",
                        0,
                        f"target-independent component handling does not contain `{needle}`",
                    )
                )


def scan_p6_specific(findings: list[Finding]) -> None:
    hypergraph = SRC / "graph" / "hypergraph.rs"
    if hypergraph.exists():
        text = hypergraph.read_text(encoding="utf-8", errors="ignore")
        if "for variable in &system.variables" not in text:
            findings.append(
                Finding(
                    "error",
                    "p6_hypergraph_missing_all_variables",
                    "graph/hypergraph.rs",
                    0,
                    "hypergraph construction does not visibly add every compressed variable",
                )
            )
    influence = SRC / "graph" / "influence.rs"
    if influence.exists():
        text = influence.read_text(encoding="utf-8", errors="ignore")
        for needle in ["bfs_target_component", "VecDeque::from([target])", "variable_relations", "relation_variables"]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p6_influence_bfs_missing",
                        "graph/influence.rs",
                        0,
                        f"target influence BFS path does not contain `{needle}`",
                    )
                )
    tree = SRC / "graph" / "tree_decomposition.rs"
    if tree.exists():
        text = tree.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "articulation_variable_candidates",
            "min_fill_separator_candidates",
            "bounded_min_cut_separator_candidates",
            "candidates.extend(bounded_min_cut_separator_candidates(g, target));\n    if g.variables.len() >= 6",
            "no_separator",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p6_separator_family_missing",
                        "graph/tree_decomposition.rs",
                        0,
                        f"decomposition path does not contain `{needle}`",
                    )
                )
    dag = SRC / "graph" / "projection_dag.rs"
    if dag.exists():
        text = dag.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "projection DAG omitted a compressed relation",
            "projection DAG duplicated a relation without a certificate",
            "projection block authorization hash mismatch",
            "relation_vars.is_subset(&block.local_variables)",
            "validate_projection_dag_topology",
            "projection DAG non-root block has no parent",
            "projection DAG parent does not list non-root child",
            "projection DAG block is reachable more than once from root",
            "projection DAG contains a block unreachable from root",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p6_dag_validation_missing",
                        "graph/projection_dag.rs",
                        0,
                        f"DAG validation does not contain `{needle}`",
                    )
                )


def scan_p7_specific(findings: list[Finding]) -> None:
    traits = SRC / "kernels" / "traits.rs"
    if traits.exists():
        text = traits.read_text(encoding="utf-8", errors="ignore")
        match = re.search(r"pub enum KernelKind\s*\{(?P<body>.*?)\}", text, flags=re.DOTALL)
        body = match.group("body") if match else ""
        variants = [
            line.strip().rstrip(",")
            for line in body.splitlines()
            if line.strip() and not line.strip().startswith("//")
        ]
        if variants != P7_KERNEL_ORDER:
            findings.append(
                Finding(
                    "error",
                    "p7_kernel_kind_order_mismatch",
                    "kernels/traits.rs",
                    0,
                    f"KernelKind order is {variants}, expected {P7_KERNEL_ORDER}",
                )
            )
    for file_name, function_name in [
        ("kernels/mod.rs", "all_kernels"),
        ("planner/admission.rs", "all_planner_kernel_kinds"),
    ]:
        path = SRC / file_name
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        positions = [text.find(f"KernelKind::{name}") for name in P7_KERNEL_ORDER]
        if any(pos < 0 for pos in positions) or positions != sorted(positions):
            findings.append(
                Finding(
                    "error",
                    "p7_kernel_registry_order_mismatch",
                    file_name,
                    0,
                    f"{function_name} does not visibly list all nine kernels in source order",
                )
            )
    admission = SRC / "planner" / "admission.rs"
    if admission.exists():
        text = admission.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "all_kernels()",
            "kernel.admit(block, &kernel_context)",
            ".into_iter()",
            "KernelAdmissionStatus::PlanProbeFailed",
            "admission_evidence",
            "runtime_admission_hash",
            "source_relation_ids",
            "source_relation_hashes",
            "initial_resource_bounds",
            "estimated_matrix_rows",
            "estimated_matrix_cols",
            "estimated_template_size",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p7_admission_all_kernel_path_missing",
                        "planner/admission.rs",
                        0,
                        f"admission path does not contain `{needle}`",
                    )
                )
    ladder = SRC / "planner" / "ladder.rs"
    if ladder.exists():
        text = ladder.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "filter(has_enforceable_route_budget)",
            "compare_cost",
            "preferred_order",
            "!preferred_order.contains(&KernelKind::UniversalTargetElimination)",
            "plans.push(universal)",
            "max_coefficient_height_bits > 0",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p7_ladder_guard_missing",
                        "planner/ladder.rs",
                        0,
                        f"declared ladder path does not contain `{needle}`",
                    )
                )
    planner = SRC / "planner" / "planner.rs"
    if planner.exists():
        text = planner.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "postorder_key",
            "record_structural_empty_block",
            "ensure_universal_admitted_for_relation_block",
            "relation block has no admitted UniversalTargetElimination kernel",
            "build_declared_ladder",
            "KernelRouteTrace",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p7_planner_contract_missing",
                        "planner/planner.rs",
                        0,
                        f"planner path does not contain `{needle}`",
                    )
                )
    kernel_plan = SRC / "planner" / "kernel_plan.rs"
    if kernel_plan.exists():
        text = kernel_plan.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "declared ladder hash mismatch before execution",
            "kernel absent from declared ladder",
            "source_relation_ids",
            "source_relation_hashes",
            "child_block_ids",
            "child_message_hashes",
            "route_budget",
            "failure_behavior",
            "plan_hash",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p7_execution_plan_contract_missing",
                        "planner/kernel_plan.rs",
                        0,
                        f"execution plan contract does not contain `{needle}`",
                    )
                )


def scan_p8_specific(findings: list[Finding]) -> None:
    target = SRC / "kernels" / "target_univariate.rs"
    if target.exists():
        text = target.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "ctx.child_messages",
            "collect_target_relation_inputs",
            "lcm_denominators",
            "squarefree_part_uni",
            "MessageRepresentation::PrincipalSupport",
            "CertificateRoute::SourceMembershipCertificate",
            "source_relation_ids",
            "source_relation_hashes",
            "child_message_hashes",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p8_target_univariate_contract_missing",
                        "kernels/target_univariate.rs",
                        0,
                        f"TargetUnivariate path does not contain `{needle}`",
                    )
                )
    affine = SRC / "kernels" / "linear_affine.rs"
    if affine.exists():
        text = affine.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "find_triangular_affine_order",
            "choose_safe_affine_pivot",
            "constant_nonzero",
            "denominator_guard_hash",
            "guard.factor.hash == pivot.hash",
            "unsafe affine pivot in plan",
            "GuardedAffineProjectionCertificate",
            "CertificateRoute::GuardedAffineProjectionCertificate",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p8_linear_affine_contract_missing",
                        "kernels/linear_affine.rs",
                        0,
                        f"LinearAffine path does not contain `{needle}`",
                    )
                )
    verifier = SRC / "verify" / "verify_message.rs"
    if verifier.exists():
        text = verifier.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "projection message relation contains a non-exported variable",
            "kernel certificate exported variables do not match message exports",
            "replay_guarded_affine_outputs",
            "guarded affine nonconstant pivot lacks guard hash",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p8_verify_message_contract_missing",
                        "verify/verify_message.rs",
                        0,
                        f"message verification path does not contain `{needle}`",
                    )
                )


def scan_p9_specific(findings: list[Finding]) -> None:
    trs = SRC / "kernels" / "target_relation_search.rs"
    if trs.exists():
        text = trs.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "collect_relation_search_inputs",
            "ctx.child_messages",
            "build_membership_matrix_builder_with_supports",
            "solve_homogeneous_modular",
            "verify_membership_exact",
            "target_relation_search: Some(target_relation_search)",
            "target_relation_search_certificate_hash",
            "rational_reconstruction_hash",
            "algorithmic_hard_case_with_traces",
            "p9_target_relation_search_uses_child_message_relations",
            "p9_target_relation_search_certificate_fields_reject_tamper",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p9_target_relation_search_contract_missing",
                        "kernels/target_relation_search.rs",
                        0,
                        f"TargetRelationSearch path does not contain `{needle}`",
                    )
                )
    linear_solve = SRC / "algebra" / "linear_solve.rs"
    if linear_solve.exists():
        text = linear_solve.read_text(encoding="utf-8", errors="ignore")
        for needle in ["CandidateOnlyRequiresExactQCheck", "reconstructed_basis_candidates"]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p9_modular_candidate_status_missing",
                        "algebra/linear_solve.rs",
                        0,
                        f"modular solve path does not contain `{needle}`",
                    )
                )
    relation_schedule = SRC / "planner" / "relation_schedule.rs"
    if relation_schedule.exists():
        text = relation_schedule.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "DenseTotalDegree",
            "SparseFootprint",
            "SpecializedInterpolationFootprint",
            "specialized_interpolation_footprint_descriptor",
            "build_sparse_export_monomial_support",
            "build_sparse_multiplier_supports",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p9_support_strategy_missing",
                        "planner/relation_schedule.rs",
                        0,
                        f"relation search support strategy path does not contain `{needle}`",
                    )
                )
    certificates = SRC / "verify" / "certificates.rs"
    if certificates.exists():
        text = certificates.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "TargetRelationSearchCertificate",
            "exported_variables_hash",
            "eliminated_variables_hash",
            "export_support_hash",
            "multiplier_support_hash",
            "membership_matrix_hash",
            "primes_used",
            "rational_reconstruction_hash",
            "relation_hash",
            "multipliers_hash",
            "exact_identity_hash",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p9_certificate_field_missing",
                        "verify/certificates.rs",
                        0,
                        f"TargetRelationSearch certificate does not contain `{needle}`",
                    )
                )
    verifier = SRC / "verify" / "verify_message.rs"
    if verifier.exists():
        text = verifier.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "verify_target_relation_search_certificate",
            "target relation search exact identity hash mismatch",
            "target relation search multipliers hash mismatch",
            "target relation search source relation ids mismatch",
            "target relation search membership matrix hash mismatch",
            "target relation search rational reconstruction hash mismatch",
            "target relation search modular prime trace mismatch",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p9_verify_message_contract_missing",
                        "verify/verify_message.rs",
                        0,
                        f"message verifier does not contain `{needle}`",
                    )
                )


def scan_p10_specific(findings: list[Finding]) -> None:
    resultant = SRC / "algebra" / "resultant.rs"
    if resultant.exists():
        text = resultant.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "pub struct ResultantInput",
            "pub struct ResultantTemplate",
            "pub struct MonomialSupport",
            "support_sets",
            "max_matrix_dim",
            "ResultantProofStatus::CandidateOnlyRequiresExactMembership",
            "ResultantBackendKind::LinearSubresultant",
            "ResultantBackendKind::QuadraticSubresultant",
            "ResultantBackendKind::SmallEntrySymbolicDeterminant",
            "modular_traces",
            "exact_verification_hash",
            "verify_resultant_certificate",
            "finite_resource_failure",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p10_resultant_contract_missing",
                        "algebra/resultant.rs",
                        0,
                        f"resultant algebra path does not contain `{needle}`",
                    )
                )
    kernel = SRC / "kernels" / "sparse_resultant.rs"
    if kernel.exists():
        text = kernel.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "ResultantRelationInput",
            "collect_relation_inputs",
            "probe_sparse_resultant_plan",
            "build_sparse_resultant_trace",
            "trace_swell_within_planned_preflight",
            "SparseResultantPlannedSwellPreflight",
            "SparseResultantMatrix",
            "CandidateCoverStrong",
            "verify_resultant_certificate(&resultant.certificate)",
            "sparse resultant produced a relation outside exported variables",
            "p12g_sparse_resultant_template_plan_does_not_overclaim_binary_chain",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p10_sparse_resultant_kernel_contract_missing",
                        "kernels/sparse_resultant.rs",
                        0,
                        f"SparseResultant kernel path does not contain `{needle}`",
                    )
                )
    verifier = SRC / "verify" / "verify_message.rs"
    if verifier.exists():
        text = verifier.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "verify_sparse_resultant_payload",
            "verify_resultant_certificate(cert)",
            "compute_resultant_relation(&template, ModularOptions::default())",
            "sparse resultant certificate failed exact replay",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p10_sparse_resultant_verify_contract_missing",
                        "verify/verify_message.rs",
                        0,
                        f"SparseResultant verifier path does not contain `{needle}`",
                    )
                )


def scan_p11_specific(findings: list[Finding]) -> None:
    quotient = SRC / "algebra" / "quotient.rs"
    if quotient.exists():
        text = quotient.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "BasisScope::TargetRelevant",
            "verify_standard_basis_matches_authorized_relations",
            "verify_normal_form_basis_certificate",
            "verify_action_column_certificate",
            "quotient handle must not expose coordinate roots or full coordinate RUR",
            "no_coordinate_roots_exported",
            "no_full_coordinate_rur_exported",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p11_quotient_contract_missing",
                        "algebra/quotient.rs",
                        0,
                        f"target-relevant quotient handle path does not contain `{needle}`",
                    )
                )
    krylov = SRC / "algebra" / "krylov.rs"
    if krylov.exists():
        text = krylov.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "plan.start_vectors.is_empty()",
            "start.entries.len() != handle.basis_size()",
            "CoverageKind::VerifiedCharacteristicSupportCoverage",
            "recurrence.polynomial != characteristic",
            "verify_cayley_hamilton_matrix_hash",
            "single_vector_krylov_undercoverage_is_rejected",
            "debug_explicit_handle_is_rejected_by_production_krylov_boundary",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p11_krylov_contract_missing",
                        "algebra/krylov.rs",
                        0,
                        f"Krylov coverage path does not contain `{needle}`",
                    )
                )
    kernel = SRC / "kernels" / "action_krylov.rs"
    if kernel.exists():
        text = kernel.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "(0..handle.basis_size())",
            "certify_krylov_coverage(&seq, &recurrence, &handle)?",
            "verify_annihilator(&handle, &coverage.characteristic_polynomial)?",
            "MessageRepresentation::QuotientAction",
            "ProjectionStrength::CandidateCoverStrong",
            "target-action-krylov quotient exported coordinate roots or RUR",
            "p11_action_krylov_replay_rejects_tampered_coverage_after_rehash",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p11_action_krylov_contract_missing",
                        "kernels/action_krylov.rs",
                        0,
                        f"TargetActionKrylov path does not contain `{needle}`",
                    )
                )
    verifier = SRC / "verify" / "verify_message.rs"
    if verifier.exists():
        text = verifier.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "verify_target_action_payload",
            "(0..handle.basis_size())",
            "certify_krylov_coverage(&sequence, &recurrence, &handle)",
            "verify_annihilator(&handle, &coverage.characteristic_polynomial)",
            "target action coverage certificate failed exact replay",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p11_target_action_verify_contract_missing",
                        "verify/verify_message.rs",
                        0,
                        f"TargetAction verifier path does not contain `{needle}`",
                    )
                )


def scan_p12_specific(findings: list[Finding]) -> None:
    expected = [
        "EliminationGroebnerLocal",
        "F4EliminationLocal",
        "TargetRelationSearchEscalated",
        "ResultantIfSquareOrOverdetermined",
        "SpecializeProjectInterpolateVerify",
    ]
    kernel_plan = SRC / "planner" / "kernel_plan.rs"
    if kernel_plan.exists():
        text = kernel_plan.read_text(encoding="utf-8", errors="ignore")
        match = re.search(r"pub enum UniversalStrategy\s*\{(?P<body>.*?)\}", text, flags=re.DOTALL)
        variants = [
            line.strip().rstrip(",")
            for line in (match.group("body") if match else "").splitlines()
            if line.strip() and not line.strip().startswith("//")
        ]
        if variants != expected:
            findings.append(
                Finding(
                    "error",
                    "p12_universal_strategy_enum_mismatch",
                    "planner/kernel_plan.rs",
                    0,
                    f"UniversalStrategy variants are {variants}, expected {expected}",
                )
            )
    universal = SRC / "kernels" / "universal_elimination.rs"
    if universal.exists():
        text = universal.read_text(encoding="utf-8", errors="ignore")
        prod = text.split("#[cfg(test)]")[0]
        for needle in [
            "fixed_universal_strategy_sequence",
            "validate_fixed_strategy_sequence(&plan.support_plan.universal_strategy_sequence)?",
            "KernelKind::TargetRelationSearch",
            "KernelKind::SparseResultantProjection",
            "KernelKind::SpecializationInterpolation",
            "LocalNonfinitePolicy::NoLocalCertifiedNonFinite",
            "algorithmic_hard_case(",
            "local elimination stage found no exported generator",
            "verify_universal_no_coordinate_fallback",
            "validate_local_elimination_result(&result, &stage.exported_variables, &relations)?",
            "extract_verified_export_generators(&result, &stage.exported_variables)?",
            "output_memberships",
            "inner_payload: Some(Box::new(inner_payload))",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p12_universal_contract_missing",
                        "kernels/universal_elimination.rs",
                        0,
                        f"Universal path does not contain `{needle}`",
                    )
                )
        for forbidden in [
            "KernelKind::TargetActionKrylov",
            "KernelKind::RegularChainProjection",
            "KernelKind::NormTraceProjection",
            "execute_target_action",
            "execute_regular_chain",
            "execute_norm_trace",
        ]:
            if forbidden in prod:
                findings.append(
                    Finding(
                        "error",
                        "p12_forbidden_internal_strategy",
                        "kernels/universal_elimination.rs",
                        0,
                        f"Universal production path contains forbidden internal stage `{forbidden}`",
                    )
                )
        for name in expected:
            if prod.count(f"UniversalStrategy::{name}") < 2:
                findings.append(
                    Finding(
                        "error",
                        "p12_strategy_not_bound_in_prod",
                        "kernels/universal_elimination.rs",
                        0,
                        f"Universal production path does not visibly plan and replay `{name}`",
                    )
                )
    elimination = SRC / "algebra" / "elimination.rs"
    if elimination.exists():
        text = elimination.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "EliminationStrategy::F4EliminationLocal(options)",
            "f4_elimination_local(relations, eliminate, keep, options)?",
            "return Err(implementation_bug(",
            "declared elimination strategy is handled by a later planner/kernel phase",
            "verify_membership_by_certificate",
            "!polynomial_in_keep_variables(&generator.generator, &keep_set)",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p12_elimination_contract_missing",
                        "algebra/elimination.rs",
                        0,
                        f"local elimination path does not contain `{needle}`",
                    )
                )
    verifier = SRC / "verify" / "verify_message.rs"
    if verifier.exists():
        text = verifier.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "verify_universal_strategy_trace",
            "universal attempted strategy sequence is not the fixed generic sequence",
            "universal chosen strategy does not match wrapped proof payload",
            "universal payload source relations do not exactly match plan-bound certificate sources",
            "universal failed strategy hashes do not match replayed attempted stage prefix",
            "verify_membership_outputs(",
            "KernelCertificatePayload::SparseResultant",
            "KernelCertificatePayload::SpecializationInterpolation",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p12_universal_verify_contract_missing",
                        "verify/verify_message.rs",
                        0,
                        f"Universal verifier path does not contain `{needle}`",
                    )
                )


def scan_p13_specific(findings: list[Finding]) -> None:
    regular = SRC / "algebra" / "regular_chain.rs"
    if regular.exists():
        text = regular.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "pub struct RegularChainDAG",
            "pub struct ProjectionGenerators",
            "guards: Vec<SparsePolynomialQ>",
            "component_semantics",
            "source_relation_hashes",
            "regularity_evidence",
            "guard_evidence",
            "RegularityCondition::GuardedNonZeroInitial",
            "regular-chain nonconstant initial lacks an explicit nonzero guard",
            "verify_regular_chain_dag_evidence",
            "nonconstant_initial_requires_and_records_guard_evidence",
            "triangular_main_variables",
            "project_chain_to_variables",
            "combine_chain_projections",
            "projection_hash",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p13_regular_chain_contract_missing",
                        "algebra/regular_chain.rs",
                        0,
                        f"regular-chain algebra path does not contain `{needle}`",
                    )
                )
    regular_kernel = SRC / "kernels" / "regular_chain_projection.rs"
    if regular_kernel.exists():
        text = regular_kernel.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "RegularChainProjectionKernel",
            "build_regular_chain_trace",
            "local_regular_chain_decomposition",
            "project_chain_to_variables",
            "combine_chain_projections",
            "RegularChainProjectionCertificate",
            "validate_exported_generators",
            "regular-chain projection trace does not match plan support",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p13_regular_chain_kernel_missing",
                        "kernels/regular_chain_projection.rs",
                        0,
                        f"regular-chain kernel path does not contain `{needle}`",
                    )
                )
    norm = SRC / "algebra" / "norm_trace.rs"
    if norm.exists():
        text = norm.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "detect_explicit_tower_plan",
            "validate_tower_expression",
            "norm_relation_for_tower_plan",
            "compute_resultant_relation",
            "verify_norm_tower_plan_relation",
            "verify_tower_plan_hashes",
            "multistep_tower_norm_eliminates_each_algebraic_variable",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p13_norm_trace_contract_missing",
                        "algebra/norm_trace.rs",
                        0,
                        f"norm/trace algebra path does not contain `{needle}`",
                    )
                )
        forbidden_label_markers = ["geometry_name", "problem_name", "semantic_label", "expected_answer"]
        for marker in forbidden_label_markers:
            if marker in text:
                findings.append(
                    Finding(
                        "error",
                        "p13_norm_trace_label_dispatch",
                        "algebra/norm_trace.rs",
                        0,
                        f"norm/trace detection contains forbidden label marker `{marker}`",
                    )
                )
    norm_kernel = SRC / "kernels" / "norm_trace_projection.rs"
    if norm_kernel.exists():
        text = norm_kernel.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "detect_explicit_tower_plan",
            "norm_relation_for_tower_plan(&tower)?.into_multivariate()",
            "verify_norm_tower_plan_relation(&tower, &relation)",
            "validate_exported_relation",
            "NormTraceProjectionCertificate",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p13_norm_trace_kernel_missing",
                        "kernels/norm_trace_projection.rs",
                        0,
                        f"norm/trace kernel path does not contain `{needle}`",
                    )
                )
    interpolation = SRC / "algebra" / "interpolation.rs"
    if interpolation.exists():
        text = interpolation.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "choose_multiseparator_specialization_points",
            "build_interpolation_matrix",
            "interpolate_sparse_coefficients_with_support",
            "build_interpolation_certificate",
            "verify_interpolated_relation",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p13_interpolation_contract_missing",
                        "algebra/interpolation.rs",
                        0,
                        f"interpolation algebra path does not contain `{needle}`",
                    )
                )
    spec_kernel = SRC / "kernels" / "specialization_interpolation.rs"
    if spec_kernel.exists():
        text = spec_kernel.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "choose_multiseparator_specialization_points",
            "execute_inner_target_only_kernel",
            "build_interpolation_certificate",
            "verify_interpolated_relation(&relation, &interpolation_certificate)",
            "verify_interpolated_relation_by_elimination",
            "interpolation candidate was not verified by exact elimination",
            "hash_specialization_samples",
            "specialization-interpolation-coefficient-support",
            "p12g_specialization_interpolation_inner_schedule_is_declared",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p13_specialization_kernel_missing",
                        "kernels/specialization_interpolation.rs",
                        0,
                        f"specialization/interpolation kernel path does not contain `{needle}`",
                    )
                )
    verifier = SRC / "verify" / "verify_message.rs"
    if verifier.exists():
        text = verifier.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "verify_regular_chain_payload",
            "regular chain regularity or guard evidence failed replay",
            "regular chain projection certificates do not recompute from DAG",
            "verify_norm_tower_plan_relation(&proof.tower, &proof.output_relation)",
            "verify_interpolated_relation(",
            "specialization interpolation output absent from exact elimination result",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p13_verify_message_missing",
                        "verify/verify_message.rs",
                        0,
                        f"P13 verifier path does not contain `{needle}`",
                    )
                )


def scan_p14_specific(findings: list[Finding]) -> None:
    message = SRC / "compose" / "message.rs"
    if message.exists():
        text = message.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "pub struct MessageIdeal",
            "pub fn message_to_relations",
            "pub fn merge_messages",
            "hash_message_ideal",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p14_message_ideal_missing",
                        "compose/message.rs",
                        0,
                        f"message ideal path does not contain `{needle}`",
                    )
                )
    compose = SRC / "compose" / "compose.rs"
    if compose.exists():
        text = compose.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "merge_messages(&messages)",
            "message_to_relations(&message)",
            "eliminate_separators_from_message_relations",
            "CertificateDesignGap",
            "ComposedIdealTargetEliminantCertificate",
            "groebner_elimination_basis(relations, &order, GroebnerOptions::default())",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p14_composition_contract_missing",
                        "compose/compose.rs",
                        0,
                        f"composition path does not contain `{needle}`",
                    )
                )
        heuristic_match = re.search(
            r"fn\s+message_relations_have_target_eliminant(?P<body>.*?)\nfn\s+algorithmic_hard_case",
            text,
            flags=re.DOTALL,
        )
        body = heuristic_match.group("body") if heuristic_match else ""
        for forbidden in [
            "poly_monomial_count",
            "poly_total_degree",
            "> 64",
            "> 8",
            "eliminate.len() >",
            "relations.len() <=",
            ".unwrap_or(false)",
        ]:
            if forbidden in body:
                findings.append(
                    Finding(
                        "error",
                        "p14_fixed_heuristic_eliminant_check",
                        "compose/compose.rs",
                        0,
                        f"target eliminant check still contains fixed heuristic marker `{forbidden}`",
                    )
                )
    separator = SRC / "compose" / "separator_elimination.rs"
    if separator.exists():
        text = separator.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "message_only_system",
            "message_only_block",
            "RelationSource::InputEquation",
            "child_messages: Vec::new()",
            "verify_separator_elimination_message",
            "verify_projection_message(message, &kctx)",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p14_message_only_separator_missing",
                        "compose/separator_elimination.rs",
                        0,
                        f"message-only separator elimination path does not contain `{needle}`",
                    )
                )
    final_support = SRC / "compose" / "final_support.rs"
    if final_support.exists():
        text = final_support.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "support_from_target_only_relations",
            "support_from_composed_ideal_membership",
            "squarefree_part_uni(&univariate_mul(&support, &sq))",
            "certify_zero_target_elimination_ideal",
            "target_free_groebner_basis_hashes",
            "target_algebraic_independence_hash",
            "proper_ideal_witness_hash",
            "dimension_lower_bound",
            "RealNonFiniteSemanticGuardSaturationCertificate",
            "looks_like_unregistered_nonzero_witness",
            "CertificateDesignGap",
            "AlgorithmicHardCase",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p14_final_support_contract_missing",
                        "compose/final_support.rs",
                        0,
                        f"final support/nonfinite path does not contain `{needle}`",
                    )
                )
        if "find_rational_consistency_witness" in text and "target_algebraic_independence_hash" not in text:
            findings.append(
                Finding(
                    "error",
                    "p14_small_witness_without_elimination_certificate",
                    "compose/final_support.rs",
                    0,
                    "nonfinite path has rational witness search without zero target elimination certificate",
                )
            )
    verifier = SRC / "verify" / "verify_support.rs"
    if verifier.exists():
        text = verifier.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "GlobalSupportProofRoute::TargetOnlyRootRelationProduct",
            "GlobalSupportProofRoute::ComposedIdealMembership",
            "verify_separator_elimination_message",
            "verify_composed_ideal_membership_support_certificate",
            "ComposedIdealMembershipSupportCertificate could not reduce S(T) to zero",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p14_support_verifier_missing",
                        "verify/verify_support.rs",
                        0,
                        f"support verifier path does not contain `{needle}`",
                    )
                )
    run_cert = SRC / "verify" / "run_certificate.rs"
    if run_cert.exists():
        text = run_cert.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "pub input_hash: Hash",
            "pub canonical_system_hash: Hash",
            "pub target_variable: VariableId",
            "pub compression_hash: Hash",
            "pub hypergraph_hash: Hash",
            "pub target_projection_dag_hash: Hash",
            "pub kernel_plan_hashes: Vec<Hash>",
            "pub projection_message_hashes: Vec<Hash>",
            "pub global_support_hash: Option<Hash>",
            "pub squarefree_support_hash: Option<Hash>",
            "pub root_isolation_hash: Option<Hash>",
            "pub decoded_candidate_hash: Option<Hash>",
            "pub invariants: CoreInvariantFlags",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p14_run_certificate_binding_missing",
                        "verify/run_certificate.rs",
                        0,
                        f"run certificate does not contain `{needle}`",
                    )
                )
    replay = SRC / "verify" / "replay.rs"
    if replay.exists():
        text = replay.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "validate_input(problem.clone())",
            "canonicalize_system(validated)",
            "pre_kernel_compress(canonical.clone(), &mut ctx)",
            "replay_target_projection_dag(&compressed)",
            "compose_projection_messages(",
            "verify_global_support(support, &composed)",
            "hash_root_isolation(&result.root_isolation)",
            "hash_decoded_candidates(&result.decoded_candidates)",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p14_replay_recompute_missing",
                        "verify/replay.rs",
                        0,
                        f"replay path does not contain `{needle}`",
                    )
                )


def scan_p15_specific(findings: list[Finding]) -> None:
    squarefree = SRC / "roots" / "squarefree.rs"
    if squarefree.exists():
        text = squarefree.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "degree_uni(&normalized).is_none()",
            "zero support",
            "squarefree_part_uni(&normalized)",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p15_squarefree_contract_missing",
                        "roots/squarefree.rs",
                        0,
                        f"squarefree support path does not contain `{needle}`",
                    )
                )
    real_root = SRC / "algebra" / "real_root.rs"
    if real_root.exists():
        text = real_root.read_text(encoding="utf-8", errors="ignore")
        prod = strip_cfg_test_blocks(text)
        for needle in [
            "sturm_sequence",
            "root_count_between",
            "cauchy_bound",
            "isolate_real_roots_descartes",
            "isolate_descartes_interval",
            "descartes_variations_on_interval",
            "pow_linear",
            "convolve_coeffs",
            "sign_variations_coeffs",
        ]:
            if needle not in prod:
                findings.append(
                    Finding(
                        "error",
                        "p15_real_root_contract_missing",
                        "algebra/real_root.rs",
                        0,
                        f"real-root isolation path does not contain `{needle}`",
                    )
                )
        match = re.search(
            r"pub fn isolate_real_roots_descartes(?P<body>.*?)\nfn isolate_descartes_interval",
            prod,
            flags=re.DOTALL,
        )
        body = match.group("body") if match else ""
        if "isolate_real_roots_sturm" in body or "sturm_sequence" in body:
            findings.append(
                Finding(
                    "error",
                    "p15_descartes_aliases_sturm",
                    "algebra/real_root.rs",
                    0,
                    "Descartes/Vincent isolation delegates to Sturm",
                )
            )
        for forbidden in ["f64", "f32", "to_f64", "from_f64", "sqrt(", "..=128"]:
            if forbidden in prod:
                findings.append(
                    Finding(
                        "error",
                        "p15_float_or_fixed_cap_root_isolation",
                        "algebra/real_root.rs",
                        0,
                        f"root isolation production path contains forbidden marker `{forbidden}`",
                    )
                )
    isolate = SRC / "roots" / "isolate.rs"
    if isolate.exists():
        text = isolate.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "RootIsolationMethod::Sturm",
            "RootIsolationMethod::Descartes",
            "root.support_hash = squarefree.hash",
            "root.root_index = root_index",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p15_root_record_binding_missing",
                        "roots/isolate.rs",
                        0,
                        f"root isolation wrapper does not contain `{needle}`",
                    )
                )
    decode = SRC / "roots" / "decode.rs"
    if decode.exists():
        text = decode.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "target",
            "support.hash",
            "root.root_index",
            "isolating_interval",
            "hash_target_candidate",
            "rational_to_bytes(&isolating_interval.lo)",
            "rational_to_bytes(&isolating_interval.hi)",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p15_candidate_decode_binding_missing",
                        "roots/decode.rs",
                        0,
                        f"candidate decode path does not contain `{needle}`",
                    )
                )
    algebraic = SRC / "roots" / "algebraic_number.rs"
    if algebraic.exists():
        text = algebraic.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "root.support_hash != support.hash",
            "hash_algebraic_root",
            "refine_algebraic_root_to_width",
            "compare_algebraic_roots",
            "rational_to_bytes(&isolating_interval.lo)",
            "rational_to_bytes(&isolating_interval.hi)",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p15_algebraic_root_binding_missing",
                        "roots/algebraic_number.rs",
                        0,
                        f"algebraic root path does not contain `{needle}`",
                    )
                )
    sign = SRC / "algebra" / "sign.rs"
    if sign.exists():
        text = sign.read_text(encoding="utf-8", errors="ignore")
        for needle in ["sign_at_algebraic_root", "thom_encoding", "roots_in_interval"]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p15_sign_contract_missing",
                        "algebra/sign.rs",
                        0,
                        f"sign/Thom path does not contain `{needle}`",
                    )
                )


def scan_p16_specific(findings: list[Finding]) -> None:
    orchestrator = SRC / "solver" / "orchestrator.rs"
    if orchestrator.exists():
        text = orchestrator.read_text(encoding="utf-8", errors="ignore")
        prod = strip_cfg_test_blocks(text)
        for needle in [
            "ExactImageOutOfScope",
            "P16ExactImageScopeGuard",
            "exact_image_out_of_scope_diagnostic",
            "exact_image_out_of_scope_nonfinite_diagnostic",
            "SolverStatus::CertificateDesignGap",
            "exact-image classification is out of scope",
            "exact-image filtering is out of scope",
            "nonfinite_certificate_hash",
            "candidate_hashes",
            "input_hash",
            "support_hash",
            "squarefree_support_hash",
        ]:
            if needle not in prod:
                findings.append(
                    Finding(
                        "error",
                        "p16_scope_guard_missing",
                        "solver/orchestrator.rs",
                        0,
                        f"exact-image scope guard path does not contain `{needle}`",
                    )
                )
        for forbidden in [
            "classify_real_target_image",
            "classification.exact_candidates",
            "classification.exact_root_isolation",
            "SolverStatus::CertifiedExactTargetImage",
            "SolverStatus::CertifiedEmptyRealTargetImage",
        ]:
            if forbidden in prod:
                findings.append(
                    Finding(
                        "error",
                        "p16_reachable_exact_image_path",
                        "solver/orchestrator.rs",
                        0,
                        f"orchestrator production path still contains `{forbidden}`",
                    )
                )
    solver_files = [SRC / "solver" / name for name in ["orchestrator.rs", "pipeline.rs"]]
    for path in solver_files:
        if not path.exists():
            continue
        text = strip_cfg_test_blocks(path.read_text(encoding="utf-8", errors="ignore"))
        if "classify_real_target_image" in text:
            findings.append(
                Finding(
                    "error",
                    "p16_solver_calls_exact_image_classifier",
                    rel(path),
                    0,
                    "solver path still calls exact-image classifier",
                )
            )
    run_cert = SRC / "verify" / "run_certificate.rs"
    if run_cert.exists():
        text = run_cert.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "pub exact_image_certificate_hash: Option<Hash>",
            "exact_image_certificate_hash: input.exact_image_certificate_hash",
            "vec![options.exact_image_mode as u8]",
            "chunks.push(optional_hash_bytes(cert.exact_image_certificate_hash))",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p16_certificate_binding_missing",
                        "verify/run_certificate.rs",
                        0,
                        f"run certificate scope binding path does not contain `{needle}`",
                    )
                )
    replay = SRC / "verify" / "replay.rs"
    if replay.exists():
        text = replay.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "cert.exact_image_certificate_hash.is_some()",
            "return false",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p16_replay_scope_guard_missing",
                        "verify/replay.rs",
                        0,
                        f"replay exact-image rejection path does not contain `{needle}`",
                    )
                )


def scan_p17_specific(findings: list[Finding]) -> None:
    orchestrator = SRC / "solver" / "orchestrator.rs"
    if orchestrator.exists():
        full_text = strip_cfg_test_blocks(orchestrator.read_text(encoding="utf-8", errors="ignore"))
        start = full_text.find("pub fn solve_with_context")
        text = full_text[start:] if start >= 0 else full_text
        ordered_markers = [
            "step_validate(problem.clone(), &mut ctx)",
            "step_canonicalize(validated, &mut ctx)",
            "step_compress(canonical.clone(), &mut ctx)",
            "step_build_graphs(&compressed, &mut ctx)",
            "step_build_dag(&graphs, &compressed, &mut ctx)",
            "step_plan(&dag, &compressed, &mut ctx)",
            "step_execute(&dag, &plans, &compressed, &mut ctx)",
            "step_verify_messages(&dag, &messages, &compressed)",
            "step_compose(&dag, messages.clone(), target, &mut ctx)",
            "step_support(&composed, &compressed, target, &mut ctx)",
            "verify_global_support(&support, &composed)",
            "step_roots(&support, target, &mut ctx)",
            "step_core_certificate(",
            "step_cost_trace(",
            "Ok(finalize_success_result",
        ]
        cursor = -1
        for marker in ordered_markers:
            idx = text.find(marker)
            if idx < 0:
                findings.append(
                    Finding(
                        "error",
                        "p17_pipeline_stage_missing",
                        "solver/orchestrator.rs",
                        0,
                        f"orchestrator does not contain stage marker `{marker}`",
                    )
                )
                continue
            if idx <= cursor:
                findings.append(
                    Finding(
                        "error",
                        "p17_pipeline_stage_order",
                        "solver/orchestrator.rs",
                        0,
                        f"stage marker `{marker}` appears out of source order",
                    )
                )
            cursor = idx
        roots_idx = text.find("step_roots(&support, target, &mut ctx)")
        verify_idx = text.find("verify_global_support(&support, &composed)")
        if roots_idx >= 0 and verify_idx >= 0 and roots_idx < verify_idx:
            findings.append(
                Finding(
                    "error",
                    "p17_roots_before_support_verification",
                    "solver/orchestrator.rs",
                    0,
                    "root isolation appears before support verification",
                )
            )
        if "TargetSolveResult::from_solver_error_for_target_with_cost_trace" not in text:
            findings.append(
                Finding(
                    "error",
                    "p17_failure_finalizer_missing_cost_trace",
                    "solver/orchestrator.rs",
                    0,
                    "failure path does not preserve cost trace",
                )
            )
    pipeline = SRC / "solver" / "pipeline.rs"
    if pipeline.exists():
        text = pipeline.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "pub fn step_validate",
            "pub fn step_canonicalize",
            "pub fn step_compress",
            "pub fn step_build_graphs",
            "pub fn step_build_dag",
            "pub fn step_plan",
            "pub fn step_execute",
            "pub fn step_verify_messages",
            "pub fn step_compose",
            "pub fn step_support",
            "pub fn step_roots",
            "pub fn step_core_certificate",
            "pub fn step_cost_trace",
            "pub fn step_failure_cost_trace",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p17_step_function_missing",
                        "solver/pipeline.rs",
                        0,
                        f"pipeline does not expose `{needle}`",
                    )
                )
    cost = SRC / "result" / "cost_trace.rs"
    if cost.exists():
        text = cost.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "total_variable_count",
            "total_relation_count",
            "total_monomial_count",
            "max_total_degree",
            "max_coefficient_height_bits",
            "max_block_width",
            "max_separator_width",
            "local_variable_count",
            "local_relation_count",
            "estimated_quotient_rank",
            "matrix_rows",
            "matrix_cols",
            "coefficient_height_before_bits",
            "coefficient_height_after_bits",
            "final_support_degree",
            "certificate_size",
            "RouteCostTrace",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p17_cost_trace_field_missing",
                        "result/cost_trace.rs",
                        0,
                        f"cost trace does not contain `{needle}`",
                    )
                )
    run_cert = SRC / "verify" / "run_certificate.rs"
    if run_cert.exists():
        text = run_cert.read_text(encoding="utf-8", errors="ignore")
        for needle in [
            "vec![options.finite_candidate_cover_mode as u8]",
            "vec![options.exact_image_mode as u8]",
            "global_support_hash",
            "squarefree_support_hash",
            "root_isolation_hash",
            "decoded_candidate_hash",
            "final_dag_replay_evidence",
        ]:
            if needle not in text:
                findings.append(
                    Finding(
                        "error",
                        "p17_certificate_binding_missing",
                        "verify/run_certificate.rs",
                        0,
                        f"run certificate does not bind `{needle}`",
                    )
                )


def scan_descartes_alias(findings: list[Finding]) -> None:
    candidates = [SRC / "roots" / "isolate.rs", SRC / "algebra" / "real_root.rs"]
    for path in candidates:
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for match in re.finditer(r"fn\s+(isolate_real_roots_descartes[^{]*)\{", text):
            start = match.end()
            snippet = text[start : start + 1800]
            if re.search(r"\bsturm\b", snippet, flags=re.IGNORECASE):
                line_no = text[: match.start()].count("\n") + 1
                findings.append(
                    Finding(
                        "error",
                        "descartes_delegates_to_sturm",
                        rel(path),
                        line_no,
                        "Descartes/Vincent isolation appears to call or delegate to Sturm",
                    )
                )


def run_audit(phase: str | None) -> list[Finding]:
    findings: list[Finding] = []
    if not SRC.exists():
        findings.append(Finding("error", "missing_src_dir", "src", 0, "geosolver-core/src is missing"))
        return findings
    scan_required_files(findings, phase)
    scan_symbols(findings, phase)
    scan_forbidden_patterns(findings, phase)
    if phase == "P5":
        scan_p5_specific(findings)
    if phase == "P6":
        scan_p6_specific(findings)
    if phase == "P7":
        scan_p7_specific(findings)
    if phase == "P8":
        scan_p8_specific(findings)
    if phase == "P9":
        scan_p9_specific(findings)
    if phase == "P10":
        scan_p10_specific(findings)
    if phase == "P11":
        scan_p11_specific(findings)
    if phase == "P12":
        scan_p12_specific(findings)
    if phase == "P13":
        scan_p13_specific(findings)
    if phase == "P14":
        scan_p14_specific(findings)
    if phase == "P15":
        scan_p15_specific(findings)
        scan_descartes_alias(findings)
    if phase == "P16":
        scan_p16_specific(findings)
    if phase == "P17":
        scan_p17_specific(findings)
    if phase in (None, "ALL"):
        scan_descartes_alias(findings)
    return findings


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--strict", action="store_true", help="Exit nonzero if any finding is present")
    parser.add_argument("--json", action="store_true", help="Emit machine-readable JSON")
    parser.add_argument("--phase", choices=["P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8", "P9", "P10", "P11", "P12", "P13", "P14", "P15", "P16", "P17"], help="Restrict audit to a specific phase")
    args = parser.parse_args(argv)

    findings = run_audit(args.phase)
    if args.json:
        print(json.dumps([asdict(f) for f in findings], indent=2, sort_keys=True))
    else:
        print("RGDTPK-Q v4 finite candidate-cover static audit")
        print(f"root: {ROOT}")
        print(f"phase: {args.phase or 'ALL'}")
        print(f"production_rust_files: {len(production_rust_files())}")
        print(f"findings: {len(findings)}")
        for finding in findings:
            loc = f"{finding.path}:{finding.line}" if finding.line else finding.path
            print(f"{finding.severity.upper()} {finding.code} {loc} :: {finding.message}")
    if args.strict and findings:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
