from __future__ import annotations

from collections import Counter
from pathlib import Path
import re

ROOT = Path("docs/ai/changes/RGDTPK-Q-v4-core")
SRC = Path("geosolver-core/src")


def find_matching_brace(text: str, open_idx: int) -> int:
    depth = 0
    for idx in range(open_idx, len(text)):
        if text[idx] == "{":
            depth += 1
        elif text[idx] == "}":
            depth -= 1
            if depth == 0:
                return idx
    return len(text) - 1


def impl_contexts(text: str) -> list[tuple[int, int, str]]:
    contexts: list[tuple[int, int, str]] = []
    for match in re.finditer(r"(?m)^\s*impl(?:<[^\n{}]*>)?\s+([^\n{};]+?)\s*\{", text):
        header = re.sub(r"\s+", " ", match.group(1).strip())
        open_idx = text.find("{", match.start())
        close_idx = find_matching_brace(text, open_idx)
        if " for " in header:
            trait, typ = header.rsplit(" for ", 1)
            context = f"{typ.strip()}({trait.strip()})"
        else:
            context = header
        contexts.append((match.start(), close_idx, context))
    return contexts


def signature_at(text: str, start: int, name: str) -> tuple[str, list[str], str]:
    paren = 0
    seen_open = False
    end = min(len(text), start + 4000)
    for idx in range(start, end):
        char = text[idx]
        if char == "(":
            paren += 1
            seen_open = True
        elif char == ")":
            paren = max(0, paren - 1)
        elif char == "{" and seen_open and paren == 0:
            end = idx
            break
    sig = re.sub(r"\s+", " ", text[start:end].strip())
    after = sig.split(f"fn {name}", 1)[1]
    open_paren = after.find("(")
    depth = 0
    close_paren = len(after)
    for idx in range(open_paren, len(after)):
        if after[idx] == "(":
            depth += 1
        elif after[idx] == ")":
            depth -= 1
            if depth == 0:
                close_paren = idx
                break
    params_text = after[open_paren + 1 : close_paren]
    params: list[str] = []
    buf = ""
    depth = 0
    for char in params_text:
        if char in "([{<":
            depth += 1
        elif char in ")]}>":
            depth = max(0, depth - 1)
        if char == "," and depth == 0:
            if buf.strip():
                params.append(buf.strip())
            buf = ""
        else:
            buf += char
    if buf.strip():
        params.append(buf.strip())
    tail = after[close_paren + 1 :].strip()
    ret = "()"
    if "->" in tail:
        ret = tail.split("->", 1)[1].split(" where ", 1)[0].strip()
    return sig, params, ret


def extract_functions() -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for path in sorted(SRC.rglob("*.rs")):
        rel = path.relative_to(SRC).as_posix()
        text = path.read_text(encoding="utf-8")
        contexts = impl_contexts(text)
        for match in re.finditer(r"(?m)^\s*pub\s+(?:\([^)]*\)\s*)?fn\s+([A-Za-z0-9_]+)\s*\(", text):
            name = match.group(1)
            line = text[: match.start()].count("\n") + 1
            inner = [ctx for ctx in contexts if ctx[0] < match.start() < ctx[1]]
            ctx = inner[-1][2] if inner else None
            sig, params, ret = signature_at(text, match.start(), name)
            path_id = f"geosolver-core/src/{rel}::{ctx}::{name}" if ctx else f"geosolver-core/src/{rel}::{name}"
            rows.append({"rel": rel, "name": name, "line": line, "ctx": ctx, "path": path_id, "sig": sig, "params": params, "ret": ret})
    return rows


