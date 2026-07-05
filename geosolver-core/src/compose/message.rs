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
