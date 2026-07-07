use serde::{Deserialize, Serialize};

use crate::problem::semantic::RealConstraintEncoding;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::polynomial::SparsePolynomialQ;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RationalTargetProblem {
    pub variables: Vec<VariableId>,
    pub target: VariableId,
    pub equations: Vec<SparsePolynomialQ>,
    pub semantic_encodings: Vec<RealConstraintEncoding>,
    pub variable_roles: Vec<VariableRoleRecord>,
    pub input_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariableRoleRecord {
    pub variable: VariableId,
    pub role_name: String,
}

pub fn make_problem(
    variables: Vec<VariableId>,
    target: VariableId,
    equations: Vec<SparsePolynomialQ>,
    semantic_encodings: Vec<RealConstraintEncoding>,
) -> RationalTargetProblem {
    make_problem_with_roles(variables, target, equations, semantic_encodings, Vec::new())
}

pub fn make_problem_with_roles(
    variables: Vec<VariableId>,
    target: VariableId,
    equations: Vec<SparsePolynomialQ>,
    semantic_encodings: Vec<RealConstraintEncoding>,
    variable_roles: Vec<VariableRoleRecord>,
) -> RationalTargetProblem {
    let input_hash = hash_problem_input(
        &variables,
        target,
        &equations,
        &semantic_encodings,
        &variable_roles,
    );
    RationalTargetProblem {
        variables,
        target,
        equations,
        semantic_encodings,
        variable_roles,
        input_hash,
    }
}

pub fn hash_problem_input(
    variables: &[VariableId],
    target: VariableId,
    equations: &[SparsePolynomialQ],
    semantic_encodings: &[RealConstraintEncoding],
    variable_roles: &[VariableRoleRecord],
) -> Hash {
    let mut chunks = vec![target.0.to_be_bytes().to_vec()];
    chunks.push(
        variables
            .iter()
            .flat_map(|variable| variable.0.to_be_bytes())
            .collect(),
    );
    chunks.extend(equations.iter().map(|p| p.hash.0.to_vec()));
    chunks.extend(
        semantic_encodings
            .iter()
            .map(|encoding| encoding.semantic_hash.0.to_vec()),
    );
    for role in variable_roles {
        chunks.push(role.variable.0.to_be_bytes().to_vec());
        chunks.push(role.role_name.as_bytes().to_vec());
    }
    hash_sequence("problem-input", &chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::semantic::{register_slack_encoding, RealConstraintKind};
    use crate::types::ids::RelationId;
    use crate::types::polynomial::variable_poly;

    #[test]
    fn input_hash_binds_target_variables_semantics_and_roles() {
        let x = VariableId(0);
        let y = VariableId(1);
        let equation = variable_poly(y);
        let base = make_problem(vec![x, y], x, vec![equation.clone()], Vec::new());
        let target_changed = make_problem(vec![x, y], y, vec![equation.clone()], Vec::new());
        assert_ne!(base.input_hash, target_changed.input_hash);

        let semantic =
            register_slack_encoding(RealConstraintKind::Positive, vec![RelationId(0)], vec![y]);
        let with_semantic = make_problem(vec![x, y], x, vec![equation.clone()], vec![semantic]);
        assert_ne!(base.input_hash, with_semantic.input_hash);

        let with_roles = make_problem_with_roles(
            vec![x, y],
            x,
            vec![equation],
            Vec::new(),
            vec![VariableRoleRecord {
                variable: y,
                role_name: "slack".to_string(),
            }],
        );
        assert_ne!(base.input_hash, with_roles.input_hash);
    }
}
