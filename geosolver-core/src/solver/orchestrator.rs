use crate::compose::compose::ComposedProjection;
use crate::compose::message::ProjectionMessage;
use crate::graph::projection_dag::TargetProjectionDAG;
use crate::preprocess::compression::CompressedSystemQ;
use crate::problem::context::SolverContext;
use crate::problem::input::RationalTargetProblem;
use crate::result::diagnostics::DiagnosticRecord;
use crate::result::output::{finalize_success_result, FinalizeSuccessInput, TargetSolveResult};
use crate::result::status::{SolverError, SolverStatus, StageId};
use crate::solver::options::SolverOptions;
use crate::solver::pipeline::{
    finalize_nonfinite_pipeline_result, step_build_dag, step_build_graphs, step_canonicalize,
    step_compose, step_compress, step_core_certificate, step_cost_trace, step_execute,
    step_failure_cost_trace, step_plan, step_roots, step_support, step_validate,
    step_verify_messages,
};
use crate::types::ids::VariableId;
use crate::types::univariate::UniPolynomialQ;
use crate::verify::run_certificate::CoreRunCertificate;

pub fn solve_target(problem: RationalTargetProblem, options: SolverOptions) -> TargetSolveResult {
    let target = problem.target;
    let ctx = SolverContext::new(options);
    match solve_with_context(problem, ctx) {
        Ok(result) => result,
        Err(err) => TargetSolveResult::from_solver_error_for_target(err, target),
    }
}

pub fn solve_with_context(
    problem: RationalTargetProblem,
    mut ctx: SolverContext,
) -> Result<TargetSolveResult, SolverError> {
    let target = problem.target;
    let validated = step_validate(problem.clone(), &mut ctx)?;
    let canonical = step_canonicalize(validated, &mut ctx)?;
    let compressed = step_compress(canonical.clone(), &mut ctx)?;
    let graphs = step_build_graphs(&compressed, &mut ctx)?;
    let dag = step_build_dag(&graphs, &compressed, &mut ctx)?;
    let plans = match step_plan(&dag, &compressed, &mut ctx) {
        Ok(plans) => plans,
        Err(err) => {
            return Ok(finalize_pipeline_error(
                err,
                target,
                &compressed,
                &dag,
                &[],
                None,
                None,
                None,
            ))
        }
    };
    let messages = match step_execute(&dag, &plans, &compressed, &mut ctx) {
        Ok(messages) => messages,
        Err(err) => {
            return Ok(finalize_pipeline_error(
                err,
                target,
                &compressed,
                &dag,
                &[],
                None,
                None,
                None,
            ))
        }
    };
    if let Err(err) = step_verify_messages(&dag, &messages, &compressed) {
        return Ok(finalize_pipeline_error(
            err,
            target,
            &compressed,
            &dag,
            &messages,
            None,
            None,
            None,
        ));
    }
    let composed = match step_compose(&dag, messages.clone(), target, &mut ctx) {
        Ok(composed) => composed,
        Err(err) => {
            return Ok(finalize_pipeline_error(
                err,
                target,
                &compressed,
                &dag,
                &messages,
                None,
                None,
                None,
            ))
        }
    };
    let support_outcome = match step_support(&composed, &compressed, target, &mut ctx) {
        Ok(support_outcome) => support_outcome,
        Err(err) => {
            return Ok(finalize_pipeline_error(
                err,
                target,
                &compressed,
                &dag,
                &messages,
                Some(&composed),
                None,
                None,
            ))
        }
    };

    match support_outcome {
        crate::compose::final_support::FinalSupportComputation::Support(support) => {
            let support_certificate =
                match crate::verify::verify_support::verify_global_support(&support, &composed) {
                    Ok(support_certificate) => support_certificate,
                    Err(err) => {
                        return Ok(finalize_pipeline_error(
                            err,
                            target,
                            &compressed,
                            &dag,
                            &messages,
                            Some(&composed),
                            Some(&support),
                            None,
                        ))
                    }
                };
            let roots = match step_roots(&support, target, &mut ctx) {
                Ok(roots) => roots,
                Err(err) => {
                    return Ok(finalize_pipeline_error(
                        err,
                        target,
                        &compressed,
                        &dag,
                        &messages,
                        Some(&composed),
                        Some(&support),
                        None,
                    ))
                }
            };
            let certificate = step_core_certificate(
                &problem,
                &ctx.options,
                &canonical,
                &compressed,
                &graphs,
                &dag,
                &plans,
                &messages,
                Some(&support),
                &roots,
                None,
                Some(&support_certificate),
            );
            let cost_trace = step_cost_trace(
                &compressed,
                &dag,
                &messages,
                Some(&composed),
                Some(&support),
                Some(&certificate),
            );
            let mut diagnostics = compressed.diagnostics.clone();
            diagnostics.extend(ctx.diagnostics.clone());
            if ctx.options.exact_image_mode {
                diagnostics.push(exact_image_out_of_scope_diagnostic(
                    &problem, &support, &roots,
                ));
                diagnostics.push(DiagnosticRecord::new(
                    "CandidateCoverMayContainSpuriousRoots",
                    "exact-image filtering is out of scope; returned candidates are an unfiltered certified candidate cover"
                        .to_owned(),
                    Some(StageId("P16ExactImageScopeGuard".to_owned())),
                ));
            } else {
                diagnostics.push(DiagnosticRecord::new(
                    "ExactImageFilteringNotRequested",
                    "candidate-cover mode returns roots of S(T) without exact-image filtering"
                        .to_owned(),
                    Some(StageId("P13CandidateCover".to_owned())),
                ));
                diagnostics.push(DiagnosticRecord::new(
                    "CandidateCoverMayContainSpuriousRoots",
                    "certified candidate cover proves true target values are contained in roots(S); extra roots are allowed"
                        .to_owned(),
                    Some(StageId("P13CandidateCover".to_owned())),
                ));
            }
            if roots.root_isolation.is_empty() {
                diagnostics.push(DiagnosticRecord::new(
                    "EmptyRealCandidateCover",
                    "support has no real roots; certified candidate cover is empty".to_owned(),
                    Some(StageId("P12RootDecode".to_owned())),
                ));
            }
            let result = TargetSolveResult {
                status: if ctx.options.exact_image_mode {
                    SolverStatus::CertificateDesignGap
                } else {
                    SolverStatus::CertifiedCandidateCover
                },
                target,
                support_polynomial: Some(support),
                squarefree_support_polynomial: Some(roots.squarefree_support),
                root_isolation: roots.root_isolation,
                decoded_candidates: roots.decoded_candidates,
                projection_messages: messages,
                certificate: Some(certificate),
                diagnostics,
                cost_trace,
            };
            Ok(finalize_success_result(FinalizeSuccessInput { result }))
        }
        crate::compose::final_support::FinalSupportComputation::CertifiedNonFinite(cert) => {
            let cost_trace =
                step_cost_trace(&compressed, &dag, &messages, Some(&composed), None, None);
            if ctx.options.exact_image_mode {
                let mut diagnostics = compressed.diagnostics.clone();
                diagnostics.extend(ctx.diagnostics.clone());
                diagnostics.push(exact_image_out_of_scope_nonfinite_diagnostic(
                    &problem,
                    cert.certificate_hash,
                ));
                return Ok(finalize_success_result(FinalizeSuccessInput {
                    result: TargetSolveResult {
                        status: SolverStatus::CertificateDesignGap,
                        target,
                        support_polynomial: None,
                        squarefree_support_polynomial: None,
                        root_isolation: Vec::new(),
                        decoded_candidates: Vec::new(),
                        projection_messages: messages,
                        certificate: None,
                        diagnostics,
                        cost_trace,
                    },
                }));
            }
            finalize_nonfinite_pipeline_result(target, cert, &composed, messages, cost_trace)
        }
    }
}

