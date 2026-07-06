use serde::{Deserialize, Serialize};

use crate::kernels::traits::KernelKind;
use crate::preprocess::compression::CompressionTrace;
use crate::result::cost_trace::ProjectionCostTrace;
use crate::types::hash::Hash;
use crate::types::ids::{BlockId, PackageId, RelationId, VariableId};
use crate::types::polynomial::SparsePolynomialQ;
use crate::verify::certificates::KernelCertificate;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionMessage {
    pub package_id: PackageId,
    pub block_id: BlockId,
    pub kernel_kind: KernelKind,
    pub source_relation_ids: Vec<RelationId>,
    pub eliminated_variables: Vec<VariableId>,
    pub exported_variables: Vec<VariableId>,
    pub relation_generators: Vec<SparsePolynomialQ>,
    pub representation: MessageRepresentation,
    pub projection_strength: ProjectionStrength,
    pub certificate: KernelCertificate,
    pub compression_trace: CompressionTrace,
    pub cost_trace: ProjectionCostTrace,
    pub package_hash: Hash,
}

pub fn hash_projection_message(message: &ProjectionMessage) -> Hash {
    let mut chunks = vec![
        message.package_id.0.to_be_bytes().to_vec(),
        message.block_id.0.to_be_bytes().to_vec(),
        format!("{:?}", message.kernel_kind).into_bytes(),
        message.certificate.certificate_hash.0.to_vec(),
    ];
    for relation in &message.relation_generators {
        chunks.push(relation.hash.0.to_vec());
    }
    if let Some(route_cost) = &message.cost_trace.route_cost {
        chunks.push(route_cost.algebraic_work_estimate_hash.0.to_vec());
        chunks.push(route_cost.route_budget_hash.0.to_vec());
        chunks.push(route_cost.predicted_work_units.0.to_be_bytes().to_vec());
        chunks.push(
            route_cost
                .route_budget_max_work_units
                .0
                .to_be_bytes()
                .to_vec(),
        );
        chunks.push(
            route_cost
                .route_budget_max_intermediate_terms
                .to_be_bytes()
                .to_vec(),
        );
        chunks.push(
            route_cost
                .route_budget_max_output_terms
                .to_be_bytes()
                .to_vec(),
        );
    }
    crate::types::hash::hash_sequence("projection-message", &chunks)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRepresentation {
    GeneratorSet,
    PrincipalSupport,
    TriangularChain,
    QuotientAction,
    NormTraceTower,
    SparseResultantMatrix,
    SpecializationInterpolation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionStrength {
    CandidateCoverWeak,
    CandidateCoverStrong,
    RadicalProjectionApprox,
    ExactProjectionIdeal,
    ExactRealFiberAware,
}
