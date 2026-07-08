use std::collections::BTreeSet;

use crate::candidate_direct::DirectTargetEquationOracle;
use crate::candidate_krylov::TargetCyclicKrylovOracle;
use crate::candidate_residual::ResidualCyclicOracle;
use crate::candidate_resultant::HiddenVariableSparseResultantOracle;
use crate::candidate_slice::SliceSpecializationOracle;
use crate::candidate_tower::NormTraceTowerOracle;
use crate::candidates::{CandidateOracle, CandidateOrigin, TargetCandidate};
use crate::compression::{
    certified_system_from_problem, lift_multipliers_to_original_problem, CertifiedSystemQ,
};
use crate::dependency_dag::{build_target_dependency_dag, plan_certificate_windows};
use crate::exact_image::{classify_real_fibers_conservative, ExactImageClassification};
use crate::fallback_elimination::{
    complete_target_elimination_fallback, try_empty_admissible_set_certificate,
    CompleteFallbackResult,
};
use crate::normalize::{factor_schedule, normalize_candidates, rank_candidates};
use crate::proof::{prove_fixed_target, CertificateMode, FixedProofInput, ProofFailure};
use crate::proof_learning::{
    expand_by_obstruction_predecessors, expand_by_total_degree, learn_initial_proof_window,
};
use crate::proof_schedule::{bounded_fair_proof_prefix, certificate_mode_for_trial};
use crate::repair_multiple::low_degree_multiple_repair;
use crate::repair_schur::{localized_schur_repair, SchurRepairOutput};
use crate::roots::isolate_real_roots_squarefree;
use crate::window::{CertificateWindow, ProofWindow};
use crate::{
    AlgebraicRealRoot, CertifiedExactTargetImage, CompositeRule, EmptyAdmissibleSetCertificate,
    ExactImageMode, ResourceLimits, SolverCertificate, SolverOptions, SolverTrace,
    TargetCertificate, TargetProblemQ, UniPolynomialQ,
};

