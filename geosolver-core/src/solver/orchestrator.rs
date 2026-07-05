use crate::problem::context::SolverContext;
use crate::problem::input::RationalTargetProblem;
use crate::result::output::{finalize_success_result, FinalizeSuccessInput, TargetSolveResult};
use crate::result::status::{SolverError, SolverStatus};
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
    let plans = step_plan(&dag, &compressed, &mut ctx)?;
    let messages = step_execute(&dag, &plans, &compressed, &mut ctx)?;
    step_verify_messages(&dag, &messages, &compressed)?;
    let composed = step_compose(&dag, messages.clone(), target, &mut ctx)?;
    let support_outcome = step_support(&composed, target, &mut ctx)?;
    let cost_trace = step_cost_trace(&compressed, &dag, &messages, Some(&composed));

    match support_outcome {
        crate::compose::final_support::FinalSupportComputation::Support(support) => {
            let support_certificate =
                crate::verify::verify_support::verify_global_support(&support, &composed)?;
            let roots = step_roots(&support, target, &mut ctx)?;
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
            let result = TargetSolveResult {
                status: SolverStatus::CertifiedCandidateCover,
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
            finalize_nonfinite_pipeline_result(target, cert, &composed, messages, cost_trace)
        }
    }
}
