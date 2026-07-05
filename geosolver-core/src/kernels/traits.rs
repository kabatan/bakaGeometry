use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KernelKind {
    TargetUnivariate,
    LinearAffine,
    TargetRelationSearch,
    SparseResultantProjection,
    TargetActionKrylov,
    UniversalTargetElimination,
    RegularChainProjection,
    NormTraceProjection,
    SpecializationInterpolation,
}