SEMANTIC_BY_PREFIX = [
    ("types/rational", ("Canonical rational arithmetic over normalized numerator/positive-denominator values", "Normalized RationalQ arithmetic result or exact DivisionByZero error")),
    ("types/monomial", ("Sparse monomial exponent vectors keyed by stable VariableId", "Canonical monomial/divisibility/variables/degree/order data")),
    ("types/polynomial", ("SparsePolynomialQ terms with RationalQ coefficients and VariableId monomials", "Canonical polynomial arithmetic, derivative, variable set, primitive form, or substitution")),
    ("types/univariate", ("UniPolynomialQ coefficient vectors over a single variable", "Normalized/gcd/derivative/evaluation/squarefree univariate data")),
    ("types/matrix", ("Sparse/dense modular or rational matrix rows", "Matrix shape/density/hash/vector data")),
    ("types/interval", ("Rational interval endpoints and exact RationalQ queries", "Validated interval or exact containment/disjointness predicate")),
    ("types/hash", ("Canonical byte chunks or typed solver data", "Stable Hash value for replay/certificate binding")),
    ("types/ids", ("Stable id counters, names, or namespaces", "Stable solver ids")),
    ("problem/input", ("Raw RationalTargetProblem variables, relations, semantics, and roles", "Problem record with deterministic input_hash")),
    ("problem/semantic", ("Real constraint/slack/branch semantic metadata", "Semantic encoding, referenced relation set, or InvalidInput check")),
    ("problem/validate", ("Raw Q-polynomial RationalTargetProblem and target declaration", "ValidatedProblem or InvalidInput without geometry dispatch")),
    ("problem/canonicalize", ("ValidatedProblem relations and variable/target ordering", "CanonicalSystemQ/CanonicalRelationQ/VariableOrder with hashes")),
    ("problem/context", ("SolverOptions, resource counters, stages, diagnostics", "SolverContext mutation, resource check, or diagnostic append")),
    ("algebra/monomial_order", ("Eliminate/keep variable blocks or ordered variables", "MonomialOrder for elimination/reduction")),
    ("algebra/polynomial_ops", ("SparsePolynomialQ reducers/targets and monomial order", "Leading term, S-polynomial, reduction, or primitive content")),
    ("algebra/modular", ("Q-polynomials or modular coefficients with prime modulus", "Prime choice, Fp reduction/lift, or modular arithmetic result")),
    ("algebra/crt", ("Integer residues/vectors and coprime moduli", "Canonical CRT-combined modular value/vector or rejection")),
    ("algebra/rational_reconstruction", ("Modular integers/polynomial data with height bounds", "RationalQ or SparsePolynomialQ reconstruction candidate")),
    ("algebra/sparse_matrix", ("Sparse modular matrix rows over Fp", "Sparse echelon/nullspace/rank results")),
    ("algebra/dense_matrix", ("Dense modular matrix rows over Fp", "Dense echelon/nullspace/rank results")),
    ("algebra/linear_solve", ("MatrixBuilder equations over modular primes and optional RHS", "Modular nullspace/solve result")),
    ("algebra/normal_form", ("Polynomial, basis, monomial order, and membership certificate", "Normal form polynomial or membership predicate")),
    ("algebra/groebner", ("Local Q-polynomial relation set and elimination order", "Elimination basis/generators or certified generators")),
    ("algebra/f4", ("Test/debug Groebner-backed batch-reduction inputs", "NotProductionF4-labelled reduction/local elimination result")),
    ("algebra/elimination", ("Local Q-polynomial relations plus eliminate/keep variables", "LocalEliminationResult with keep-variable generators and certificates")),
    ("algebra/resultant", ("Sparse resultant supports/templates and modular options", "Support sets, template, resultant relation, or certificate predicate")),
    ("algebra/interpolation", ("Specialization variables/points/samples and sparse support", "Specialized samples, interpolated relation, or interpolation certificate")),
    ("algebra/quotient", ("Finite quotient basis, authorized relations, action-column certificates", "TargetQuotientHandle, vectors, hashes, or exact normal-form/action certificates")),
    ("algebra/krylov", ("TargetQuotientHandle action vectors and Krylov data", "Krylov sequence, recurrence, coverage, or annihilator certificate")),
    ("algebra/regular_chain", ("Regular-chain input relations/components and kept variables", "RegularChainDAG or projected/combined generators")),
    ("algebra/norm_trace", ("Explicit algebraic tower relations and exports", "TowerDescription, norm relation, or norm verification predicate")),
    ("algebra/real_root", ("UniPolynomialQ for exact real-root algorithms", "Sturm/Descartes isolating data")),
    ("algebra/sign", ("UniPolynomialQ and exact RealRootRecord", "SignDetermination or ThomEncoding")),
    ("preprocess/compression", ("CanonicalSystemQ and compression state", "CompressedSystemQ/CompressionState after simplification passes")),
    ("preprocess/definitional", ("Canonical/compression relations scanned for definitions", "Definitional candidates or eliminated state")),
    ("preprocess/linear_affine", ("CompressionState affine candidates and pivot guards", "Affine pivot/order or guarded-substitution state")),
    ("preprocess/binomial", ("CompressionState relation set scanned for binomials", "Binomial candidates or simplified state")),
    ("preprocess/saturation", ("CompressionState plus explicit nonzero semantics", "Saturated state or explicit-nonzero predicate")),
    ("preprocess/independent", ("Compression components relative to target influence", "Target-independent marks or feasibility obligations")),
    ("graph/hypergraph", ("CompressedSystemQ relations and variables", "Relation-variable hypergraph/incidence/components")),
    ("graph/influence", ("RelationVariableHypergraph and target", "TargetInfluenceGraph")),
    ("graph/weighted_primal", ("CompressedSystemQ and target influence graph", "Weighted primal graph or algebraic weights")),
    ("graph/separators", ("WeightedPrimalGraph, target, and cost model", "Separator candidates or separator score")),
    ("graph/tree_decomposition", ("WeightedPrimalGraph, target, and cost model", "Target-rooted decomposition tree")),
    ("graph/projection_dag", ("Decomposition/graph bundle and compressed system", "TargetProjectionDAG, authorization hash, or validation result")),
    ("graph/metrics", ("ProjectionBlock and compressed system", "Structural metrics and algebraic estimates")),
    ("planner/cost_model", ("ProjectionBlock, KernelKind, and probes", "Kernel cost estimate or deterministic ordering")),
    ("planner/probes", ("ProjectionBlock, compressed system, primes, context", "Cost/rank/Macaulay/mixed-support probe results only")),
    ("planner/admission", ("ProjectionBlock and KernelContext for registered kernels", "KernelAdmission list and declared execution-plan candidates")),
    ("planner/kernel_plan", ("Kernel plan/execution metadata, support plans, resources, routes, failures, probes", "KernelPlan/KernelExecutionPlan/hash/support/failure/probe result")),
    ("planner/ladder", ("Kernel admissions and cost estimates", "Declared deterministic ladder of KernelExecutionPlan values")),
    ("planner/planner", ("TargetProjectionDAG, compressed system, context", "KernelPlan per block or implementation error")),
    ("kernels/target_univariate", ("ProjectionBlock with relation variables inside target/export set", "Principal target support ProjectionMessage")),
    ("kernels/linear_affine", ("Triangular affine local variables with constant/guarded pivots", "Affine-elimination ProjectionMessage or hard case")),
    ("kernels/target_relation_search", ("Local ideal J with eliminated/exported variables and schedule", "Verified export relation from modular nullspace and membership checks")),
    ("kernels/sparse_resultant", ("Low-dimensional/pair-chain sparse-resultant template class", "ProjectionMessage relation from current resultant trace")),
    ("kernels/action_krylov", ("Target-only or alias/local-univariate quotient-action slice", "Characteristic/annihilator ProjectionMessage for that slice")),
    ("kernels/universal_elimination", ("Q-polynomial projection block in current staged elimination plan", "ProjectionMessage from current bounded local strategy")),
    ("kernels/regular_chain_projection", ("Triangular/regular-chain-like current recognizer input", "ProjectionMessage with current chain projection generators")),
    ("kernels/norm_trace_projection", ("Explicit algebraic tower detected by current recognizer", "Norm-trace ProjectionMessage for detected tower")),
    ("kernels/specialization_interpolation", ("Separator-specialization samples with inner target relation search", "Interpolated ProjectionMessage after current verification")),
    ("kernels/traits", ("Kernel trait objects, KernelKind, KernelContext, messages", "Kernel interface, admission, plan, execute, or replay value")),
    ("kernels/mod", ("Kernel registry request or KernelKind", "Registered kernel object list or selected kernel")),
    ("compose/message", ("ProjectionMessage relation/certificate/cost data", "ProjectionMessage hash/representation/strength")),
    ("compose/compose", ("TargetProjectionDAG and ProjectionMessage list", "ComposedProjection/message ideal from current composition")),
    ("compose/separator_elimination", ("Message relation polynomials plus separator/target variables", "Separator-eliminated support relation candidates")),
    ("compose/final_support", ("ComposedProjection, target, nonfinite witness/certificate, context", "Support polynomial, limited nonfinite result, or candidate-cover result without CoreRunCertificate")),
    ("verify/certificates", ("KernelExecutionPlan, ProjectionMessage payload, certificate metadata", "KernelCertificate/binding hash; synthetic helper is test-only")),
    ("verify/verify_message", ("ProjectionMessage and actual KernelContext/block authorization", "Current generator/certificate verification Result")),
    ("verify/verify_support", ("Support polynomial, ComposedProjection, context", "GlobalSupportCertificate/hash or verification result")),
    ("verify/replay", ("TargetSolveResult and original problem", "ReplayResult from recomputed stages plus synthetic relation-order replay checks")),
    ("verify/run_certificate", ("Canonical/compressed/DAG/message/root/candidate evidence", "CoreRunCertificate/hash/invariant flags from current evidence, not final proof")),
    ("roots/squarefree", ("UniPolynomialQ support polynomial", "Exact squarefree support via gcd/derivative")),
    ("roots/isolate", ("UniPolynomialQ and RootIsolationOptions", "Exact rational isolating RealRootRecord intervals")),
    ("roots/decode", ("Target, squarefree support, isolated roots", "TargetCandidate list with hashes/intervals")),
    ("roots/algebraic_number", ("Support polynomial/root record data", "Algebraic number record/hash/comparison predicate")),
    ("fiber/", ("Exact-image P13 placeholder module scope", "No candidate-cover output; exact-image semantics deferred")),
    ("result/status", ("SolverError/SolverStatus construction inputs", "Closed status/failure/error value")),
    ("result/diagnostics", ("Diagnostic message/stage data", "DiagnosticRecord")),
    ("result/cost_trace", ("Compression/projection/composition/root/certificate cost fields", "Cost trace updates and hashes")),
    ("result/output", ("FinalizeSuccessInput/FinalizeFailureInput and solver errors", "TargetSolveResult success/failure object")),
    ("solver/options", ("User solver options/root/certificate/resource knobs", "SolverOptions/defaults")),
    ("solver/pipeline", ("Problem/canonical stage values and SolverContext", "Currently validate/canonicalize only; later stages missing")),
    ("solver/orchestrator", ("Public RationalTargetProblem and SolverContext", "temporary_pipeline_not_connected error, not pipeline result")),
    ("api", ("Public RationalTargetProblem and SolverOptions", "TargetSolveResult from orchestrator mapping")),
]


