use crate::compose::compose::ComposedProjection;
use crate::compose::message::ProjectionMessage;
use crate::graph::projection_dag::TargetProjectionDAG;
use crate::preprocess::compression::CompressedSystemQ;
use crate::problem::context::SolverContext;
use crate::problem::input::RationalTargetProblem;
use crate::result::diagnostics::DiagnosticRecord;
use crate::result::output::{finalize_success_result, FinalizeSuccessInput, TargetSolveResult};
use crate::result::status::{SolverError, SolverStatus, StageId};
use crate::solver::pipeline::{
    finalize_nonfinite_pipeline_result, step_build_dag, step_build_graphs, step_canonicalize,
    step_compose, step_compress, step_core_certificate, step_cost_trace, step_execute,
    step_failure_cost_trace, step_plan, step_roots, step_support, step_validate,
    step_verify_messages,
};
use crate::types::ids::VariableId;
use crate::types::univariate::UniPolynomialQ;
use crate::verify::run_certificate::CoreRunCertificate;

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
            let mut roots = match step_roots(&support, target, &mut ctx) {
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
            let exact_image_certificate = if ctx.options.exact_image_mode {
                let classification = match crate::fiber::exact_image::classify_real_target_image(
                    &compressed,
                    &roots.squarefree_support,
                    &roots.decoded_candidates,
                    &mut ctx,
                ) {
                    Ok(classification) => classification,
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
                roots.root_isolation = classification.exact_root_isolation.clone();
                roots.decoded_candidates = classification.exact_candidates.clone();
                Some(classification)
            } else {
                None
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
                exact_image_certificate.as_ref(),
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
            if !ctx.options.exact_image_mode {
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
                    if ctx.options.exact_image_mode {
                        "CertifiedEmptyRealTargetImage"
                    } else {
                        "EmptyRealCandidateCover"
                    },
                    if ctx.options.exact_image_mode {
                        "exact-image classification rejected every real target candidate".to_owned()
                    } else {
                        "support has no real roots; certified candidate cover is empty".to_owned()
                    },
                    Some(StageId(if ctx.options.exact_image_mode {
                        "P13ExactImage".to_owned()
                    } else {
                        "P12RootDecode".to_owned()
                    })),
                ));
            }
            let result = TargetSolveResult {
                status: if ctx.options.exact_image_mode {
                    if roots.decoded_candidates.is_empty() {
                        SolverStatus::CertifiedEmptyRealTargetImage
                    } else {
                        SolverStatus::CertifiedExactTargetImage
                    }
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
                exact_image_certificate,
                nonfinite_certificate: None,
                diagnostics,
                cost_trace,
            };
            Ok(finalize_success_result(FinalizeSuccessInput { result }))
        }
        crate::compose::final_support::FinalSupportComputation::CertifiedNonFinite(cert) => {
            let cost_trace =
                step_cost_trace(&compressed, &dag, &messages, Some(&composed), None, None);
            finalize_nonfinite_pipeline_result(target, cert, &composed, messages, cost_trace)
        }
    }
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
