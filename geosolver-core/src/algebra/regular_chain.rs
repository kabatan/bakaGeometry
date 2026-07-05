use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::algebra::elimination::{eliminate_to_keep_variables, EliminationStrategy};
use crate::algebra::groebner::{polynomial_in_keep_variables, GroebnerOptions};
use crate::problem::context::new_context;
use crate::result::status::{AlgebraicReason, FailureKind, SolverError, SolverErrorKind, StageId};
use crate::solver::options::SolverOptions;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::polynomial::{
    clear_denominators_primitive, normalize_poly, poly_mul, poly_variables, SparsePolynomialQ,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegularChainInput {
    pub relations: Vec<SparsePolynomialQ>,
    pub variables: Vec<VariableId>,
    pub guards: Vec<SparsePolynomialQ>,
    pub semantics: UnionSemantics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegularChainDAG {
    pub chains: Vec<RegularChain>,
    pub semantics: UnionSemantics,
    pub dag_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegularChain {
    pub relations: Vec<SparsePolynomialQ>,
    pub variables: Vec<VariableId>,
    pub main_variables: Vec<VariableId>,
    pub free_variables: Vec<VariableId>,
    pub guards: Vec<SparsePolynomialQ>,
    pub component_semantics: UnionSemantics,
    pub source_relation_hashes: Vec<Hash>,
    pub component_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionGenerators {
    pub chain_hash: Hash,
    pub keep_variables: Vec<VariableId>,
    pub generators: Vec<SparsePolynomialQ>,
    pub guards: Vec<SparsePolynomialQ>,
    pub component_semantics: UnionSemantics,
    pub projection_hash: Hash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnionSemantics {
    ComponentUnion,
    ComponentIntersection,
}

pub fn local_regular_chain_decomposition(
    input: RegularChainInput,
) -> Result<RegularChainDAG, SolverError> {
    let variables = canonical_variables(&input.variables)?;
    let variable_set: BTreeSet<_> = variables.iter().copied().collect();
    let relations = input
        .relations
        .into_iter()
        .map(normalize_poly)
        .filter(|relation| !relation.terms.is_empty())
        .collect::<Vec<_>>();
    if relations.is_empty() {
        return Err(algorithmic_hard_case(
            None,
            "RegularChainDecomposition",
            "regular-chain decomposition requires at least one nonzero relation",
        ));
    }
    for relation in &relations {
        if poly_variables(relation)
            .iter()
            .any(|var| !variable_set.contains(var))
        {
            return Err(SolverError::invalid_input(
                None,
                "regular-chain relation contains a variable outside the declared variable order",
            ));
        }
    }

    let guards = input
        .guards
        .into_iter()
        .map(normalize_poly)
        .filter(|guard| !guard.terms.is_empty())
        .collect::<Vec<_>>();
    for guard in &guards {
        if poly_variables(guard)
            .iter()
            .any(|var| !variable_set.contains(var))
        {
            return Err(SolverError::invalid_input(
                None,
                "regular-chain guard contains a variable outside the declared variable order",
            ));
        }
    }

    let main_variables = triangular_main_variables(&relations, &variables)?;
    let chains = build_component_chains(
        &relations,
        &variables,
        &main_variables,
        &guards,
        input.semantics,
    )?;
    let dag_hash = hash_sequence(
        "regular-chain-dag",
        &chains
            .iter()
            .map(|chain| chain.component_hash.0.to_vec())
            .collect::<Vec<_>>(),
    );
    Ok(RegularChainDAG {
        chains,
        semantics: input.semantics,
        dag_hash,
    })
}

pub fn project_chain_to_variables(
    chain: &RegularChain,
    keep: &[VariableId],
) -> Result<ProjectionGenerators, SolverError> {
    let keep_variables = canonical_variables(keep)?;
    let keep_set: BTreeSet<_> = keep_variables.iter().copied().collect();
    let eliminate = chain
        .variables
        .iter()
        .copied()
        .filter(|var| !keep_set.contains(var))
        .collect::<Vec<_>>();
    let generators = if eliminate.is_empty() {
        chain
            .relations
            .iter()
            .filter(|relation| polynomial_in_keep_variables(relation, &keep_set))
            .cloned()
            .collect::<Vec<_>>()
    } else {
        let mut ctx = new_context(SolverOptions::default());
        eliminate_to_keep_variables(
            &chain.relations,
            &eliminate,
            &keep_variables,
            EliminationStrategy::LocalGroebner(GroebnerOptions::default()),
            &mut ctx,
        )?
        .generators
        .into_iter()
        .map(|entry| entry.generator)
        .collect::<Vec<_>>()
    };
    for generator in &generators {
        if !polynomial_in_keep_variables(generator, &keep_set) {
            return Err(implementation_bug(
                "regular-chain projection exported a generator containing a non-keep variable",
            ));
        }
    }
    let generators = dedup_polynomials(generators);
    let projection_hash = hash_projection(
        chain.component_hash,
        &keep_variables,
        &generators,
        &chain.guards,
        chain.component_semantics,
    );
    Ok(ProjectionGenerators {
        chain_hash: chain.component_hash,
        keep_variables,
        generators,
        guards: chain.guards.clone(),
        component_semantics: chain.component_semantics,
        projection_hash,
    })
}

pub fn combine_chain_projections(
    chains: &[ProjectionGenerators],
    semantics: UnionSemantics,
) -> Result<Vec<SparsePolynomialQ>, SolverError> {
    if chains.is_empty() {
        return Ok(Vec::new());
    }
    match semantics {
        UnionSemantics::ComponentIntersection => Ok(dedup_polynomials(
            chains
                .iter()
                .flat_map(|chain| chain.generators.clone())
                .collect(),
        )),
        UnionSemantics::ComponentUnion => {
            let mut products = chains[0].generators.clone();
            if products.is_empty() {
                return Ok(Vec::new());
            }
            for chain in chains.iter().skip(1) {
                if chain.generators.is_empty() {
                    return Ok(Vec::new());
                }
                let mut next = Vec::new();
                for left in &products {
                    for right in &chain.generators {
                        next.push(clear_denominators_primitive(&poly_mul(left, right)));
                    }
                }
                products = dedup_polynomials(next);
            }
            Ok(dedup_polynomials(products))
        }
    }
}

fn triangular_main_variables(
    relations: &[SparsePolynomialQ],
    variables: &[VariableId],
) -> Result<Vec<VariableId>, SolverError> {
    let order_index = variables
        .iter()
        .enumerate()
        .map(|(idx, var)| (*var, idx))
        .collect::<BTreeMap<_, _>>();
    let mut main_variables = Vec::new();
    for relation in relations {
        let vars = poly_variables(relation);
        let Some(main) = vars
            .iter()
            .max_by_key(|var| order_index.get(var).copied().unwrap_or(0))
            .copied()
        else {
            return Err(algorithmic_hard_case(
                None,
                "RegularChainDecomposition",
                "nonzero constant relation cannot be a regular-chain polynomial",
            ));
        };
        main_variables.push(main);
    }
    Ok(main_variables)
}

fn build_component_chains(
    relations: &[SparsePolynomialQ],
    variables: &[VariableId],
    main_variables: &[VariableId],
    guards: &[SparsePolynomialQ],
    semantics: UnionSemantics,
) -> Result<Vec<RegularChain>, SolverError> {
    let mut components: Vec<(
        Vec<SparsePolynomialQ>,
        Vec<VariableId>,
        BTreeSet<VariableId>,
    )> = Vec::new();
    for (relation, main) in relations
        .iter()
        .cloned()
        .zip(main_variables.iter().copied())
    {
        if let Some((component_relations, component_mains, seen_mains)) = components
            .iter_mut()
            .find(|(_, _, seen)| !seen.contains(&main))
        {
            component_relations.push(relation);
            component_mains.push(main);
            seen_mains.insert(main);
        } else {
            let mut seen_mains = BTreeSet::new();
            seen_mains.insert(main);
            components.push((vec![relation], vec![main], seen_mains));
        }
    }
    components
        .into_iter()
        .map(|(component_relations, component_mains, _)| {
            let main_set: BTreeSet<_> = component_mains.iter().copied().collect();
            let free_variables = variables
                .iter()
                .copied()
                .filter(|var| !main_set.contains(var))
                .collect::<Vec<_>>();
            let source_relation_hashes = component_relations
                .iter()
                .map(|relation| relation.hash)
                .collect::<Vec<_>>();
            let component_hash = hash_chain(
                &component_relations,
                variables,
                &component_mains,
                &free_variables,
                guards,
                semantics,
            );
            Ok(RegularChain {
                relations: component_relations,
                variables: variables.to_vec(),
                main_variables: component_mains,
                free_variables,
                guards: guards.to_vec(),
                component_semantics: semantics,
                source_relation_hashes,
                component_hash,
            })
        })
        .collect()
}

fn canonical_variables(vars: &[VariableId]) -> Result<Vec<VariableId>, SolverError> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut previous = None;
    for var in vars {
        if previous.is_some_and(|last| last > *var) || !seen.insert(*var) {
            return Err(SolverError::invalid_input(
                Some(*var),
                "variable lists must be sorted, duplicate-free, and canonical",
            ));
        }
        out.push(*var);
        previous = Some(*var);
    }
    Ok(out)
}

fn dedup_polynomials(polys: Vec<SparsePolynomialQ>) -> Vec<SparsePolynomialQ> {
    let mut by_hash = BTreeMap::new();
    for poly in polys {
        by_hash.insert(poly.hash, clear_denominators_primitive(&poly));
    }
    by_hash.into_values().collect()
}

fn hash_chain(
    relations: &[SparsePolynomialQ],
    variables: &[VariableId],
    main_variables: &[VariableId],
    free_variables: &[VariableId],
    guards: &[SparsePolynomialQ],
    semantics: UnionSemantics,
) -> Hash {
    let mut chunks = Vec::new();
    chunks.push(vec![semantics as u8]);
    for var in variables {
        chunks.push(var.0.to_be_bytes().to_vec());
    }
    for var in main_variables {
        chunks.push(var.0.to_be_bytes().to_vec());
    }
    for var in free_variables {
        chunks.push(var.0.to_be_bytes().to_vec());
    }
    for relation in relations {
        chunks.push(relation.hash.0.to_vec());
    }
    for guard in guards {
        chunks.push(guard.hash.0.to_vec());
    }
    hash_sequence("regular-chain", &chunks)
}

fn hash_projection(
    chain_hash: Hash,
    keep_variables: &[VariableId],
    generators: &[SparsePolynomialQ],
    guards: &[SparsePolynomialQ],
    semantics: UnionSemantics,
) -> Hash {
    let mut chunks = vec![chain_hash.0.to_vec(), vec![semantics as u8]];
    for var in keep_variables {
        chunks.push(var.0.to_be_bytes().to_vec());
    }
    for generator in generators {
        chunks.push(generator.hash.0.to_vec());
    }
    for guard in guards {
        chunks.push(guard.hash.0.to_vec());
    }
    hash_sequence("regular-chain-projection", &chunks)
}

fn algorithmic_hard_case(target: Option<VariableId>, stage: &str, reason: &str) -> SolverError {
    SolverError {
        target,
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId(stage.to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash: hash_sequence(
                "p3f-regular-chain-hard-case",
                &[reason.as_bytes().to_vec()],
            ),
        }),
    }
}

fn implementation_bug(message: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::ImplementationBug {
            invariant_violated: message.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::polynomial::{constant_poly, poly_add, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    fn x() -> VariableId {
        VariableId(1)
    }

    fn y() -> VariableId {
        VariableId(2)
    }

    #[test]
    fn regular_chain_projection_preserves_guards_and_keep_variables() {
        let x_poly = variable_poly(x());
        let y_poly = variable_poly(y());
        let relations = vec![
            poly_sub(&y_poly, &x_poly),
            poly_sub(&poly_mul(&x_poly, &x_poly), &constant_poly(int_q(2))),
        ];
        let guard = poly_add(&x_poly, &constant_poly(int_q(1)));
        let dag = local_regular_chain_decomposition(RegularChainInput {
            relations,
            variables: vec![x(), y()],
            guards: vec![guard.clone()],
            semantics: UnionSemantics::ComponentUnion,
        })
        .unwrap();
        let projected = project_chain_to_variables(&dag.chains[0], &[x()]).unwrap();
        assert_eq!(projected.guards, vec![guard]);
        assert!(projected
            .generators
            .iter()
            .all(|generator| poly_variables(generator).iter().all(|var| *var == x())));
    }

    #[test]
    fn duplicate_main_variables_create_component_dag() {
        let x_poly = variable_poly(x());
        let y_poly = variable_poly(y());
        let dag = local_regular_chain_decomposition(RegularChainInput {
            relations: vec![
                poly_sub(&y_poly, &x_poly),
                poly_sub(&y_poly, &constant_poly(int_q(1))),
            ],
            variables: vec![x(), y()],
            guards: Vec::new(),
            semantics: UnionSemantics::ComponentUnion,
        })
        .unwrap();
        assert_eq!(dag.chains.len(), 2);
        assert_ne!(dag.chains[0].component_hash, dag.chains[1].component_hash);
    }

    #[test]
    fn union_combination_multiplies_component_generators() {
        let x_poly = variable_poly(x());
        let g1 = poly_sub(&x_poly, &constant_poly(int_q(1)));
        let g2 = poly_sub(&x_poly, &constant_poly(int_q(2)));
        let first = ProjectionGenerators {
            chain_hash: hash_sequence("a", &[]),
            keep_variables: vec![x()],
            generators: vec![g1.clone()],
            guards: Vec::new(),
            component_semantics: UnionSemantics::ComponentUnion,
            projection_hash: hash_sequence("pa", &[]),
        };
        let second = ProjectionGenerators {
            chain_hash: hash_sequence("b", &[]),
            keep_variables: vec![x()],
            generators: vec![g2.clone()],
            guards: Vec::new(),
            component_semantics: UnionSemantics::ComponentUnion,
            projection_hash: hash_sequence("pb", &[]),
        };
        let combined =
            combine_chain_projections(&[first, second], UnionSemantics::ComponentUnion).unwrap();
        assert_eq!(
            combined,
            vec![clear_denominators_primitive(&poly_mul(&g1, &g2))]
        );
    }
}