def semantics(rel: str) -> tuple[str, str]:
    for prefix, value in SEMANTIC_BY_PREFIX:
        if rel.startswith(prefix):
            return value
    return ("Typed inputs shown in signature", "Typed output shown in signature")


def role(rel: str, name: str) -> str:
    if rel.startswith("problem/") or name in {"solve_target", "solve_with_context"}:
        return "validate"
    if rel.startswith("preprocess/"):
        return "compress"
    if rel.startswith("graph/") or rel.startswith("planner/"):
        return "plan"
    if rel.startswith("kernels/") or rel.startswith("algebra/"):
        return "execute"
    if rel.startswith("compose/") or rel.startswith("verify/"):
        return "verify"
    if rel.startswith("roots/"):
        return "root"
    if rel.startswith("result/") or rel.startswith("solver/"):
        return "finalize"
    return "validate"


HIGH_FILES = {
    "planner/kernel_plan.rs",
    "planner/planner.rs",
    "planner/admission.rs",
    "planner/ladder.rs",
    "kernels/action_krylov.rs",
    "kernels/sparse_resultant.rs",
    "kernels/specialization_interpolation.rs",
    "kernels/regular_chain_projection.rs",
    "kernels/norm_trace_projection.rs",
    "kernels/universal_elimination.rs",
    "kernels/target_relation_search.rs",
    "compose/final_support.rs",
    "compose/compose.rs",
    "compose/separator_elimination.rs",
    "verify/replay.rs",
    "verify/verify_message.rs",
    "verify/verify_support.rs",
    "verify/run_certificate.rs",
    "verify/certificates.rs",
}


