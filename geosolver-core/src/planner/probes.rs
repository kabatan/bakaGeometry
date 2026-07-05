use serde::{Deserialize, Serialize};

use crate::graph::metrics::{
    estimate_coefficient_growth, estimate_local_quotient_rank, estimate_sparse_template_size,
    structural_metrics, HeightEstimate, RankEstimate, StructuralMetrics, TemplateEstimate,
};
use crate::graph::projection_dag::ProjectionBlock;
use crate::preprocess::compression::CompressedSystemQ;
use crate::problem::context::SolverContext;
use crate::types::hash::{hash_sequence, Hash};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeResults {
    pub structural: StructuralMetrics,
    pub modular_rank: RankProbeResult,
    pub local_macaulay_size: MacaulaySizeProbe,
    pub mixed_support: SparseSupportProbe,
    pub coefficient_growth: HeightEstimate,
    pub probe_hash: Hash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prime(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RankProbeResult {
    pub primes: Vec<Prime>,
    pub rank_estimate: RankEstimate,
    pub probe_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacaulaySizeProbe {
    pub template_estimate: TemplateEstimate,
    pub probe_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparseSupportProbe {
    pub template_estimate: TemplateEstimate,
    pub projected_nonzero_hint: usize,
    pub probe_hash: Hash,
}

pub fn run_cost_probes(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    _ctx: &mut SolverContext,
) -> ProbeResults {
    let structural = structural_metrics(block, system);
    let modular_rank = modular_rank_probe(block, &[Prime(2), Prime(3), Prime(5)]);
    let local_macaulay_size = local_macaulay_size_probe(block);
    let mixed_support = mixed_support_probe(block);
    let coefficient_growth = estimate_coefficient_growth(block);
    let probe_hash = hash_sequence(
        "planner-probe-results",
        &[
            structural.metrics_hash.0.to_vec(),
            modular_rank.probe_hash.0.to_vec(),
            local_macaulay_size.probe_hash.0.to_vec(),
            mixed_support.probe_hash.0.to_vec(),
            coefficient_growth.estimate_hash.0.to_vec(),
        ],
    );
    ProbeResults {
        structural,
        modular_rank,
        local_macaulay_size,
        mixed_support,
        coefficient_growth,
        probe_hash,
    }
}

pub fn modular_rank_probe(block: &ProjectionBlock, primes: &[Prime]) -> RankProbeResult {
    let rank_estimate = estimate_local_quotient_rank(block);
    let mut chunks = vec![rank_estimate.estimate_hash.0.to_vec()];
    for prime in primes {
        chunks.push(prime.0.to_be_bytes().to_vec());
    }
    let probe_hash = hash_sequence("modular-rank-probe", &chunks);
    RankProbeResult {
        primes: primes.to_vec(),
        rank_estimate,
        probe_hash,
    }
}

pub fn local_macaulay_size_probe(block: &ProjectionBlock) -> MacaulaySizeProbe {
    let template_estimate = estimate_sparse_template_size(block);
    let probe_hash = hash_sequence(
        "local-macaulay-size-probe",
        &[template_estimate.estimate_hash.0.to_vec()],
    );
    MacaulaySizeProbe {
        template_estimate,
        probe_hash,
    }
}

pub fn mixed_support_probe(block: &ProjectionBlock) -> SparseSupportProbe {
    let template_estimate = estimate_sparse_template_size(block);
    let projected_nonzero_hint = template_estimate
        .nonzero_hint
        .saturating_sub(block.exported_variables.len());
    let probe_hash = hash_sequence(
        "mixed-support-probe",
        &[
            template_estimate.estimate_hash.0.to_vec(),
            projected_nonzero_hint.to_be_bytes().to_vec(),
        ],
    );
    SparseSupportProbe {
        template_estimate,
        projected_nonzero_hint,
        probe_hash,
    }
}
