use crate::problem::canonicalize::CanonicalSystemQ;
use crate::problem::context::SolverContext;
use crate::problem::input::RationalTargetProblem;
use crate::problem::validate::{validate_input, ValidatedProblem};
use crate::result::status::SolverError;

pub fn step_validate(
    problem: RationalTargetProblem,
    _ctx: &mut SolverContext,
) -> Result<ValidatedProblem, SolverError> {
    validate_input(problem)
}

pub fn step_canonicalize(
    validated: ValidatedProblem,
    _ctx: &mut SolverContext,
) -> Result<CanonicalSystemQ, SolverError> {
    crate::problem::canonicalize::canonicalize_system(validated)
}