fn exact_image_out_of_scope_diagnostic(
    problem: &RationalTargetProblem,
    support: &UniPolynomialQ,
    roots: &crate::solver::pipeline::RootCandidateBundle,
) -> DiagnosticRecord {
    let mut diagnostic = DiagnosticRecord::new(
        "ExactImageOutOfScope",
        "exact-image classification is out of scope for this finite candidate-cover repair; support roots are returned unfiltered"
            .to_owned(),
        Some(StageId("P16ExactImageScopeGuard".to_owned())),
    );
    diagnostic
        .details
        .insert("input_hash".to_owned(), format!("{:?}", problem.input_hash));
    diagnostic
        .details
        .insert("support_hash".to_owned(), format!("{:?}", support.hash));
    diagnostic.details.insert(
        "squarefree_support_hash".to_owned(),
        format!("{:?}", roots.squarefree_support.hash),
    );
    diagnostic.details.insert(
        "candidate_hashes".to_owned(),
        roots
            .decoded_candidates
            .iter()
            .map(|candidate| format!("{:?}", candidate.candidate_hash))
            .collect::<Vec<_>>()
            .join(","),
    );
    diagnostic.details.insert(
        "candidate_count".to_owned(),
        roots.decoded_candidates.len().to_string(),
    );
    diagnostic
}

fn exact_image_out_of_scope_nonfinite_diagnostic(
    problem: &RationalTargetProblem,
    nonfinite_certificate_hash: crate::types::hash::Hash,
) -> DiagnosticRecord {
    let mut diagnostic = DiagnosticRecord::new(
        "ExactImageOutOfScope",
        "exact-image classification is out of scope for this finite candidate-cover repair; nonfinite target-image success is not exposed for exact-image requests"
            .to_owned(),
        Some(StageId("P16ExactImageScopeGuard".to_owned())),
    );
    diagnostic
        .details
        .insert("input_hash".to_owned(), format!("{:?}", problem.input_hash));
    diagnostic.details.insert(
        "nonfinite_certificate_hash".to_owned(),
        format!("{:?}", nonfinite_certificate_hash),
    );
    diagnostic
}

fn finalize_pipeline_error(
    err: SolverError,
    target: VariableId,
    compressed: &CompressedSystemQ,
    dag: &TargetProjectionDAG,
    messages: &[ProjectionMessage],
    composed: Option<&ComposedProjection>,
    support: Option<&UniPolynomialQ>,
    certificate: Option<&CoreRunCertificate>,
) -> TargetSolveResult {
    let cost_trace = step_failure_cost_trace(
        compressed,
        dag,
        messages,
        composed,
        support,
        certificate,
        &err,
    );
    TargetSolveResult::from_solver_error_for_target_with_cost_trace(err, target, cost_trace)
}
