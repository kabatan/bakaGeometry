use crate::problem::input::RationalTargetProblem;
use crate::problem::semantic::verify_semantic_references;
use crate::result::status::SolverError;
use crate::types::ids::RelationId;
use crate::types::rational::is_zero_q;

#[derive(Debug, Clone)]
pub struct ValidatedProblem {
    pub problem: RationalTargetProblem,
}

pub fn validate_input(problem: RationalTargetProblem) -> Result<ValidatedProblem, SolverError> {
    if !problem.variables.contains(&problem.target) {
        return Err(SolverError::invalid_input(
            Some(problem.target),
            "target is not listed as a variable",
        ));
    }
    let declared: std::collections::BTreeSet<_> = problem.variables.iter().copied().collect();
    for poly in &problem.equations {
        for term in &poly.terms {
            if is_zero_q(&term.coeff) {
                return Err(SolverError::invalid_input(
                    Some(problem.target),
                    "polynomial contains non-normalized zero coefficient term",
                ));
            }
            for (var, _) in &term.monomial.exponents {
                if !declared.contains(var) {
                    return Err(SolverError::invalid_input(
                        Some(problem.target),
                        "polynomial references undeclared variable",
                    ));
                }
            }
        }
    }
    let relation_ids = (0..problem.equations.len())
        .map(|idx| RelationId(idx as u32))
        .collect::<Vec<_>>();
    verify_semantic_references(
        &problem.semantic_encodings,
        &relation_ids,
        &problem.variables,
    )
    .map_err(|_| {
        SolverError::invalid_input(
            Some(problem.target),
            "semantic encoding references unknown relation or variable",
        )
    })?;
    Ok(ValidatedProblem { problem })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::input::{make_problem, make_problem_with_roles, VariableRoleRecord};
    use crate::problem::semantic::{register_slack_encoding, RealConstraintKind};
    use crate::types::ids::VariableId;
    use crate::types::polynomial::variable_poly;

    #[test]
    fn roles_do_not_affect_validation() {
        let problem = make_problem_with_roles(
            vec![VariableId(0), VariableId(1)],
            VariableId(0),
            vec![variable_poly(VariableId(1))],
            Vec::new(),
            vec![VariableRoleRecord {
                variable: VariableId(1),
                role_name: "coordinate".to_string(),
            }],
        );
        assert!(validate_input(problem).is_ok());
    }

    #[test]
    fn slack_and_branch_semantics_are_accepted_when_references_exist() {
        let sem = register_slack_encoding(
            RealConstraintKind::BranchChoice,
            vec![RelationId(0)],
            vec![VariableId(2)],
        );
        let problem = make_problem(
            vec![VariableId(0), VariableId(1), VariableId(2)],
            VariableId(0),
            vec![variable_poly(VariableId(1))],
            vec![sem],
        );
        assert!(validate_input(problem).is_ok());
    }

    #[test]
    fn invalid_target_is_rejected() {
        let problem = make_problem(
            vec![VariableId(1)],
            VariableId(0),
            vec![variable_poly(VariableId(1))],
            Vec::new(),
        );
        assert!(validate_input(problem).is_err());
    }

    #[test]
    fn invalid_semantic_reference_is_rejected() {
        let sem = register_slack_encoding(
            RealConstraintKind::NonZero,
            vec![RelationId(99)],
            Vec::new(),
        );
        let problem = make_problem(
            vec![VariableId(0), VariableId(1)],
            VariableId(0),
            vec![variable_poly(VariableId(1))],
            vec![sem],
        );
        assert!(validate_input(problem).is_err());
    }

    #[test]
    fn invalid_semantic_slack_variable_is_rejected() {
        let sem = register_slack_encoding(
            RealConstraintKind::Positive,
            vec![RelationId(0)],
            vec![VariableId(99)],
        );
        let problem = make_problem(
            vec![VariableId(0), VariableId(1)],
            VariableId(0),
            vec![variable_poly(VariableId(1))],
            vec![sem],
        );
        assert!(validate_input(problem).is_err());
    }
}
