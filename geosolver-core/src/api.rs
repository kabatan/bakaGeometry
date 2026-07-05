use crate::problem::context::SolverContext;
use crate::problem::input::RationalTargetProblem;
use crate::result::output::TargetSolveResult;
use crate::solver::options::SolverOptions;

pub fn solve_target(problem: RationalTargetProblem, options: SolverOptions) -> TargetSolveResult {
    let ctx = SolverContext::new(options);
    match crate::solver::orchestrator::solve_with_context(problem, ctx) {
        Ok(result) => result,
        Err(err) => TargetSolveResult::from_solver_error(err),
    }
}