def production_reachable(rel: str, name: str) -> bool:
    if rel.startswith("fiber/") or rel == "algebra/f4.rs":
        return False
    return not ("for_test" in name or "for_tests" in name or "debug" in name or name.startswith("test_"))


def risk_action(rel: str, name: str, ctx: str | None) -> tuple[str, str]:
    if name in {"solve_target", "solve_with_context"}:
        return "fatal", "replace"
    if rel == "algebra/f4.rs":
        return "high", "move_to_test"
    if "for_test" in name or "for_tests" in name or "debug" in name or name.startswith("test_"):
        return "medium", "move_to_test"
    if rel in HIGH_FILES:
        if rel == "verify/replay.rs" or (rel == "planner/kernel_plan.rs" and name == "new" and ctx == "KernelExecutionPlan"):
            return "high", "replace"
        return "high", "generalize"
    if rel.startswith("preprocess/") or rel in {"graph/projection_dag.rs", "graph/tree_decomposition.rs", "graph/separators.rs"}:
        return "medium", "generalize"
    if rel.startswith("fiber/"):
        return "medium", "keep"
    if rel.startswith(("roots/", "problem/", "types/", "algebra/", "graph/", "result/", "solver/")):
        return "low", "keep"
    return "none", "keep"


def plan_time(rel: str, name: str) -> str:
    if name.startswith("plan_") and rel in {
        "kernels/sparse_resultant.rs",
        "kernels/specialization_interpolation.rs",
        "kernels/regular_chain_projection.rs",
        "kernels/norm_trace_projection.rs",
        "kernels/action_krylov.rs",
    }:
        return "final_relation_construction"
    if name in {"build_sparse_resultant_trace", "build_specialization_interpolation_trace"}:
        return "final_relation_construction"
    if "probe" in name:
        return "cost_probe_only"
    return "none"


