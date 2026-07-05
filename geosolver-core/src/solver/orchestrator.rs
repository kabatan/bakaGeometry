use crate::problem::context::SolverContext;
use crate::problem::input::RationalTargetProblem;
use crate::result::output::TargetSolveResult;
use crate::result::status::SolverError;

pub fn solve_with_context(
    problem: RationalTargetProblem,
    _ctx: SolverContext,
) -> Result<TargetSolveResult, SolverError> {
    Err(SolverError::temporary_pipeline_not_connected(Some(
        problem.target,
    )))
}