#[derive(Clone, Debug)]
pub struct TargetSolveResult {
    pub status: SolverStatus,
    pub cover: Option<CertifiedCandidateCover>,
    pub exact_image: Option<CertifiedExactTargetImage>,
    pub certificate: Option<SolverCertificate>,
    pub trace: SolverTrace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolverStatus {
    CertifiedCandidateCover,
    CertifiedExactTargetImage,
    CertifiedEmptyAdmissibleSet,
    CertifiedNoNonzeroTargetEliminant,
    NoVerifiedTargetCertificate,
    FiniteResourceFailure,
    CertificateDesignGap,
    InvalidInput,
    ImplementationBug,
}

#[derive(Clone, Debug)]
pub struct CertifiedCandidateCover {
    pub support: UniPolynomialQ,
    pub squarefree_support: UniPolynomialQ,
    pub real_roots: Vec<AlgebraicRealRoot>,
    pub certificate: TargetCertificate,
}

pub fn solve_target(problem: TargetProblemQ, options: SolverOptions) -> TargetSolveResult {
    solve_target_with_route_control(problem, options, None, true)
}

fn solve_target_with_route_control(
    problem: TargetProblemQ,
    options: SolverOptions,
    enabled_origins: Option<&BTreeSet<CandidateOrigin>>,
    allow_complete_fallback: bool,
) -> TargetSolveResult {
    solve_target_with_route_control_inner(
        problem,
        options,
        enabled_origins,
        allow_complete_fallback,
        false,
    )
}

fn solve_target_with_route_control_inner(
    problem: TargetProblemQ,
    options: SolverOptions,
    enabled_origins: Option<&BTreeSet<CandidateOrigin>>,
    allow_complete_fallback: bool,
    tamper_first_candidate: bool,
) -> TargetSolveResult {
    let system = match certified_system_from_problem(&problem) {
        Ok(system) => system,
        Err(_) => {
            return TargetSolveResult {
                status: SolverStatus::InvalidInput,
                cover: None,
                exact_image: None,
                certificate: None,
                trace: SolverTrace::default(),
            };
        }
    };
    if !problem.is_well_formed() {
        return TargetSolveResult {
            status: SolverStatus::InvalidInput,
            cover: None,
            exact_image: None,
            certificate: None,
            trace: SolverTrace::default(),
        };
    }

    let mut trace = SolverTrace::default();

    if let Some(certificate) =
        try_empty_admissible_set_certificate(&system, &options.resource_limits)
    {
        let Some(certificate) = lift_empty_certificate_to_original(&problem, certificate) else {
            return implementation_bug_result(trace);
        };
        trace.events.push("early_empty:certified".to_string());
        return TargetSolveResult {
            status: SolverStatus::CertifiedEmptyAdmissibleSet,
            cover: None,
            exact_image: None,
            certificate: Some(SolverCertificate::EmptyAdmissibleSet(certificate)),
            trace,
        };
    }

    if options.resource_limits.max_proof_weight.is_none() {
        trace
            .events
            .push("resource:unbounded_proof_requires_bound".to_string());
        return finite_resource_failure(trace);
    }

    let dag = build_target_dependency_dag(&system);
    let bounded_windows = plan_certificate_windows(&system, &dag, &options.resource_limits);
    let window_source: Box<dyn Iterator<Item = CertificateWindow> + '_> =
        Box::new(bounded_windows.into_iter());
    let candidate_limit = options
        .resource_limits
        .max_candidate_count
        .unwrap_or(usize::MAX);
    if candidate_limit == 0 {
        trace.events.push("resource:candidate_limit".to_string());
        return finite_resource_failure(trace);
    }
    let mut candidate_count = 0;
    let mut verified = Vec::new();
    let mut collected_obstructions = Vec::new();

    for window in window_source {
        trace
            .events
            .push(format!("window:degree={}", window.target_degree));
        if window_exceeds_limits(&window, &options.resource_limits) {
            trace.events.push(format!(
                "resource:window:rows={}:cols={}",
                window.row_monomials.len(),
                window
                    .multiplier_supports
                    .iter()
                    .map(Vec::len)
                    .sum::<usize>()
            ));
            return finite_resource_failure(trace);
        }

        let candidates = collect_candidate_routes(&system, &window, enabled_origins);
        let candidates = rank_candidates(normalize_candidates(candidates));
        #[cfg(test)]
        let candidates = {
            let mut candidates = candidates;
            if tamper_first_candidate {
                shift_reconstructed_candidates_for_test(&mut candidates);
            }
            candidates
        };
        let mut last_proof_window = None;

        for candidate in candidates {
            if candidate_count >= candidate_limit {
                trace.events.push("resource:candidate_limit".to_string());
                return finite_resource_failure(trace);
            }
            candidate_count += 1;
            trace.events.push(candidate_trace_event(&candidate));
            if candidate.reconstructed.is_some() {
                let factor_trials = factor_schedule(&candidate);
                trace.events.push(factorization_trace_event(&factor_trials));
                for proof_candidate in factor_trials.candidates {
                    trace.events.push(proof_try_trace_event(&proof_candidate));
                    if let Some(certificate) = try_candidate_certificate(
                        &system,
                        &window,
                        &proof_candidate,
                        &options.resource_limits,
                        &mut collected_obstructions,
                        &mut last_proof_window,
                        &mut trace,
                        tamper_first_candidate,
                    ) {
                        let Some(certificate) =
                            lift_target_certificate_to_original(&problem, certificate)
                        else {
                            return implementation_bug_result(trace);
                        };
                        return return_verified_cover(&mut verified, certificate, &options, trace);
                    }
                }
            }
        }

        if !tamper_first_candidate {
            if let Some(proof_window) = last_proof_window.as_ref() {
                match localized_schur_repair(
                    &system,
                    proof_window,
                    &collected_obstructions,
                    &options.resource_limits,
                ) {
                    SchurRepairOutput::Certified(certificate) => {
                        trace.events.push("localized_schur:certified".to_string());
                        let Some(certificate) =
                            lift_target_certificate_to_original(&problem, certificate)
                        else {
                            return implementation_bug_result(trace);
                        };
                        return return_verified_cover(&mut verified, certificate, &options, trace);
                    }
                    SchurRepairOutput::SupportInformation(_) => {
                        trace.events.push("localized_schur:support".to_string());
                    }
                    SchurRepairOutput::NoLocalScope => {}
                }
            }
        }
    }

    if !allow_complete_fallback && candidate_count > 0 {
        trace.events.push("route_forcing:no_verified".to_string());
        return TargetSolveResult {
            status: SolverStatus::NoVerifiedTargetCertificate,
            cover: None,
            exact_image: None,
            certificate: None,
            trace,
        };
    }
    assert!(
        allow_complete_fallback,
        "complete fallback reached while disabled"
    );
    match complete_target_elimination_fallback(&system, &options.resource_limits) {
        CompleteFallbackResult::CertifiedEmpty(certificate) => {
            let Some(certificate) = lift_empty_certificate_to_original(&problem, certificate)
            else {
                return implementation_bug_result(trace);
            };
            trace.events.push("target_elimination:empty".to_string());
            return TargetSolveResult {
                status: SolverStatus::CertifiedEmptyAdmissibleSet,
                cover: None,
                exact_image: None,
                certificate: Some(SolverCertificate::EmptyAdmissibleSet(certificate)),
                trace,
            };
        }
        CompleteFallbackResult::CertifiedNoTargetEliminant(_) => {
            trace
                .events
                .push("target_elimination:no_target_eliminant".to_string());
            return TargetSolveResult {
                status: SolverStatus::CertificateDesignGap,
                cover: None,
                exact_image: None,
                certificate: None,
                trace,
            };
        }
        CompleteFallbackResult::CertifiedSupport(certificate) => {
            let Some(certificate) = lift_target_certificate_to_original(&problem, certificate)
            else {
                return implementation_bug_result(trace);
            };
            trace.events.push("target_elimination:support".to_string());
            return maybe_classify_exact_target_image(certificate, &options, trace);
        }
        CompleteFallbackResult::ResourceFailure(cost) => {
            trace.events.push(format!(
                "target_elimination:resource:rows={}:cols={}",
                cost.matrix_rows, cost.matrix_cols
            ));
        }
    }

    let status = match options.exact_image_mode {
        ExactImageMode::CoverOnly
        | ExactImageMode::TryExactImage
        | ExactImageMode::RequireExactImage => SolverStatus::NoVerifiedTargetCertificate,
    };

    TargetSolveResult {
        status,
        cover: None,
        exact_image: None,
        certificate: None,
        trace,
    }
}

fn finite_resource_failure(trace: SolverTrace) -> TargetSolveResult {
    TargetSolveResult {
        status: SolverStatus::FiniteResourceFailure,
        cover: None,
        exact_image: None,
        certificate: None,
        trace,
    }
}

fn implementation_bug_result(trace: SolverTrace) -> TargetSolveResult {
    TargetSolveResult {
        status: SolverStatus::ImplementationBug,
        cover: None,
        exact_image: None,
        certificate: None,
        trace,
    }
}

fn window_exceeds_limits(window: &CertificateWindow, limits: &ResourceLimits) -> bool {
    limits
        .max_matrix_rows
        .is_some_and(|max_rows| window.row_monomials.len() > max_rows)
        || limits.max_matrix_cols.is_some_and(|max_cols| {
            window
                .multiplier_supports
                .iter()
                .map(Vec::len)
                .sum::<usize>()
                > max_cols
        })
}

#[cfg(test)]
pub(crate) fn solve_target_for_test(
    problem: TargetProblemQ,
    options: SolverOptions,
    forcing: &crate::test_support::TestRouteForcing,
) -> TargetSolveResult {
    solve_target_with_route_forcing(problem, options, forcing)
}

#[cfg(test)]
pub(crate) fn solve_target_with_route_forcing(
    problem: TargetProblemQ,
    options: SolverOptions,
    forcing: &crate::test_support::RouteForcing,
) -> TargetSolveResult {
    if let Some(seed) = forcing.forced_localized_schur.as_ref() {
        return solve_target_with_forced_localized_schur(problem, options, forcing, seed);
    }
    let enabled_origins = if forcing.allow_other_heavy_routes {
        None
    } else {
        Some(&forcing.enabled_origins)
    };
    solve_target_with_route_control_inner(
        problem,
        options,
        enabled_origins,
        forcing.allow_complete_fallback,
        forcing.tamper_first_candidate,
    )
}

#[cfg(test)]
fn solve_target_with_forced_localized_schur(
    problem: TargetProblemQ,
    options: SolverOptions,
    forcing: &crate::test_support::RouteForcing,
    seed: &crate::test_support::ForcedLocalizedSchur,
) -> TargetSolveResult {
    let system = match certified_system_from_problem(&problem) {
        Ok(system) => system,
        Err(_) => {
            return TargetSolveResult {
                status: SolverStatus::InvalidInput,
                cover: None,
                exact_image: None,
                certificate: None,
                trace: SolverTrace::default(),
            };
        }
    };
    let proof_window = ProofWindow {
        multiplier_supports: seed.proof_multiplier_supports.clone(),
    };
    let obstruction = crate::proof_learning::LeftNullObstruction {
        row_monomials: seed.obstruction_rows.clone(),
        coefficients: seed.obstruction_coefficients.clone(),
    };
    let mut trace = SolverTrace::default();
    match localized_schur_repair(
        &system,
        &proof_window,
        &[obstruction],
        &options.resource_limits,
    ) {
        SchurRepairOutput::Certified(certificate) => {
            trace.events.push("localized_schur:certified".to_string());
            let Some(certificate) = lift_target_certificate_to_original(&problem, certificate)
            else {
                return implementation_bug_result(trace);
            };
            return return_verified_cover(&mut Vec::new(), certificate, &options, trace);
        }
        SchurRepairOutput::SupportInformation(_) => {
            trace.events.push("localized_schur:support".to_string());
        }
        SchurRepairOutput::NoLocalScope => {}
    }
    if !forcing.allow_complete_fallback {
        return TargetSolveResult {
            status: SolverStatus::NoVerifiedTargetCertificate,
            cover: None,
            exact_image: None,
            certificate: None,
            trace,
        };
    }
    solve_target_with_route_control(problem, options, None, true)
}

fn try_candidate_certificate(
    system: &CertifiedSystemQ,
    window: &CertificateWindow,
    candidate: &TargetCandidate,
    limits: &ResourceLimits,
    collected_obstructions: &mut Vec<crate::proof_learning::LeftNullObstruction>,
    last_proof_window: &mut Option<ProofWindow>,
    trace: &mut SolverTrace,
    proof_gate_only_for_test: bool,
) -> Option<TargetCertificate> {
    let support = candidate.reconstructed.as_ref()?;
    if is_nonzero_constant_support(support) {
        trace
            .events
            .push("proof_constant_support:defer_empty".to_string());
        return None;
    }

    let proof_window = learn_initial_proof_window(window, &candidate.traces);
    *last_proof_window = Some(proof_window.clone());

    let max_weight = limits.max_proof_weight?;
    for trial in bounded_fair_proof_prefix(max_weight) {
        let scheduled_window =
            expand_by_total_degree(system, &proof_window, trial.tuple.multiplier_degree);
        *last_proof_window = Some(scheduled_window.clone());
        if let Some(certificate) = try_fixed_certificate(
            system,
            support,
            &scheduled_window,
            certificate_mode_for_trial(&trial),
            limits,
            collected_obstructions,
            trace,
        ) {
            return Some(certificate);
        }
    }

    if proof_gate_only_for_test {
        return None;
    }

    match low_degree_multiple_repair(system, support, &proof_window, limits) {
        Ok(certificate)
            if !is_nonzero_constant_support(target_certificate_support(&certificate)) =>
        {
            Some(certificate)
        }
        Ok(_) | Err(_) => None,
    }
}

fn try_fixed_certificate(
    system: &CertifiedSystemQ,
    support: &UniPolynomialQ,
    proof_window: &ProofWindow,
    mode: CertificateMode,
    limits: &ResourceLimits,
    collected_obstructions: &mut Vec<crate::proof_learning::LeftNullObstruction>,
    trace: &mut SolverTrace,
) -> Option<TargetCertificate> {
    let mut current_window = proof_window.clone();
    let expansion_limit = limits.max_window_degree.map(|degree| degree.max(1));
    let mut expansion_count = 0;

    loop {
        let input = FixedProofInput {
            system: system.clone(),
            candidate: support.clone(),
            proof_window: current_window.clone(),
            certificate_mode: mode.clone(),
        };
        match prove_fixed_target(input) {
            Ok(certificate) => return Some(certificate),
            Err(ProofFailure::Inconsistent { obstruction }) => {
                if expansion_limit.is_some_and(|limit| expansion_count >= limit) {
                    return None;
                }
                expansion_count += 1;
                trace.events.push("proof_obstruction".to_string());
                let expanded =
                    expand_by_obstruction_predecessors(system, &current_window, &obstruction);
                collected_obstructions.push(obstruction);
                if expanded == current_window {
                    return None;
                }
                current_window = expanded;
            }
            Err(
                ProofFailure::InvalidInput
                | ProofFailure::IdentityCheckFailed
                | ProofFailure::NoCertificateFound,
            ) => return None,
        }
    }
}

fn return_verified_cover(
    verified: &mut Vec<TargetCertificate>,
    certificate: TargetCertificate,
    options: &SolverOptions,
    trace: SolverTrace,
) -> TargetSolveResult {
    verified.push(certificate);
    let Some(final_certificate) = refine_and_finalize(verified) else {
        return TargetSolveResult {
            status: SolverStatus::ImplementationBug,
            cover: None,
            exact_image: None,
            certificate: None,
            trace,
        };
    };
    maybe_classify_exact_target_image(final_certificate, options, trace)
}

fn lift_target_certificate_to_original(
    problem: &TargetProblemQ,
    certificate: TargetCertificate,
) -> Option<TargetCertificate> {
    match certificate {
        TargetCertificate::IdealMembership {
            support,
            multipliers,
            identity,
        } => Some(TargetCertificate::IdealMembership {
            support,
            multipliers: lift_multipliers_to_original_problem(problem, &multipliers)?,
            identity,
        }),
        TargetCertificate::RadicalMembership {
            support,
            power,
            multipliers,
            identity,
        } => Some(TargetCertificate::RadicalMembership {
            support,
            power,
            multipliers: lift_multipliers_to_original_problem(problem, &multipliers)?,
            identity,
        }),
        TargetCertificate::GuardedRadicalMembership {
            support,
            support_power,
            guard_power,
            guard_product,
            guard_certificates,
            multipliers,
            identity,
        } => Some(TargetCertificate::GuardedRadicalMembership {
            support,
            support_power,
            guard_power,
            guard_product,
            guard_certificates,
            multipliers: lift_multipliers_to_original_problem(problem, &multipliers)?,
            identity,
        }),
        TargetCertificate::CompositeCover {
            support,
            children,
            rule,
            component_union_source,
        } => {
            let children = children
                .into_iter()
                .map(|child| lift_target_certificate_to_original(problem, child))
                .collect::<Option<Vec<_>>>()?;
            Some(TargetCertificate::CompositeCover {
                support,
                children,
                rule,
                component_union_source,
            })
        }
    }
}

fn lift_empty_certificate_to_original(
    problem: &TargetProblemQ,
    certificate: EmptyAdmissibleSetCertificate,
) -> Option<EmptyAdmissibleSetCertificate> {
    match certificate {
        EmptyAdmissibleSetCertificate::AlgebraicInfeasibility {
            multipliers,
            identity,
        } => Some(EmptyAdmissibleSetCertificate::AlgebraicInfeasibility {
            multipliers: lift_multipliers_to_original_problem(problem, &multipliers)?,
            identity,
        }),
        EmptyAdmissibleSetCertificate::GuardedAlgebraicInfeasibility {
            guard_product,
            guard_power,
            guard_certificates,
            multipliers,
            identity,
        } => Some(
            EmptyAdmissibleSetCertificate::GuardedAlgebraicInfeasibility {
                guard_product,
                guard_power,
                guard_certificates,
                multipliers: lift_multipliers_to_original_problem(problem, &multipliers)?,
                identity,
            },
        ),
        EmptyAdmissibleSetCertificate::RealInfeasibility { certificate } => {
            Some(EmptyAdmissibleSetCertificate::RealInfeasibility { certificate })
        }
    }
}

fn refine_and_finalize(verified: &[TargetCertificate]) -> Option<TargetCertificate> {
    let mut certificates = verified.iter();
    let first = certificates.next()?.clone();
    let mut support = target_certificate_support(&first).clone();
    let mut children = vec![first];
    for certificate in certificates {
        support = support.gcd(target_certificate_support(certificate));
        children.push(certificate.clone());
    }

    if children.len() == 1 {
        children.into_iter().next()
    } else {
        Some(TargetCertificate::CompositeCover {
            support: support.primitive_integer_normalized(),
            children,
            rule: CompositeRule::SameIdealGcd,
            component_union_source: None,
        })
    }
}

fn maybe_classify_exact_target_image(
    certificate: TargetCertificate,
    options: &SolverOptions,
    mut trace: SolverTrace,
) -> TargetSolveResult {
    match options.exact_image_mode {
        ExactImageMode::CoverOnly => target_cover_result(certificate, trace),
        ExactImageMode::TryExactImage => {
            let cover = candidate_cover_from_certificate(certificate);
            match classify_real_fibers_conservative(
                cover.certificate.clone(),
                cover.support.clone(),
                cover.squarefree_support.clone(),
                cover.real_roots.clone(),
            ) {
                ExactImageClassification::Complete(image) => exact_image_result(image, trace),
                ExactImageClassification::Incomplete { unclassified_roots } => {
                    trace.events.push(format!(
                        "exact_image:incomplete:roots={}",
                        unclassified_roots.len()
                    ));
                    target_cover_from_cover(cover, trace)
                }
            }
        }
        ExactImageMode::RequireExactImage => {
            let cover = candidate_cover_from_certificate(certificate);
            match classify_real_fibers_conservative(
                cover.certificate,
                cover.support,
                cover.squarefree_support,
                cover.real_roots,
            ) {
                ExactImageClassification::Complete(image) => exact_image_result(image, trace),
                ExactImageClassification::Incomplete { unclassified_roots } => {
                    trace.events.push(format!(
                        "exact_image:incomplete:roots={}",
                        unclassified_roots.len()
                    ));
                    TargetSolveResult {
                        status: SolverStatus::NoVerifiedTargetCertificate,
                        cover: None,
                        exact_image: None,
                        certificate: None,
                        trace,
                    }
                }
            }
        }
    }
}

fn target_cover_result(certificate: TargetCertificate, trace: SolverTrace) -> TargetSolveResult {
    target_cover_from_cover(candidate_cover_from_certificate(certificate), trace)
}

fn candidate_cover_from_certificate(certificate: TargetCertificate) -> CertifiedCandidateCover {
    let support = target_certificate_support(&certificate).clone();
    let squarefree_support = support.squarefree_part();
    let real_roots = isolate_real_roots_squarefree(&squarefree_support);
    CertifiedCandidateCover {
        squarefree_support,
        support,
        real_roots,
        certificate,
    }
}

fn target_cover_from_cover(
    cover: CertifiedCandidateCover,
    trace: SolverTrace,
) -> TargetSolveResult {
    let certificate = cover.certificate.clone();
    TargetSolveResult {
        status: SolverStatus::CertifiedCandidateCover,
        cover: Some(cover),
        exact_image: None,
        certificate: Some(SolverCertificate::TargetCover(certificate)),
        trace,
    }
}

fn exact_image_result(image: CertifiedExactTargetImage, trace: SolverTrace) -> TargetSolveResult {
    let certificate = image.certificate.clone();
    TargetSolveResult {
        status: SolverStatus::CertifiedExactTargetImage,
        cover: None,
        exact_image: Some(image),
        certificate: Some(SolverCertificate::ExactTargetImage(certificate)),
        trace,
    }
}

fn is_nonzero_constant_support(support: &UniPolynomialQ) -> bool {
    support.degree() == Some(0)
}

fn target_certificate_support(certificate: &TargetCertificate) -> &UniPolynomialQ {
    match certificate {
        TargetCertificate::IdealMembership { support, .. }
        | TargetCertificate::RadicalMembership { support, .. }
        | TargetCertificate::GuardedRadicalMembership { support, .. }
        | TargetCertificate::CompositeCover { support, .. } => support,
    }
}

fn collect_candidate_routes(
    system: &CertifiedSystemQ,
    window: &CertificateWindow,
    enabled_origins: Option<&BTreeSet<CandidateOrigin>>,
) -> Vec<TargetCandidate> {
    let mut candidates = Vec::new();
    if origin_enabled(enabled_origins, CandidateOrigin::DirectTargetEquation) {
        let oracle = DirectTargetEquationOracle;
        candidates.extend(oracle.generate(system, window));
    }
    if origin_enabled(enabled_origins, CandidateOrigin::ResidualCyclic) {
        let oracle = ResidualCyclicOracle {
            primes: vec![5, 7, 11, 13],
        };
        candidates.extend(oracle.generate(system, window));
    }
    if origin_enabled(enabled_origins, CandidateOrigin::NormTraceTower) {
        let oracle = NormTraceTowerOracle;
        candidates.extend(oracle.generate(system, window));
    }
    if origin_enabled(enabled_origins, CandidateOrigin::TargetCyclicKrylov) {
        let oracle = TargetCyclicKrylovOracle;
        candidates.extend(oracle.generate(system, window));
    }
    if origin_enabled(
        enabled_origins,
        CandidateOrigin::HiddenVariableSparseResultant,
    ) {
        let oracle = HiddenVariableSparseResultantOracle {
            primes: vec![5, 7, 11, 13],
        };
        candidates.extend(oracle.generate(system, window));
    }
    if origin_enabled(enabled_origins, CandidateOrigin::SliceSpecialization) {
        let oracle = SliceSpecializationOracle {
            primes: vec![5, 7, 11, 13],
            slice_count: window.target_degree.saturating_add(1),
        };
        candidates.extend(oracle.generate(system, window));
    }
    candidates
}

fn origin_enabled(
    enabled_origins: Option<&BTreeSet<CandidateOrigin>>,
    origin: CandidateOrigin,
) -> bool {
    enabled_origins.is_none_or(|origins| origins.contains(&origin))
}

fn candidate_trace_event(candidate: &TargetCandidate) -> String {
    format!(
        "candidate:{:?}:degree={}:origins={}",
        candidate.origin,
        candidate_degree(candidate),
        candidate.origin_evidence.len()
    )
}

fn proof_try_trace_event(candidate: &TargetCandidate) -> String {
    format!(
        "proof_try:{:?}:degree={}:origins={}",
        candidate.origin,
        candidate_degree(candidate),
        candidate.origin_evidence.len()
    )
}

fn factorization_trace_event(schedule: &crate::normalize::FactorTrialSchedule) -> String {
    format!(
        "factorization:{:?}:searched_degrees={}:divisors={}:failure={:?}",
        schedule.status,
        schedule.trace.searched_factor_degrees.len(),
        schedule.trace.divisor_enumerations,
        schedule.failure
    )
}

fn candidate_degree(candidate: &TargetCandidate) -> usize {
    if let Some(reconstructed) = &candidate.reconstructed {
        return reconstructed.degree().unwrap_or(usize::MAX);
    }
    candidate
        .support_mod_primes
        .iter()
        .filter_map(|support| {
            support
                .coefficients
                .iter()
                .rposition(|coefficient| *coefficient != 0)
        })
        .min()
        .unwrap_or(usize::MAX)
}

#[cfg(test)]
fn shift_reconstructed_candidates_for_test(candidates: &mut [TargetCandidate]) {
    for candidate in candidates {
        let Some(support) = candidate.reconstructed.as_mut() else {
            continue;
        };
        if support.coefficients.is_empty() {
            support.coefficients.push(crate::arith::rational_one());
        } else {
            support.coefficients[0] += crate::arith::rational_one();
        }
        support.normalize();
    }
}

#[cfg(test)]
pub(crate) fn collect_candidates_for_test(
    problem: TargetProblemQ,
    limits: &ResourceLimits,
    forcing: &crate::test_support::TestRouteForcing,
) -> Vec<TargetCandidate> {
    if !problem.is_well_formed() {
        return Vec::new();
    }
    let _fallback_allowed = forcing.allow_complete_fallback;
    let enabled_origins = if forcing.allow_other_heavy_routes {
        None
    } else {
        Some(&forcing.enabled_origins)
    };
    let Ok(system) = certified_system_from_problem(&problem) else {
        return Vec::new();
    };
    let dag = build_target_dependency_dag(&system);
    plan_certificate_windows(&system, &dag, limits)
        .iter()
        .flat_map(|window| collect_candidate_routes(&system, window, enabled_origins))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::*;
    use crate::candidates::CandidateTrace;
    use crate::window::make_row_closed_certificate_window;
    use crate::{
        verify_certificate, ExactIdentity, ExactIdentityKind, GuardRecord, Monomial, PolynomialQ,
        Rational, Variable, VerificationResult,
    };

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    fn rational(value: i64) -> Rational {
        BigRational::from_integer(BigInt::from(value))
    }

    fn monomial(exponents: &[u32]) -> Monomial {
        Monomial {
            exponents: exponents.to_vec(),
        }
    }

    fn term(variables: &[Variable], coefficient: i64, exponents: &[u32]) -> PolynomialQ {
        PolynomialQ::from_term(
            variables.to_vec(),
            rational(coefficient),
            monomial(exponents),
        )
    }

    fn polynomial(variables: &[Variable], terms: &[(i64, Vec<u32>)]) -> PolynomialQ {
        terms
            .iter()
            .fold(PolynomialQ::zero(variables.to_vec()), |sum, entry| {
                sum.add(&term(variables, entry.0, &entry.1))
            })
    }

    fn uni(variable: &Variable, coefficients: &[i64]) -> UniPolynomialQ {
        let mut polynomial = UniPolynomialQ {
            variable: variable.clone(),
            coefficients: coefficients.iter().map(|value| rational(*value)).collect(),
        };
        polynomial.normalize();
        polynomial
    }

    fn problem(
        equations: Vec<PolynomialQ>,
        variables: Vec<Variable>,
        target: Variable,
    ) -> TargetProblemQ {
        TargetProblemQ {
            equations,
            variables,
            target,
            semantic_guards: Vec::<GuardRecord>::new(),
        }
    }

    fn options(mode: ExactImageMode) -> SolverOptions {
        SolverOptions {
            resource_limits: ResourceLimits {
                max_window_degree: Some(2),
                max_proof_weight: Some(2),
                max_matrix_rows: None,
                max_matrix_cols: None,
                max_candidate_count: None,
            },
            exact_image_mode: mode,
        }
    }

    #[test]
    fn same_ideal_gcd_refinement_builds_verified_composite() {
        let t = variable("T");
        let variables = vec![t.clone()];
        let s1 = uni(&t, &[0, -1, 1]);
        let s2 = uni(&t, &[2, -3, 1]);
        let equations = vec![
            s1.to_multivariate(&variables),
            s2.to_multivariate(&variables),
        ];
        let identity = ExactIdentity {
            kind: ExactIdentityKind::IdealMembership,
        };
        let cert1 = TargetCertificate::IdealMembership {
            support: s1,
            multipliers: vec![
                PolynomialQ::one(variables.clone()),
                PolynomialQ::zero(variables.clone()),
            ],
            identity: identity.clone(),
        };
        let cert2 = TargetCertificate::IdealMembership {
            support: s2,
            multipliers: vec![
                PolynomialQ::zero(variables.clone()),
                PolynomialQ::one(variables.clone()),
            ],
            identity,
        };

        let certificate = refine_and_finalize(&[cert1, cert2]).unwrap();

        assert_eq!(
            target_certificate_support(&certificate).primitive_integer_normalized(),
            uni(&t, &[-1, 1])
        );
        assert_eq!(
            verify_certificate(
                problem(equations, variables, t),
                SolverCertificate::TargetCover(certificate)
            ),
            VerificationResult::Verified
        );
    }

    #[test]
    fn require_exact_image_mode_fails_closed_without_classifier() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
        ];

