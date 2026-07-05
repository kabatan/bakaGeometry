use crate::result::status::{AlgebraicReason, FailureKind, SolverError, SolverErrorKind, StageId};
use crate::types::hash::hash_sequence;
use crate::types::univariate::{
    degree_uni, normalize_univariate, squarefree_part_uni, UniPolynomialQ,
};

pub fn squarefree_support(p: &UniPolynomialQ) -> Result<UniPolynomialQ, SolverError> {
    let normalized = normalize_univariate(p.clone());
    if degree_uni(&normalized).is_none() && normalized.coeffs_low_to_high.is_empty() {
        return Err(SolverError {
            target: Some(normalized.variable),
            kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
                stage: StageId("SquarefreeSupport".to_owned()),
                reason: AlgebraicReason("zero support".to_owned()),
                minimal_block_hash: hash_sequence(
                    "p12-zero-support",
                    &[normalized.hash.0.to_vec()],
                ),
            }),
        });
    }
    Ok(squarefree_part_uni(&normalized))
}
