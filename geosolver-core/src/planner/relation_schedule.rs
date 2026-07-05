use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::solver::options::SolverOptions;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::monomial::{
    monomial_degree, monomial_mul, monomial_to_bytes, normalize_monomial, Monomial,
};
use crate::types::polynomial::{poly_total_degree, SparsePolynomialQ};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationSearchBound {
    pub export_degree: usize,
    pub multiplier_total_degree: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationSearchStage {
    pub export_degree: usize,
    pub multiplier_total_degree: usize,
    pub export_support_hash: Hash,
    pub multiplier_support_hashes: Vec<Hash>,
    pub row_monomial_hash: Hash,
    pub row_monomial_count: usize,
    pub matrix_rows: usize,
    pub matrix_cols: usize,
    pub stage_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DenseRelationSearchSchedule {
    pub eliminated_variables: Vec<VariableId>,
    pub exported_variables: Vec<VariableId>,
    pub z_seed: usize,
    pub e_cap: usize,
    pub d_max: usize,
    pub stages: Vec<RelationSearchStage>,
    pub schedule_hash: Hash,
}

pub fn relation_search_default_export_degree_cap(
    j: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
) -> usize {
    let d_max = j
        .iter()
        .map(|relation| poly_total_degree(relation) as usize)
        .max()
        .unwrap_or(0);
    let z_seed = relation_search_z_seed(j, exported);
    z_seed.max(
        2_usize
            .saturating_mul(d_max)
            .saturating_add(eliminated.len())
            .saturating_add(exported.len()),
    )
}

pub fn build_dense_relation_search_schedule(
    j: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    options: &SolverOptions,
) -> DenseRelationSearchSchedule {
    let eliminated_variables = sorted_variables(eliminated);
    let exported_variables = sorted_variables(exported);
    let z_seed = relation_search_z_seed(j, &exported_variables);
    let d_max = j
        .iter()
        .map(|relation| poly_total_degree(relation) as usize)
        .max()
        .unwrap_or(0);
    let default_cap =
        relation_search_default_export_degree_cap(j, &eliminated_variables, &exported_variables);
    let e_cap = options
        .max_relation_search_export_degree
        .unwrap_or(default_cap);
    let mut stages = Vec::new();
    for e in z_seed..=e_cap {
        let bound = RelationSearchBound {
            export_degree: e,
            multiplier_total_degree: e.saturating_add(d_max),
        };
        let export_support = build_export_monomial_support(&exported_variables, &bound);
        let multiplier_supports =
            build_multiplier_supports(j, &eliminated_variables, &exported_variables, &bound);
        let row_monomials = build_row_monomial_support(j, &export_support, &multiplier_supports);
        let export_support_hash = hash_monomials("rgq042-export-support", &export_support);
        let multiplier_support_hashes = multiplier_supports
            .iter()
            .map(|support| hash_monomials("rgq042-multiplier-support", support))
            .collect::<Vec<_>>();
        let row_monomial_hash = hash_monomials("rgq042-row-monomials", &row_monomials);
        let matrix_rows = row_monomials.len();
        let matrix_cols = export_support.len()
            + multiplier_supports
                .iter()
                .map(|support| support.len())
                .sum::<usize>();
        let mut stage = RelationSearchStage {
            export_degree: e,
            multiplier_total_degree: bound.multiplier_total_degree,
            export_support_hash,
            multiplier_support_hashes,
            row_monomial_hash,
            row_monomial_count: matrix_rows,
            matrix_rows,
            matrix_cols,
            stage_hash: hash_sequence("rgq042-relation-search-stage", &[]),
        };
        stage.stage_hash = hash_relation_search_stage(&stage);
        stages.push(stage);
    }
    let mut schedule = DenseRelationSearchSchedule {
        eliminated_variables,
        exported_variables,
        z_seed,
        e_cap,
        d_max,
        stages,
        schedule_hash: hash_sequence("rgq042-dense-relation-search-schedule", &[]),
    };
    schedule.schedule_hash = hash_dense_relation_search_schedule(&schedule);
    schedule
}

pub fn hash_dense_relation_search_schedule(schedule: &DenseRelationSearchSchedule) -> Hash {
    let mut chunks = vec![
        schedule.z_seed.to_be_bytes().to_vec(),
        schedule.e_cap.to_be_bytes().to_vec(),
        schedule.d_max.to_be_bytes().to_vec(),
    ];
    for variable in &schedule.eliminated_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for variable in &schedule.exported_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for stage in &schedule.stages {
        chunks.push(stage.stage_hash.0.to_vec());
        chunks.push(hash_relation_search_stage(stage).0.to_vec());
    }
    hash_sequence("rgq042-dense-relation-search-schedule", &chunks)
}

pub fn hash_relation_search_stage(stage: &RelationSearchStage) -> Hash {
    let mut chunks = vec![
        stage.export_degree.to_be_bytes().to_vec(),
        stage.multiplier_total_degree.to_be_bytes().to_vec(),
        stage.export_support_hash.0.to_vec(),
    ];
    for hash in &stage.multiplier_support_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    chunks.push(stage.row_monomial_hash.0.to_vec());
    chunks.push(stage.row_monomial_count.to_be_bytes().to_vec());
    chunks.push(stage.matrix_rows.to_be_bytes().to_vec());
    chunks.push(stage.matrix_cols.to_be_bytes().to_vec());
    hash_sequence("rgq042-relation-search-stage", &chunks)
}

fn build_export_monomial_support(
    exported: &[VariableId],
    bound: &RelationSearchBound,
) -> Vec<Monomial> {
    monomials_total_degree_leq(&sorted_variables(exported), bound.export_degree)
}

fn build_multiplier_supports(
    relations: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    bound: &RelationSearchBound,
) -> Vec<Vec<Monomial>> {
    let variables = sorted_union(eliminated, exported);
    relations
        .iter()
        .map(|relation| {
            let relation_degree = poly_total_degree(relation) as usize;
            let multiplier_degree = bound
                .multiplier_total_degree
                .saturating_sub(relation_degree);
            monomials_total_degree_leq(&variables, multiplier_degree)
        })
        .collect()
}

fn relation_search_z_seed(j: &[SparsePolynomialQ], exported: &[VariableId]) -> usize {
    let exported_set = exported.iter().copied().collect::<BTreeSet<_>>();
    j.iter()
        .flat_map(|relation| relation.terms.iter())
        .map(|term| {
            term.monomial
                .exponents
                .iter()
                .filter(|(var, _)| exported_set.contains(var))
                .map(|(_, exp)| *exp as usize)
                .sum::<usize>()
        })
        .max()
        .unwrap_or(0)
        .max(1)
}

fn build_row_monomial_support(
    relations: &[SparsePolynomialQ],
    export_support: &[Monomial],
    multiplier_supports: &[Vec<Monomial>],
) -> Vec<Monomial> {
    let mut row_monomials = export_support.iter().cloned().collect::<BTreeSet<_>>();
    for (relation, support) in relations.iter().zip(multiplier_supports.iter()) {
        for multiplier in support {
            for term in &relation.terms {
                row_monomials.insert(monomial_mul(multiplier, &term.monomial));
            }
        }
    }
    row_monomials.into_iter().collect()
}

fn monomials_total_degree_leq(variables: &[VariableId], max_degree: usize) -> Vec<Monomial> {
    let mut out = Vec::new();
    let mut current = Vec::new();
    enumerate_monomials(variables, 0, max_degree as u32, &mut current, &mut out);
    out.sort_by(|a, b| (monomial_degree(a), a).cmp(&(monomial_degree(b), b)));
    out
}

fn enumerate_monomials(
    variables: &[VariableId],
    index: usize,
    remaining: u32,
    current: &mut Vec<(VariableId, u32)>,
    out: &mut Vec<Monomial>,
) {
    if index == variables.len() {
        out.push(normalize_monomial(current.clone()));
        return;
    }
    let variable = variables[index];
    for exponent in 0..=remaining {
        if exponent > 0 {
            current.push((variable, exponent));
        }
        enumerate_monomials(variables, index + 1, remaining - exponent, current, out);
        if exponent > 0 {
            current.pop();
        }
    }
}

fn sorted_variables(vars: &[VariableId]) -> Vec<VariableId> {
    vars.iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sorted_union(a: &[VariableId], b: &[VariableId]) -> Vec<VariableId> {
    a.iter()
        .chain(b.iter())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn hash_monomials(tag: &str, monomials: &[Monomial]) -> Hash {
    hash_sequence(
        tag,
        &monomials.iter().map(monomial_to_bytes).collect::<Vec<_>>(),
    )
}
