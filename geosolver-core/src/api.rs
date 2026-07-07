use crate::problem::input::RationalTargetProblem;
use crate::result::output::TargetSolveResult;
use crate::solver::options::SolverOptions;

pub fn solve_target(problem: RationalTargetProblem, options: SolverOptions) -> TargetSolveResult {
    crate::solver::orchestrator::solve_target(problem, options)
}
