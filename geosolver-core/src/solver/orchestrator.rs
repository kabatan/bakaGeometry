use crate::compose::compose::{hash_composed_projection, ComposedProjection};
use crate::problem::context::SolverContext;
use crate::problem::input::RationalTargetProblem;
use crate::result::diagnostics::DiagnosticRecord;
use crate::result::output::{finalize_success_result, FinalizeSuccessInput, TargetSolveResult};
use crate::result::status::{SolverError, SolverStatus, StageId};
use crate::solver::pipeline::{
    finalize_nonfinite_pipeline_result, step_build_dag, step_build_graphs, step_canonicalize,
    step_compose, step_compress, step_core_certificate, step_cost_trace, step_execute, step_plan,
    step_roots, step_support, step_validate, step_verify_messages,
};

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
    if compressed.relations.is_empty() {
        let mut composed = ComposedProjection {
            target,
            root_block_id: dag.root_block_id,
            message_relations: Vec::new(),
            root_relations: Vec::new(),
            source_message_hashes: Vec::new(),
            separator_elimination_hashes: Vec::new(),
            separator_elimination_messages: Vec::new(),
            composition_cost: Default::default(),
            composed_hash: crate::types::hash::hash_sequence("composed-projection", &[]),
        };
        composed.composed_hash = hash_composed_projection(&composed);
        let support_outcome = step_support(&composed, &compressed, target, &mut ctx)?;
        let cost_trace = step_cost_trace(&compressed, &dag, &[], Some(&composed));
        if let crate::compose::final_support::FinalSupportComputation::CertifiedNonFinite(cert) =
            support_outcome
        {
            return finalize_nonfinite_pipeline_result(
                target,
                cert,
                &composed,
                Vec::new(),
                cost_trace,
            );
        }
    }
    let plans = step_plan(&dag, &compressed, &mut ctx)?;
    let messages = step_execute(&dag, &plans, &compressed, &mut ctx)?;
    step_verify_messages(&dag, &messages, &compressed)?;
    let composed = step_compose(&dag, messages.clone(), target, &mut ctx)?;
    let support_outcome = step_support(&composed, &compressed, target, &mut ctx)?;
    let cost_trace = step_cost_trace(&compressed, &dag, &messages, Some(&composed));

    match support_outcome {
        crate::compose::final_support::FinalSupportComputation::Support(support) => {
            let support_certificate =
                crate::verify::verify_support::verify_global_support(&support, &composed)?;
            let mut roots = step_roots(&support, target, &mut ctx)?;
            let exact_image_certificate = if ctx.options.exact_image_mode {
                let classification = crate::fiber::exact_image::classify_real_target_image(
                    &compressed,
                    &roots.squarefree_support,
                    &roots.decoded_candidates,
                    &mut ctx,
                )?;
                roots.root_isolation = classification.exact_root_isolation.clone();
                roots.decoded_candidates = classification.exact_candidates.clone();
                Some(classification)
            } else {
                None
            };
            let certificate = step_core_certificate(
                &problem,
                &canonical,
                &compressed,
                &graphs,
                &dag,
                &plans,
                &messages,
                Some(&support),
                &roots,
                Some(&support_certificate),
            );
            let mut diagnostics = compressed.diagnostics.clone();
            diagnostics.extend(ctx.diagnostics.clone());
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
                diagnostics,
                cost_trace,
            };
            Ok(finalize_success_result(FinalizeSuccessInput { result }))
        }
        crate::compose::final_support::FinalSupportComputation::CertifiedNonFinite(cert) => {
            finalize_nonfinite_pipeline_result(target, cert, &composed, messages, cost_trace)
        }
    }
}
