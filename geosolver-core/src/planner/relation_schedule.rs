use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::solver::options::SolverOptions;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::monomial::{
    monomial_degree, monomial_mul, monomial_to_bytes, normalize_monomial, Monomial,
};
use crate::types::polynomial::{poly_total_degree, SparsePolynomialQ};

pub const DEFAULT_DENSE_TRS_MAX_EXPORT_COLS: usize = 32_768;
pub const DEFAULT_DENSE_TRS_MAX_MULTIPLIER_COLS: usize = 65_536;
pub const DEFAULT_DENSE_TRS_MAX_MATRIX_ROWS: usize = 65_536;
pub const DEFAULT_DENSE_TRS_MAX_MATRIX_COLS: usize = 65_536;
pub const DEFAULT_DENSE_TRS_MAX_ESTIMATED_MEMORY_BYTES: u64 = 256 * 1024 * 1024;
pub const DEFAULT_DENSE_TRS_MAX_MATERIALIZED_STAGES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaturatingCount {
    pub value: Option<u128>,
    pub saturated: bool,
}

impl SaturatingCount {
    pub fn exact(value: u128) -> Self {
        Self {
            value: Some(value),
            saturated: false,
        }
    }

    pub fn saturated() -> Self {
        Self {
            value: None,
            saturated: true,
        }
    }

    pub fn add(self, rhs: Self) -> Self {
        match (self.value, rhs.value) {
            (Some(lhs), Some(rhs_value)) if !self.saturated && !rhs.saturated => lhs
                .checked_add(rhs_value)
                .map(Self::exact)
                .unwrap_or_else(Self::saturated),
            _ => Self::saturated(),
        }
    }

    pub fn mul_u128(self, rhs: u128) -> Self {
        match self.value {
            Some(lhs) if !self.saturated => lhs
                .checked_mul(rhs)
                .map(Self::exact)
                .unwrap_or_else(Self::saturated),
            _ => Self::saturated(),
        }
    }

    pub fn exceeds_usize(self, cap: usize) -> bool {
        self.saturated || self.value.is_none_or(|value| value > cap as u128)
    }

    pub fn exceeds_u64(self, cap: u64) -> bool {
        self.saturated || self.value.is_none_or(|value| value > cap as u128)
    }

