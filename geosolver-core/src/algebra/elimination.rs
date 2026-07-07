use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::algebra::f4::{f4_elimination_local, F4Options};
use crate::algebra::groebner::{
    extract_certified_elimination_generators, groebner_elimination_basis, implementation_bug,
    polynomial_in_keep_variables, CertifiedPolynomialQ, GroebnerBasisResult, GroebnerOptions,
};
use crate::algebra::monomial_order::elimination_order;
use crate::algebra::normal_form::{verify_membership_by_certificate, MembershipCertificate};
use crate::problem::context::SolverContext;
use crate::result::status::SolverError;
use crate::types::ids::VariableId;
use crate::types::polynomial::{poly_variables, SparsePolynomialQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EliminationStrategy {
    EliminationGroebnerLocal(GroebnerOptions),
    F4EliminationLocal(F4Options),
    TargetRelationSearchEscalated,
    ResultantIfSquareOrOverdetermined,
    SpecializeProjectInterpolateVerify,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalEliminationStrategyName {
    EliminationGroebnerLocal,
    F4EliminationLocal,
    TargetRelationSearchEscalated,
    ResultantIfSquareOrOverdetermined,
    SpecializeProjectInterpolateVerify,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedEliminationGenerator {
    pub generator: SparsePolynomialQ,
    pub certificate: MembershipCertificate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalEliminationResult {
    pub generators: Vec<CertifiedEliminationGenerator>,
    pub strategy: LocalEliminationStrategyName,
    pub matrix_rows: usize,
    pub matrix_cols: usize,
}

pub type EliminationResult = LocalEliminationResult;

pub fn eliminate_to_keep_variables(
    relations: &[SparsePolynomialQ],
    eliminate: &[VariableId],
    keep: &[VariableId],
    strategy: EliminationStrategy,
    _ctx: &mut SolverContext,
) -> Result<EliminationResult, SolverError> {
    ensure_disjoint(eliminate, keep)?;
    let result = match strategy {
        EliminationStrategy::EliminationGroebnerLocal(options) => {
            let order = elimination_order(eliminate, keep);
            let basis = groebner_elimination_basis(relations, &order, options)?;
            local_result_from_groebner(
                relations,
                keep,
                basis,
                LocalEliminationStrategyName::EliminationGroebnerLocal,
            )?
        }
        EliminationStrategy::F4EliminationLocal(options) => {
            f4_elimination_local(relations, eliminate, keep, options)?
        }
        EliminationStrategy::TargetRelationSearchEscalated
        | EliminationStrategy::ResultantIfSquareOrOverdetermined
        | EliminationStrategy::SpecializeProjectInterpolateVerify => {
            return Err(implementation_bug(
                "declared elimination strategy is handled by a later planner/kernel phase",
            ));
        }
    };
    validate_local_elimination_result(&result, keep, relations)?;
    Ok(result)
}

pub fn local_result_from_groebner(
    relations: &[SparsePolynomialQ],
    keep: &[VariableId],
    basis: GroebnerBasisResult,
    strategy: LocalEliminationStrategyName,
) -> Result<LocalEliminationResult, SolverError> {
    let matrix_cols = basis.basis.len();
    local_result_from_certified_basis(
        relations,
        keep,
        basis.basis,
        strategy,
        basis.pairs_processed,
        matrix_cols,
    )
}

pub fn local_result_from_certified_basis(
    relations: &[SparsePolynomialQ],
    keep: &[VariableId],
    basis: Vec<CertifiedPolynomialQ>,
    strategy: LocalEliminationStrategyName,
    matrix_rows: usize,
    matrix_cols: usize,
) -> Result<LocalEliminationResult, SolverError> {
    let keep_set: BTreeSet<_> = keep.iter().copied().collect();
    let certified = extract_certified_elimination_generators(&basis, &keep_set);
    let generators = certified
        .into_iter()
        .map(|entry| certified_generator_from_basis_entry(entry, relations))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(LocalEliminationResult {
        generators,
        strategy,
        matrix_rows,
        matrix_cols,
    })
}

pub fn validate_local_elimination_result(
    result: &LocalEliminationResult,
    keep: &[VariableId],
    relations: &[SparsePolynomialQ],
) -> Result<(), SolverError> {
    let keep_set: BTreeSet<_> = keep.iter().copied().collect();
    for generator in &result.generators {
        if !polynomial_in_keep_variables(&generator.generator, &keep_set) {
            return Err(implementation_bug(
                "local elimination exported a generator containing a non-keep variable",
            ));
        }
        if !verify_membership_by_certificate(
            &generator.generator,
            &generator.certificate,
            relations,
        ) {
            return Err(implementation_bug(
                "local elimination exported a generator with an invalid membership certificate",
            ));
        }
    }
    Ok(())
}

fn certified_generator_from_basis_entry(
    entry: CertifiedPolynomialQ,
    relations: &[SparsePolynomialQ],
) -> Result<CertifiedEliminationGenerator, SolverError> {
    if !verify_membership_by_certificate(&entry.polynomial, &entry.certificate, relations) {
        return Err(implementation_bug(
            "Groebner elimination basis entry failed certificate verification",
        ));
    }
    Ok(CertifiedEliminationGenerator {
        generator: entry.polynomial,
        certificate: entry.certificate,
    })
}

fn ensure_disjoint(eliminate: &[VariableId], keep: &[VariableId]) -> Result<(), SolverError> {
    let eliminate_set: BTreeSet<_> = eliminate.iter().copied().collect();
    if keep.iter().any(|var| eliminate_set.contains(var)) {
        return Err(SolverError::invalid_input(
            None,
            "eliminate and keep variable sets must be disjoint",
        ));
    }
    Ok(())
}

#[allow(dead_code)]
fn generator_variables(result: &LocalEliminationResult) -> BTreeSet<VariableId> {
    result
        .generators
        .iter()
        .flat_map(|generator| poly_variables(&generator.generator))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::algebra::normal_form::MembershipTerm;
    use crate::problem::context::new_context;
    use crate::solver::options::SolverOptions;
    use crate::types::polynomial::{constant_poly, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    use super::*;

    #[test]
    fn dispatcher_checks_disjoint_keep_and_eliminate_sets() {
        let x = VariableId(1);
        let mut ctx = new_context(SolverOptions::default());
        let err = eliminate_to_keep_variables(
            &[],
            &[x],
            &[x],
            EliminationStrategy::EliminationGroebnerLocal(GroebnerOptions::default()),
            &mut ctx,
        )
        .unwrap_err();
        assert_eq!(
            err.public_status(),
            crate::result::status::SolverStatus::InvalidInput
        );
    }

    #[test]
    fn dispatcher_exports_keep_only_generator_with_certificate() {
        let x = VariableId(1);
        let y = VariableId(2);
        let relations = vec![
            poly_sub(&variable_poly(y), &variable_poly(x)),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
        ];
        let mut ctx = new_context(SolverOptions::default());
        let result = eliminate_to_keep_variables(
            &relations,
            &[y],
            &[x],
            EliminationStrategy::EliminationGroebnerLocal(GroebnerOptions::default()),
            &mut ctx,
        )
        .unwrap();
        assert!(!result.generators.is_empty());
        validate_local_elimination_result(&result, &[x], &relations).unwrap();
    }

    #[test]
    fn non_keep_export_is_implementation_bug() {
        let x = VariableId(1);
        let y = VariableId(2);
        let relations = vec![variable_poly(y)];
        let bad = LocalEliminationResult {
            generators: vec![CertifiedEliminationGenerator {
                generator: variable_poly(y),
                certificate: MembershipCertificate {
                    combination_terms: vec![MembershipTerm {
                        relation_id: 0,
                        multiplier: constant_poly(int_q(1)),
                    }],
                },
            }],
            strategy: LocalEliminationStrategyName::EliminationGroebnerLocal,
            matrix_rows: 0,
            matrix_cols: 0,
        };
        let err = validate_local_elimination_result(&bad, &[x], &relations).unwrap_err();
        assert_eq!(
            err.public_status(),
            crate::result::status::SolverStatus::ImplementationBug
        );
    }

    #[test]
    fn dispatcher_executes_declared_f4_strategy_without_groebner_fallback() {
        let x = VariableId(1);
        let y = VariableId(2);
        let relations = vec![
            poly_sub(&variable_poly(y), &variable_poly(x)),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
        ];
        let mut ctx = new_context(SolverOptions::default());
        let result = eliminate_to_keep_variables(
            &relations,
            &[y],
            &[x],
            EliminationStrategy::F4EliminationLocal(F4Options::default()),
            &mut ctx,
        )
        .unwrap();
        assert_eq!(
            result.strategy,
            LocalEliminationStrategyName::F4EliminationLocal
        );
        validate_local_elimination_result(&result, &[x], &relations).unwrap();
        assert!(result.matrix_rows > 0);
        assert!(result.matrix_cols > 0);
    }
}
