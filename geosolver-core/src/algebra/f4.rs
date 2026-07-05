use serde::{Deserialize, Serialize};

use crate::algebra::elimination::{LocalEliminationResult, LocalEliminationStrategyName};
use crate::algebra::groebner::{groebner_elimination_basis, GroebnerOptions};
use crate::algebra::monomial_order::{elimination_order, MonomialOrder};
use crate::algebra::polynomial_ops::{reduce_by_set, ReductionResult};
use crate::result::status::SolverError;
use crate::types::ids::VariableId;
use crate::types::polynomial::SparsePolynomialQ;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum F4ImplementationLevel {
    NotProductionF4,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroebnerBackedBatchOptions {
    pub groebner_options: GroebnerOptions,
    pub implementation_level: F4ImplementationLevel,
}

impl Default for GroebnerBackedBatchOptions {
    fn default() -> Self {
        Self {
            groebner_options: GroebnerOptions::default(),
            implementation_level: F4ImplementationLevel::NotProductionF4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroebnerBackedBatchReductionResult {
    pub reductions: Vec<ReductionResult>,
    pub matrix_rows: usize,
    pub matrix_cols: usize,
    pub implementation_level: F4ImplementationLevel,
}

pub fn groebner_backed_batch_reduce_for_tests(
    reducers: &[SparsePolynomialQ],
    targets: &[SparsePolynomialQ],
    order: &MonomialOrder,
    options: GroebnerBackedBatchOptions,
) -> Result<GroebnerBackedBatchReductionResult, SolverError> {
    let reductions: Vec<ReductionResult> = targets
        .iter()
        .map(|target| reduce_by_set(target, reducers, order))
        .collect();
    let matrix_rows = targets.len();
    let matrix_cols = reducers.iter().map(|p| p.terms.len()).sum::<usize>();
    Ok(GroebnerBackedBatchReductionResult {
        reductions,
        matrix_rows,
        matrix_cols,
        implementation_level: options.implementation_level,
    })
}

pub fn non_production_groebner_batch_elimination_for_tests(
    relations: &[SparsePolynomialQ],
    eliminate: &[VariableId],
    keep: &[VariableId],
    options: GroebnerBackedBatchOptions,
) -> Result<LocalEliminationResult, SolverError> {
    let order = elimination_order(eliminate, keep);
    let basis = groebner_elimination_basis(relations, &order, options.groebner_options)?;
    crate::algebra::elimination::local_result_from_groebner(
        relations,
        keep,
        basis,
        LocalEliminationStrategyName::NonProductionGroebnerBatch,
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
    fn groebner_backed_batch_reduce_is_labelled_non_production() {
        let x = VariableId(1);
        let reducer = poly_sub(&variable_poly(x), &constant_poly(int_q(1)));
        let target = poly_sub(
            &poly_mul(&variable_poly(x), &variable_poly(x)),
            &constant_poly(int_q(1)),
        );
        let order = lex_order(&[x]);

        let result = groebner_backed_batch_reduce_for_tests(
            &[reducer],
            &[target],
            &order,
            GroebnerBackedBatchOptions::default(),
        )
        .unwrap();

        assert_eq!(result.matrix_rows, 1);
        assert_eq!(result.matrix_cols, 2);
        assert!(result.reductions[0].remainder.terms.is_empty());
        assert_eq!(
            result.implementation_level,
            F4ImplementationLevel::NotProductionF4
        );
    }

    #[test]
    fn non_production_groebner_batch_exports_keep_only_generators_with_certificates() {
        let x = VariableId(1);
        let y = VariableId(2);
        let relations = vec![
            poly_sub(&variable_poly(y), &variable_poly(x)),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
        ];

        let result = non_production_groebner_batch_elimination_for_tests(
            &relations,
            &[y],
            &[x],
            GroebnerBackedBatchOptions::default(),
        )
        .unwrap();

        assert_eq!(
            result.strategy,
            LocalEliminationStrategyName::NonProductionGroebnerBatch
        );
        assert!(!result.generators.is_empty());
        validate_local_elimination_result(&result, &[x], &relations).unwrap();
        let order = elimination_order(&[y], &[x]);
        let reduced = groebner_backed_batch_reduce_for_tests(
            &relations,
            &result
                .generators
                .iter()
                .map(|generator| generator.generator.clone())
                .collect::<Vec<_>>(),
            &order,
            GroebnerBackedBatchOptions::default(),
        )
        .unwrap();
        assert_eq!(reduced.matrix_rows, result.generators.len());
    }
}
