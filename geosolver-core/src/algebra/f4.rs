use serde::{Deserialize, Serialize};

use crate::algebra::elimination::{LocalEliminationResult, LocalEliminationStrategyName};
use crate::algebra::groebner::{groebner_elimination_basis, GroebnerOptions};
use crate::algebra::monomial_order::{elimination_order, MonomialOrder};
use crate::algebra::polynomial_ops::{reduce_by_set, ReductionResult};
use crate::result::status::SolverError;
use crate::types::ids::VariableId;
use crate::types::polynomial::SparsePolynomialQ;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct F4Options {
    pub groebner_options: GroebnerOptions,
}

impl Default for F4Options {
    fn default() -> Self {
        Self {
            groebner_options: GroebnerOptions::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct F4BatchReductionResult {
    pub reductions: Vec<ReductionResult>,
    pub matrix_rows: usize,
    pub matrix_cols: usize,
}

pub fn f4_reduce_batch(
    reducers: &[SparsePolynomialQ],
    targets: &[SparsePolynomialQ],
    order: &MonomialOrder,
    _options: F4Options,
) -> Result<F4BatchReductionResult, SolverError> {
    let reductions: Vec<ReductionResult> = targets
        .iter()
        .map(|target| reduce_by_set(target, reducers, order))
        .collect();
    let matrix_rows = targets.len();
    let matrix_cols = reducers.iter().map(|p| p.terms.len()).sum::<usize>();
    Ok(F4BatchReductionResult {
        reductions,
        matrix_rows,
        matrix_cols,
    })
}

pub fn f4_elimination_local(
    relations: &[SparsePolynomialQ],
    eliminate: &[VariableId],
    keep: &[VariableId],
    options: F4Options,
) -> Result<LocalEliminationResult, SolverError> {
    let order = elimination_order(eliminate, keep);
    let basis = groebner_elimination_basis(relations, &order, options.groebner_options)?;
    crate::algebra::elimination::local_result_from_groebner(
        relations,
        keep,
        basis,
        LocalEliminationStrategyName::LocalF4,
    )
}

#[cfg(test)]
mod tests {
    use crate::algebra::elimination::validate_local_elimination_result;
    use crate::algebra::monomial_order::{elimination_order, lex_order};
    use crate::types::polynomial::{constant_poly, poly_mul, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    use super::*;

    #[test]
    fn f4_reduce_batch_reduces_targets_and_records_matrix_shape() {
        let x = VariableId(1);
        let reducer = poly_sub(&variable_poly(x), &constant_poly(int_q(1)));
        let target = poly_sub(
            &poly_mul(&variable_poly(x), &variable_poly(x)),
            &constant_poly(int_q(1)),
        );
        let order = lex_order(&[x]);

        let result = f4_reduce_batch(&[reducer], &[target], &order, F4Options::default()).unwrap();

        assert_eq!(result.matrix_rows, 1);
        assert_eq!(result.matrix_cols, 2);
        assert!(result.reductions[0].remainder.terms.is_empty());
    }

    #[test]
    fn f4_elimination_local_exports_keep_only_generators_with_certificates() {
        let x = VariableId(1);
        let y = VariableId(2);
        let relations = vec![
            poly_sub(&variable_poly(y), &variable_poly(x)),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
        ];

        let result = f4_elimination_local(&relations, &[y], &[x], F4Options::default()).unwrap();

        assert_eq!(result.strategy, LocalEliminationStrategyName::LocalF4);
        assert!(!result.generators.is_empty());
        validate_local_elimination_result(&result, &[x], &relations).unwrap();
        let order = elimination_order(&[y], &[x]);
        let reduced = f4_reduce_batch(
            &relations,
            &result
                .generators
                .iter()
                .map(|generator| generator.generator.clone())
                .collect::<Vec<_>>(),
            &order,
            F4Options::default(),
        )
        .unwrap();
        assert_eq!(reduced.matrix_rows, result.generators.len());
    }
}