def certificate_binding(rel: str, name: str, prod: bool) -> str:
    if not prod:
        return "missing"
    if rel == "verify/replay.rs":
        return "missing"
    if rel in {"verify/run_certificate.rs", "verify/certificates.rs", "verify/verify_message.rs", "verify/verify_support.rs"}:
        return "decorative"
    if rel == "compose/final_support.rs":
        return "missing" if name.startswith("finalize_") else "decorative"
    if rel.startswith(("kernels/", "compose/", "planner/")):
        return "decorative"
    if name.startswith(("verify_", "certify_")):
        return "decorative"
    if rel.startswith(("algebra/", "roots/")):
        return "exact"
    return "missing"


def limitations(rel: str, name: str, ctx: str | None, risk: str) -> list[str]:
    result: list[str] = []
    if risk in {"fatal", "high"}:
        result.append("FCR repair required before candidate-cover readiness")
    if rel == "verify/replay.rs":
        result.append("replay reconstructs relation-order blocks instead of actual DAG block evidence")
    if rel == "compose/final_support.rs":
        result.append("finalizers do not attach CoreRunCertificate and nonfinite proof kind is limited")
    if rel == "algebra/f4.rs":
        result.append("NotProductionF4/test helper only")
    if rel == "kernels/action_krylov.rs":
        result.append("target-only or alias/local-univariate quotient-action slice")
    if rel == "kernels/sparse_resultant.rs":
        result.append("pair/template resultant class and plan-time relation trace")
    if rel == "kernels/norm_trace_projection.rs":
        result.append("explicit tower recognizer only")
    if rel == "planner/kernel_plan.rs" and name == "new" and ctx == "KernelExecutionPlan":
        result.append("KernelExecutionPlan::new defaults to PurePlan unless disambiguated by impl context")
    elif rel == "planner/kernel_plan.rs" and name == "new" and ctx == "KernelPlan":
        result.append("KernelPlan::new selects the first declared ladder entry and depends on ladder purity")
    if rel.startswith("fiber/"):
        result.append("P13 exact-image scope, not candidate-cover proof")
    return result or ["No known FCR-P1 blocker beyond later semantic review"]


def esc(value: object) -> str:
    return str(value).replace("|", "\\|").replace("\n", " ")


def function_rows(functions: list[dict[str, object]]) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for fn in functions:
        rel = str(fn["rel"])
        name = str(fn["name"])
        ctx = fn["ctx"] if isinstance(fn["ctx"], str) else None
        input_sem, output_sem = semantics(rel)
        prod = production_reachable(rel, name)
        risk, action = risk_action(rel, name, ctx)
        rows.append(
            {
                "path": fn["path"],
                "source_line": f"{rel}:{fn['line']}",
                "production_reachable": prod,
                "algorithmic_role": role(rel, name),
                "actual_input_class": f"{input_sem}; signature inputs: {'; '.join(fn['params']) if fn['params'] else 'none'}",
                "actual_output_class": f"{output_sem}; return: {fn['ret']}",
                "known_limitations": limitations(rel, name, ctx, risk),
                "partial_slice_risk": risk,
                "plan_time_execution": plan_time(rel, name),
                "certificate_binding": certificate_binding(rel, name, prod),
                "required_action": action,
            }
        )
    return rows