        let result = solve_target(
            problem(equations, variables, t),
            options(ExactImageMode::RequireExactImage),
        );

        assert_eq!(result.status, SolverStatus::NoVerifiedTargetCertificate);
        assert!(result.cover.is_none());
        assert!(result.certificate.is_none());
        assert!(result
            .trace
            .events
            .iter()
            .any(|event| event.starts_with("exact_image:incomplete")));
    }

    #[test]
    fn bounded_window_resource_failure_does_not_reach_fallback() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![polynomial(&variables, &[(1, vec![1, 0])])];
        let options = SolverOptions {
            resource_limits: ResourceLimits {
                max_window_degree: Some(1),
                max_proof_weight: Some(1),
                max_matrix_rows: Some(0),
                max_matrix_cols: None,
                max_candidate_count: None,
            },
            exact_image_mode: ExactImageMode::CoverOnly,
        };

        let result = solve_target(problem(equations, variables, t), options);

        assert_eq!(result.status, SolverStatus::FiniteResourceFailure);
        assert!(result.cover.is_none());
        assert!(result.certificate.is_none());
        assert!(result
            .trace
            .events
            .iter()
            .any(|event| event.starts_with("resource:window:")));
        assert!(!result
            .trace
            .events
            .iter()
            .any(|event| event.starts_with("target_elimination:")));
    }

    #[test]
    fn max_window_degree_none_does_not_insert_hidden_default_window_bound() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![polynomial(&variables, &[(1, vec![1, 0])])];
        let options = SolverOptions {
            resource_limits: ResourceLimits {
                max_window_degree: None,
                max_proof_weight: Some(1),
                max_matrix_rows: None,
                max_matrix_cols: None,
                max_candidate_count: None,
            },
            exact_image_mode: ExactImageMode::CoverOnly,
        };

        let result = solve_target(problem(equations, variables, t), options);

        assert_eq!(result.status, SolverStatus::NoVerifiedTargetCertificate);
        assert!(result.cover.is_none());
        assert!(result.certificate.is_none());
        assert!(!result
            .trace
            .events
            .iter()
            .any(|event| event.starts_with("window:")));
        assert!(!result
            .trace
            .events
            .iter()
            .any(|event| event.starts_with("proof_try:")));
        assert!(result
            .trace
            .events
            .iter()
            .any(|event| event.starts_with("target_elimination:resource:")));
        assert!(!result
            .trace
            .events
            .iter()
            .any(|event| event == "target_elimination:no_target_eliminant"));
    }

    #[test]
    fn solve_target_without_proof_bound_does_not_silently_use_default_six() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
        ];
        let options = SolverOptions {
            resource_limits: ResourceLimits {
                max_window_degree: Some(2),
                max_proof_weight: None,
                max_matrix_rows: None,
                max_matrix_cols: None,
                max_candidate_count: None,
            },
            exact_image_mode: ExactImageMode::CoverOnly,
        };

        let result = solve_target(problem(equations, variables, t), options);

        assert_eq!(result.status, SolverStatus::FiniteResourceFailure);
        assert!(result.cover.is_none());
        assert!(result.certificate.is_none());
        assert!(result
            .trace
            .events
            .iter()
            .any(|event| event == "resource:unbounded_proof_requires_bound"));
        assert!(!result
            .trace
            .events
            .iter()
            .any(|event| event.starts_with("target_elimination:")));
    }

    #[test]
    fn origin_count_does_not_certify_candidate_without_exact_proof() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let input = problem(
            vec![polynomial(&variables, &[(1, vec![1, 0])])],
            variables.clone(),
            t.clone(),
        );
        let system = certified_system_from_problem(&input).unwrap();
        let window = make_row_closed_certificate_window(&system, 1, vec![vec![monomial(&[0, 0])]]);
        let candidate = TargetCandidate {
            support_mod_primes: Vec::new(),
            reconstructed: Some(uni(&t, &[-1, 1])),
            origin: CandidateOrigin::DirectTargetEquation,
            origin_evidence: BTreeSet::from([
                CandidateOrigin::DirectTargetEquation,
                CandidateOrigin::NormTraceTower,
            ]),
            traces: vec![CandidateTrace::DirectEquation { equation_index: 0 }],
        };
        let limits = ResourceLimits {
            max_window_degree: Some(1),
            max_proof_weight: Some(1),
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        };
        let mut collected_obstructions = Vec::new();
        let mut last_proof_window = None;
        let mut trace = SolverTrace::default();

        let certificate = try_candidate_certificate(
            &system,
            &window,
            &candidate,
            &limits,
            &mut collected_obstructions,
            &mut last_proof_window,
            &mut trace,
            false,
        );

        assert!(certificate.is_none());
    }
}
