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
    let mut chunks = equations
        .iter()
        .map(|p| p.hash.0.to_vec())
        .collect::<Vec<_>>();
    chunks.extend(
        semantic_encodings
            .iter()
            .map(|encoding| encoding.semantic_hash.0.to_vec()),
    );
    RationalTargetProblem {
        variables,
        target,
        equations,
        semantic_encodings,
        variable_roles: Vec::new(),
        input_hash: hash_sequence("problem-input", &chunks),
    }
}