KERNEL_TRAIT_METHODS = [
    ("kind", "plan"),
    ("admit", "plan"),
    ("plan", "plan"),
    ("execute", "execute"),
    ("replay", "verify"),
]


def trait_impl_method_lines(rel: str, kernel: str) -> dict[str, int]:
    text = (SRC / rel).read_text(encoding="utf-8")
    match = re.search(
        rf"(?m)^\s*impl\s+TargetProjectionKernel\s+for\s+{re.escape(kernel)}\s*\{{",
        text,
    )
    if not match:
        raise RuntimeError(f"missing TargetProjectionKernel impl for {kernel} in {rel}")
    open_idx = text.find("{", match.start())
    close_idx = find_matching_brace(text, open_idx)
    block = text[open_idx:close_idx]
    lines: dict[str, int] = {}
    for method, _role in KERNEL_TRAIT_METHODS:
        method_match = re.search(rf"(?m)^\s*fn\s+{method}\s*\(", block)
        if not method_match:
            raise RuntimeError(f"missing TargetProjectionKernel::{method} for {kernel} in {rel}")
        lines[method] = text.count("\n", 0, open_idx + method_match.start()) + 1
    return lines


def production_path_rows() -> list[dict[str, object]]:
    base = [
        ("geosolver-core/src/solver/orchestrator.rs::temporary_pipeline_not_connected path", True, "validate", "Any public solve_with_context invocation.", "Immediate temporary error; no solver pipeline.", ["No candidate-cover pipeline executes."], "fatal", "none", "missing", "replace"),
        ("geosolver-core/src/algebra/f4.rs::NotProductionF4 and *_for_tests", False, "execute", "Test/debug Groebner-backed F4-like helpers.", "Non-production results labelled NotProductionF4.", ["Not Appendix A production F4."], "high", "none", "missing", "move_to_test"),
        ("geosolver-core/src/kernels/action_krylov.rs::target-only quotient-action path", True, "execute", "Target-only univariate quotient-action relation.", "Characteristic/annihilator message for target-only slice.", ["Cannot stand in for generic TargetActionKrylov."], "high", "final_relation_construction", "decorative", "generalize"),
        ("geosolver-core/src/kernels/action_krylov.rs::alias-univariate quotient-action path", True, "execute", "Local univariate relation plus target alias.", "Characteristic/annihilator message for alias slice.", ["Current non-target coverage remains alias/univariate shaped."], "high", "final_relation_construction", "decorative", "generalize"),
        ("geosolver-core/src/kernels/sparse_resultant.rs::plan-time resultant trace", True, "plan", "Pair/template sparse resultant candidate class.", "Plan support hash and trace based on computed relation.", ["Plan-time final relation construction; binary/pair limitation."], "high", "final_relation_construction", "decorative", "generalize"),
        ("geosolver-core/src/kernels/specialization_interpolation.rs::plan-time inner TargetRelationSearch", True, "plan", "Specialization samples where inner target-only relation search admits.", "Sample relation traces used to interpolate relation.", ["Plan executes inner support-producing computation."], "high", "final_relation_construction", "decorative", "generalize"),
        ("geosolver-core/src/kernels/specialization_interpolation.rs::local Groebner verification", True, "verify", "Interpolated relation and local elimination result.", "Verification against local generated elimination data.", ["Local verification cannot replace full declared proof."], "high", "final_relation_construction", "decorative", "generalize"),
        ("geosolver-core/src/kernels/regular_chain_projection.rs::plan-time chain decomposition/projection", True, "plan", "Triangular/regular-chain-like current recognizer input.", "Projected generators embedded in trace/template support.", ["Plan-time output construction and limited class."], "high", "final_relation_construction", "decorative", "generalize"),
        ("geosolver-core/src/kernels/norm_trace_projection.rs::plan-time tower/norm construction", True, "plan", "Explicit tower forms detected by current recognizer.", "Norm relation embedded in trace/support.", ["Explicit-tower-only path."], "high", "final_relation_construction", "decorative", "generalize"),
        ("geosolver-core/src/verify/replay.rs::synthetic all-relations replay", True, "verify", "TargetSolveResult and original problem; reconstructed compressed relation-order block context.", "ReplayResult without actual TargetProjectionDAG block replay.", ["Not exact DAG replay."], "high", "none", "missing", "replace"),
        ("geosolver-core/src/compose/final_support.rs::limited nonfinite proof kind", True, "verify", "ComposedProjection lacking target-only support plus optional witness.", "CertifiedNonFiniteTargetImage may be emitted with certificate: None::<CoreRunCertificate>.", ["Positive final run proof incomplete."], "high", "none", "missing", "generalize"),
        ("geosolver-core/src/planner/kernel_plan.rs::KernelExecutionPlan::new default PurePlan path", True, "plan", "Kernel execution plan metadata without explicit work classification.", "KernelExecutionPlan with PlanWorkClassification::PurePlan.", ["PurePlan can hide plan-time output construction."], "high", "none", "missing", "replace"),
    ]
    rows = [
        {"path": p, "source_line": "", "production_reachable": pr, "algorithmic_role": r, "actual_input_class": i, "actual_output_class": o, "known_limitations": l, "partial_slice_risk": risk, "plan_time_execution": pt, "certificate_binding": cb, "required_action": a}
        for p, pr, r, i, o, l, risk, pt, cb, a in base
    ]
    kernels = [
        ("kernels/target_univariate.rs", "TargetUnivariateKernel", "medium", False),
        ("kernels/linear_affine.rs", "LinearAffineKernel", "medium", False),
        ("kernels/target_relation_search.rs", "TargetRelationSearchKernel", "high", False),
        ("kernels/sparse_resultant.rs", "SparseResultantProjectionKernel", "high", True),
        ("kernels/action_krylov.rs", "TargetActionKrylovKernel", "high", True),
        ("kernels/universal_elimination.rs", "UniversalTargetEliminationKernel", "high", False),
        ("kernels/regular_chain_projection.rs", "RegularChainProjectionKernel", "high", True),
        ("kernels/norm_trace_projection.rs", "NormTraceProjectionKernel", "high", True),
        ("kernels/specialization_interpolation.rs", "SpecializationInterpolationKernel", "high", True),
    ]
    for rel, kernel, risk, plan_constructs_relation in kernels:
        method_lines = trait_impl_method_lines(rel, kernel)
        for method, role_name in KERNEL_TRAIT_METHODS:
            rows.append(
                {
                    "path": f"geosolver-core/src/{rel}::{kernel}(TargetProjectionKernel)::{method}",
                    "source_line": f"{rel}:{method_lines[method]}",
                    "production_reachable": True,
                    "algorithmic_role": role_name,
                    "actual_input_class": f"Concrete TargetProjectionKernel::{method} dispatch in {rel} for projection block/context/message.",
                    "actual_output_class": f"{kernel} {method} result routed through the production kernel registry.",
                    "known_limitations": [f"Concrete trait dispatch must be replay-bound with the matching {kernel} implementation and certificate semantics."],
                    "partial_slice_risk": risk,
                    "plan_time_execution": "final_relation_construction" if method == "plan" and plan_constructs_relation else "none",
                    "certificate_binding": "decorative",
                    "required_action": "generalize",
                }
            )
    return rows