    pub fn display_value(self) -> String {
        if self.saturated {
            "saturated".to_owned()
        } else {
            self.value
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_owned())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationSearchPlanningCaps {
    pub max_export_cols: usize,
    pub max_total_multiplier_cols: usize,
    pub max_matrix_rows: usize,
    pub max_matrix_cols: usize,
    pub max_estimated_memory_bytes: u64,
    pub max_materialized_stages: usize,
}

impl RelationSearchPlanningCaps {
    pub fn from_options(options: &SolverOptions) -> Self {
        let max_matrix_rows = options
            .max_matrix_rows
            .unwrap_or(DEFAULT_DENSE_TRS_MAX_MATRIX_ROWS)
            .max(DEFAULT_DENSE_TRS_MAX_MATRIX_ROWS);
        let max_matrix_cols = options
            .max_matrix_cols
            .unwrap_or(DEFAULT_DENSE_TRS_MAX_MATRIX_COLS)
            .max(DEFAULT_DENSE_TRS_MAX_MATRIX_COLS);
        Self {
            max_export_cols: max_matrix_cols.max(DEFAULT_DENSE_TRS_MAX_EXPORT_COLS),
            max_total_multiplier_cols: max_matrix_cols.max(DEFAULT_DENSE_TRS_MAX_MULTIPLIER_COLS),
            max_matrix_rows,
            max_matrix_cols,
            max_estimated_memory_bytes: options
                .max_memory_bytes
                .unwrap_or(DEFAULT_DENSE_TRS_MAX_ESTIMATED_MEMORY_BYTES)
                .max(DEFAULT_DENSE_TRS_MAX_ESTIMATED_MEMORY_BYTES),
            max_materialized_stages: DEFAULT_DENSE_TRS_MAX_MATERIALIZED_STAGES,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationSearchBound {
    pub export_degree: usize,
    pub multiplier_total_degree: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportDescriptor {
    DenseTotalDegree {
        variables: Vec<VariableId>,
        degree: usize,
        estimated_count: SaturatingCount,
    },
    SparseFootprint {
        variables: Vec<VariableId>,
        support_hash: Hash,
        estimated_count: SaturatingCount,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationSearchStageEstimate {
    pub export_degree: usize,
    pub multiplier_total_degree: usize,
    pub export_cols: SaturatingCount,
    pub multiplier_col_counts: Vec<SaturatingCount>,
    pub total_multiplier_cols: SaturatingCount,
    pub estimated_matrix_cols: SaturatingCount,
    pub estimated_rows_upper_bound: SaturatingCount,
    pub estimated_memory_bytes_upper_bound: SaturatingCount,
    pub feasible: bool,
    pub prohibition_reason: Option<String>,
    pub estimate_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DenseRelationSearchPreflight {
    pub eliminated_variables: Vec<VariableId>,
    pub exported_variables: Vec<VariableId>,
    pub z_seed: usize,
    pub e_cap: usize,
    pub d_max: usize,
    pub planned_stage_count: SaturatingCount,
    pub estimated_stage_count: usize,
    pub first_prohibited_stage: Option<usize>,
    pub caps: RelationSearchPlanningCaps,
    pub stage_estimates: Vec<RelationSearchStageEstimate>,
    pub materialization_allowed: bool,
    pub preflight_hash: Hash,
}

impl DenseRelationSearchPreflight {
    pub fn first_prohibition_reason(&self) -> Option<&str> {
        self.stage_estimates
            .iter()
            .find_map(|stage| stage.prohibition_reason.as_deref())
            .or_else(|| {
                if !self.materialization_allowed {
                    Some("stage count exceeds dense TargetRelationSearch materialization cap")
                } else {
                    None
                }
            })
    }
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
    pub preflight: DenseRelationSearchPreflight,
    pub support_descriptors: Vec<SupportDescriptor>,
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

pub fn estimate_dense_relation_search_schedule(
    j: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    options: &SolverOptions,
) -> DenseRelationSearchPreflight {
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
    let caps = RelationSearchPlanningCaps::from_options(options);
    let planned_stage_count = if e_cap >= z_seed {
        SaturatingCount::exact(e_cap.saturating_sub(z_seed).saturating_add(1) as u128)
    } else {
        SaturatingCount::exact(0)
    };
    let planned_stage_count_usize = planned_stage_count
        .value
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(usize::MAX);
    let estimated_stage_count =
        planned_stage_count_usize.min(caps.max_materialized_stages.saturating_add(
            if planned_stage_count_usize > caps.max_materialized_stages {
                1
            } else {
                0
            },
        ));
    let mut stage_estimates = Vec::new();
    for offset in 0..estimated_stage_count {
        let Some(export_degree) = z_seed.checked_add(offset) else {
            break;
        };
        if export_degree > e_cap {
            break;
        }
        stage_estimates.push(estimate_relation_search_stage(
            j,
            &eliminated_variables,
            &exported_variables,
            d_max,
            export_degree,
            &caps,
        ));
    }
    let too_many_stages = planned_stage_count_usize > caps.max_materialized_stages;
    let no_stages = planned_stage_count.value == Some(0);
    let first_prohibited_stage = stage_estimates
        .iter()
        .find(|stage| !stage.feasible)
        .map(|stage| stage.export_degree)
        .or_else(|| {
            if too_many_stages {
                z_seed.checked_add(caps.max_materialized_stages)
            } else {
                None
            }
        });
    let materialization_allowed = !no_stages
        && !too_many_stages
        && stage_estimates.len() == planned_stage_count_usize
        && stage_estimates.iter().all(|stage| stage.feasible);
    let mut preflight = DenseRelationSearchPreflight {
        eliminated_variables,
        exported_variables,
        z_seed,
        e_cap,
        d_max,
        planned_stage_count,
        estimated_stage_count,
        first_prohibited_stage,
        caps,
        stage_estimates,
        materialization_allowed,
        preflight_hash: hash_sequence("dense-trs-preflight", &[]),
    };
    preflight.preflight_hash = hash_dense_relation_search_preflight(&preflight);
    preflight
}

pub fn dense_relation_search_decline_reason(preflight: &DenseRelationSearchPreflight) -> String {
    let reason = preflight
        .first_prohibition_reason()
        .unwrap_or("dense TargetRelationSearch preflight did not admit materialization");
    let first_stage = preflight
        .stage_estimates
        .first()
        .map(|stage| {
            format!(
                "first_export_degree={}, estimated_matrix_cols={}, estimated_rows={}, estimated_memory_bytes={}",
                stage.export_degree,
                stage.estimated_matrix_cols.display_value(),
                stage.estimated_rows_upper_bound.display_value(),
                stage.estimated_memory_bytes_upper_bound.display_value()
            )
        })
        .unwrap_or_else(|| "first_export_degree=none".to_owned());
    format!(
        "CostProhibitedDenseRoute: kernel=TargetRelationSearch route=DenseTotalDegree decision=CostProhibitedDenseRoute {first_stage} matrix_col_cap={} matrix_row_cap={} memory_cap_bytes={} stage_count={} materialized_stage_cap={} reason={reason}",
        preflight.caps.max_matrix_cols,
        preflight.caps.max_matrix_rows,
        preflight.caps.max_estimated_memory_bytes,
        preflight.planned_stage_count.display_value(),
        preflight.caps.max_materialized_stages,
    )
}

pub fn build_dense_relation_search_schedule(
    j: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    options: &SolverOptions,
) -> DenseRelationSearchSchedule {
    let preflight = estimate_dense_relation_search_schedule(j, eliminated, exported, options);
    let support_descriptors = build_support_descriptors(j, &preflight);
    let mut stages = Vec::new();
    if preflight.materialization_allowed {
        for stage_estimate in &preflight.stage_estimates {
            let bound = RelationSearchBound {
                export_degree: stage_estimate.export_degree,
                multiplier_total_degree: stage_estimate.multiplier_total_degree,
            };
            let export_support =
                build_export_monomial_support(&preflight.exported_variables, &bound);
            let multiplier_supports = build_multiplier_supports(
                j,
                &preflight.eliminated_variables,
                &preflight.exported_variables,
                &bound,
            );
            let row_monomials =
                build_row_monomial_support(j, &export_support, &multiplier_supports);
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
                export_degree: bound.export_degree,
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
    }
    let mut schedule = DenseRelationSearchSchedule {
        eliminated_variables: preflight.eliminated_variables.clone(),
        exported_variables: preflight.exported_variables.clone(),
        z_seed: preflight.z_seed,
        e_cap: preflight.e_cap,
        d_max: preflight.d_max,
        preflight,
        support_descriptors,
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
        schedule.preflight.preflight_hash.0.to_vec(),
    ];
    for variable in &schedule.eliminated_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for variable in &schedule.exported_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for descriptor in &schedule.support_descriptors {
        chunks.push(hash_support_descriptor(descriptor).0.to_vec());
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

fn estimate_relation_search_stage(
    relations: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    d_max: usize,
    export_degree: usize,
    caps: &RelationSearchPlanningCaps,
) -> RelationSearchStageEstimate {
    let all_variables = sorted_union(eliminated, exported);
    let multiplier_total_degree = export_degree.saturating_add(d_max);
    let export_cols = monomial_count_total_degree_leq(exported.len(), export_degree);
    let multiplier_col_counts = relations
        .iter()
        .map(|relation| {
            let relation_degree = poly_total_degree(relation) as usize;
            let multiplier_degree = multiplier_total_degree.saturating_sub(relation_degree);
            monomial_count_total_degree_leq(all_variables.len(), multiplier_degree)
        })
        .collect::<Vec<_>>();
    let total_multiplier_cols = multiplier_col_counts
        .iter()
        .copied()
        .fold(SaturatingCount::exact(0), SaturatingCount::add);
    let estimated_matrix_cols = export_cols.add(total_multiplier_cols);
    let estimated_rows_upper_bound = export_cols.add(
        multiplier_col_counts
            .iter()
            .copied()
            .zip(relations.iter())
            .map(|(count, relation)| count.mul_u128(relation.terms.len() as u128))
            .fold(SaturatingCount::exact(0), SaturatingCount::add),
    );
    let estimated_memory_bytes_upper_bound = estimated_rows_upper_bound
        .add(estimated_matrix_cols)
        .mul_u128(128);
    let mut reasons = Vec::new();
    if export_cols.exceeds_usize(caps.max_export_cols) {
        reasons.push(format!(
            "export_cols {} exceeds cap {}",
            export_cols.display_value(),
            caps.max_export_cols
        ));
    }
    if total_multiplier_cols.exceeds_usize(caps.max_total_multiplier_cols) {
        reasons.push(format!(
            "total_multiplier_cols {} exceeds cap {}",
            total_multiplier_cols.display_value(),
            caps.max_total_multiplier_cols
        ));
    }
    if estimated_matrix_cols.exceeds_usize(caps.max_matrix_cols) {
        reasons.push(format!(
            "estimated_matrix_cols {} exceeds cap {}",
            estimated_matrix_cols.display_value(),
            caps.max_matrix_cols
        ));
    }
    if estimated_rows_upper_bound.exceeds_usize(caps.max_matrix_rows) {
        reasons.push(format!(
            "estimated_rows {} exceeds cap {}",
            estimated_rows_upper_bound.display_value(),
            caps.max_matrix_rows
        ));
    }
    if estimated_memory_bytes_upper_bound.exceeds_u64(caps.max_estimated_memory_bytes) {
        reasons.push(format!(
            "estimated_memory_bytes {} exceeds cap {}",
            estimated_memory_bytes_upper_bound.display_value(),
            caps.max_estimated_memory_bytes
        ));
    }
    let feasible = reasons.is_empty();
    let mut estimate = RelationSearchStageEstimate {
        export_degree,
        multiplier_total_degree,
        export_cols,
        multiplier_col_counts,
        total_multiplier_cols,
        estimated_matrix_cols,
        estimated_rows_upper_bound,
        estimated_memory_bytes_upper_bound,
        feasible,
        prohibition_reason: if feasible {
            None
        } else {
            Some(reasons.join("; "))
        },
        estimate_hash: hash_sequence("dense-trs-stage-estimate", &[]),
    };
    estimate.estimate_hash = hash_stage_estimate(&estimate);
    estimate
}

fn build_support_descriptors(
    relations: &[SparsePolynomialQ],
    preflight: &DenseRelationSearchPreflight,
) -> Vec<SupportDescriptor> {
    let all_variables = sorted_union(
        &preflight.eliminated_variables,
        &preflight.exported_variables,
    );
    let mut descriptors = Vec::new();
    for estimate in &preflight.stage_estimates {
        descriptors.push(SupportDescriptor::DenseTotalDegree {
            variables: preflight.exported_variables.clone(),
            degree: estimate.export_degree,
            estimated_count: estimate.export_cols,
        });
        for (relation, count) in relations.iter().zip(estimate.multiplier_col_counts.iter()) {
            let relation_degree = poly_total_degree(relation) as usize;
            descriptors.push(SupportDescriptor::DenseTotalDegree {
                variables: all_variables.clone(),
                degree: estimate
                    .multiplier_total_degree
                    .saturating_sub(relation_degree),
                estimated_count: *count,
            });
        }
    }
    descriptors
}

fn monomial_count_total_degree_leq(variable_count: usize, max_degree: usize) -> SaturatingCount {
    binomial_saturating(variable_count.saturating_add(max_degree), max_degree)
}

fn binomial_saturating(n: usize, k: usize) -> SaturatingCount {
    if k > n {
        return SaturatingCount::exact(0);
    }
    let k = k.min(n - k);
    let mut result = 1_u128;
    for i in 1..=k {
        let numerator = (n - k + i) as u128;
        let Some(product) = result.checked_mul(numerator) else {
            return SaturatingCount::saturated();
        };
        result = product / i as u128;
    }
    SaturatingCount::exact(result)
}

fn hash_dense_relation_search_preflight(preflight: &DenseRelationSearchPreflight) -> Hash {
    let mut chunks = vec![
        preflight.z_seed.to_be_bytes().to_vec(),
        preflight.e_cap.to_be_bytes().to_vec(),
        preflight.d_max.to_be_bytes().to_vec(),
        count_to_bytes(preflight.planned_stage_count),
        preflight.estimated_stage_count.to_be_bytes().to_vec(),
        preflight
            .first_prohibited_stage
            .unwrap_or(usize::MAX)
            .to_be_bytes()
            .to_vec(),
        (preflight.materialization_allowed as u8)
            .to_be_bytes()
            .to_vec(),
        preflight.caps.max_export_cols.to_be_bytes().to_vec(),
        preflight
            .caps
            .max_total_multiplier_cols
            .to_be_bytes()
            .to_vec(),
        preflight.caps.max_matrix_rows.to_be_bytes().to_vec(),
        preflight.caps.max_matrix_cols.to_be_bytes().to_vec(),
        preflight
            .caps
            .max_estimated_memory_bytes
            .to_be_bytes()
            .to_vec(),
        preflight
            .caps
            .max_materialized_stages
            .to_be_bytes()
            .to_vec(),
    ];
    for variable in &preflight.eliminated_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for variable in &preflight.exported_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for estimate in &preflight.stage_estimates {
        chunks.push(estimate.estimate_hash.0.to_vec());
        chunks.push(hash_stage_estimate(estimate).0.to_vec());
    }
    hash_sequence("dense-trs-preflight", &chunks)
}

fn hash_stage_estimate(estimate: &RelationSearchStageEstimate) -> Hash {
    let mut chunks = vec![
        estimate.export_degree.to_be_bytes().to_vec(),
        estimate.multiplier_total_degree.to_be_bytes().to_vec(),
        count_to_bytes(estimate.export_cols),
        count_to_bytes(estimate.total_multiplier_cols),
        count_to_bytes(estimate.estimated_matrix_cols),
        count_to_bytes(estimate.estimated_rows_upper_bound),
        count_to_bytes(estimate.estimated_memory_bytes_upper_bound),
        (estimate.feasible as u8).to_be_bytes().to_vec(),
    ];
    for count in &estimate.multiplier_col_counts {
        chunks.push(count_to_bytes(*count));
    }
    chunks.push(Vec::new());
    chunks.push(
        estimate
            .prohibition_reason
            .as_deref()
            .unwrap_or("")
            .as_bytes()
            .to_vec(),
    );
    hash_sequence("dense-trs-stage-estimate", &chunks)
}

fn hash_support_descriptor(descriptor: &SupportDescriptor) -> Hash {
    match descriptor {
        SupportDescriptor::DenseTotalDegree {
            variables,
            degree,
            estimated_count,
        } => {
            let mut chunks = vec![
                b"DenseTotalDegree".to_vec(),
                degree.to_be_bytes().to_vec(),
                count_to_bytes(*estimated_count),
            ];
            for variable in variables {
                chunks.push(variable.0.to_be_bytes().to_vec());
            }
            hash_sequence("support-descriptor", &chunks)
        }
        SupportDescriptor::SparseFootprint {
            variables,
            support_hash,
            estimated_count,
        } => {
            let mut chunks = vec![
                b"SparseFootprint".to_vec(),
                support_hash.0.to_vec(),
                count_to_bytes(*estimated_count),
            ];
            for variable in variables {
                chunks.push(variable.0.to_be_bytes().to_vec());
            }
            hash_sequence("support-descriptor", &chunks)
        }
    }
}

fn count_to_bytes(count: SaturatingCount) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(count.saturated as u8);
    out.extend_from_slice(&count.value.unwrap_or(u128::MAX).to_be_bytes());
    out
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