def render(functions: list[dict[str, object]], rows: list[dict[str, object]], path_rows: list[dict[str, object]]) -> str:
    all_rows = rows + path_rows
    risk_counts = Counter(row["partial_slice_risk"] for row in all_rows)
    action_counts = Counter(row["required_action"] for row in all_rows)
    plan_counts = Counter(row["plan_time_execution"] for row in all_rows)
    cert_counts = Counter(row["certificate_binding"] for row in all_rows)
    lines = [
        "# Full Core Production Audit",
        "",
        "Purpose: FCR-P1 direct audit of current P1-P12G production code. This file records actual current implementation classes and required repair actions; it is not implementation readiness evidence.",
        "",
        "Claim ceiling after this audit remains `PARTIAL_MECHANISM_READY:MECH-011`. `CANDIDATE_COVER_CORE_READY`, exact-image readiness, source-fidelity, and acceptance completion remain forbidden.",
        "",
        "## Scope",
        "",
        "- Audited source roots: `types`, `problem`, `algebra`, `preprocess`, `graph`, `planner`, `kernels`, `compose`, `verify`, `roots`, `result`, and `solver`.",
        "- Public function/method rows use exact `file::ImplType::method` identity when the function is declared inside an `impl` block.",
        "- `actual_input_class` and `actual_output_class` include both the current semantic class and the extracted Rust signature inputs/return type.",
        "- Rows marked `generalize`, `replace`, or `move_to_test` are not closed by documentation; FCR-P2+ must change production reachability or semantics.",
        "",
        "## Summary Counts",
        "",
        "| category | counts |",
        "|---|---|",
        f"| public function/method rows | {len(functions)} |",
        f"| production path rows | {len(path_rows)} |",
        "| partial_slice_risk | " + ", ".join(f"{key}={risk_counts[key]}" for key in ["fatal", "high", "medium", "low", "none"] if risk_counts[key]) + " |",
        "| required_action | " + ", ".join(f"{key}={value}" for key, value in sorted(action_counts.items())) + " |",
        "| plan_time_execution | " + ", ".join(f"{key}={value}" for key, value in sorted(plan_counts.items())) + " |",
        "| certificate_binding | " + ", ".join(f"{key}={value}" for key, value in sorted(cert_counts.items())) + " |",
        "",
        "## Mandatory And Production Path Findings",
        "",
        "| path | source_line | production_reachable | algorithmic_role | actual_input_class | actual_output_class | known_limitations | partial_slice_risk | plan_time_execution | certificate_binding | required_action |",
        "|---|---|---:|---|---|---|---|---|---|---|---|",
    ]
    for row in path_rows:
        lines.append("| " + " | ".join([esc(row["path"]), esc(row["source_line"]), str(row["production_reachable"]).lower(), row["algorithmic_role"], esc(row["actual_input_class"]), esc(row["actual_output_class"]), esc("; ".join(row["known_limitations"])), row["partial_slice_risk"], row["plan_time_execution"], row["certificate_binding"], row["required_action"]]) + " |")
    lines.extend(["", "## Public Function And Method Audit", "", "| path | source_line | production_reachable | algorithmic_role | actual_input_class | actual_output_class | known_limitations | partial_slice_risk | plan_time_execution | certificate_binding | required_action |", "|---|---|---:|---|---|---|---|---|---|---|---|"])
    for row in rows:
        lines.append("| " + " | ".join([esc(row["path"]), esc(row["source_line"]), str(row["production_reachable"]).lower(), row["algorithmic_role"], esc(row["actual_input_class"]), esc(row["actual_output_class"]), esc("; ".join(row["known_limitations"])), row["partial_slice_risk"], row["plan_time_execution"], row["certificate_binding"], row["required_action"]]) + " |")
    lines.extend([
        "",
        "## Required Repair Rules",
        "",
        "- `partial_slice_risk: fatal`: public candidate-cover pipeline cannot claim progress until the row is replaced and replay-bound.",
        "- `partial_slice_risk: high`: FCR-P2 must either generalize, replace, delete, or move the row out of production reachability. Keeping a limited production path with documentation is not allowed.",
        "- `plan_time_execution: final_relation_construction`: FCR-P4 must make planning pure or prove the row is removed from production planning.",
        "- `certificate_binding: decorative|missing` on production-reachable rows blocks candidate-cover readiness until exact DAG/certificate replay evidence replaces it.",
        "- `move_to_test` rows must be behind test-only reachability or removed from production modules before later readiness claims.",
    ])
    return "\n".join(lines) + "\n"


def main() -> None:
    functions = extract_functions()
    rows = function_rows(functions)
    path_rows = production_path_rows()
    (ROOT / "FULL_CORE_PRODUCTION_AUDIT.md").write_text(render(functions, rows, path_rows), encoding="utf-8")
    evidence = ROOT / "evidence/FCR-P1"
    evidence.mkdir(parents=True, exist_ok=True)
    (evidence / "production_function_list.tsv").write_text(
        "\n".join(f"{fn['rel']}\t{fn['line']}\t{fn['ctx'] or ''}\t{fn['name']}\t{fn['sig']}" for fn in functions) + "\n",
        encoding="utf-8",
    )
    all_rows = rows + path_rows
    print(f"wrote audit rows={len(all_rows)} public={len(functions)} production_paths={len(path_rows)}")
    for label, counter in [
        ("risk_counts", Counter(row["partial_slice_risk"] for row in all_rows)),
        ("action_counts", Counter(row["required_action"] for row in all_rows)),
        ("plan_time_counts", Counter(row["plan_time_execution"] for row in all_rows)),
        ("certificate_counts", Counter(row["certificate_binding"] for row in all_rows)),
    ]:
        print(label + "=" + ", ".join(f"{key}={value}" for key, value in sorted(counter.items())))


if __name__ == "__main__":
    main()
