pub mod action_krylov;
pub mod linear_affine;
pub mod norm_trace_projection;
pub mod regular_chain_projection;
pub mod sparse_resultant;
pub mod specialization_interpolation;
pub mod target_relation_search;
pub mod target_univariate;
pub mod traits;
pub mod universal_elimination;

pub use traits::*;

use crate::kernels::action_krylov::TargetActionKrylovKernel;
use crate::kernels::linear_affine::LinearAffineKernel;
use crate::kernels::norm_trace_projection::NormTraceProjectionKernel;
use crate::kernels::regular_chain_projection::RegularChainProjectionKernel;
use crate::kernels::sparse_resultant::SparseResultantProjectionKernel;
use crate::kernels::specialization_interpolation::SpecializationInterpolationKernel;
use crate::kernels::target_relation_search::TargetRelationSearchKernel;
use crate::kernels::target_univariate::TargetUnivariateKernel;
use crate::kernels::universal_elimination::UniversalTargetEliminationKernel;

pub fn all_kernels() -> Vec<Box<dyn TargetProjectionKernel>> {
    vec![
        Box::new(TargetUnivariateKernel),
        Box::new(LinearAffineKernel),
        Box::new(TargetRelationSearchKernel),
        Box::new(SparseResultantProjectionKernel),
        Box::new(TargetActionKrylovKernel),
        Box::new(NormTraceProjectionKernel),
        Box::new(RegularChainProjectionKernel),
        Box::new(SpecializationInterpolationKernel),
        Box::new(UniversalTargetEliminationKernel),
    ]
}

pub fn kernel_by_kind(kind: KernelKind) -> Box<dyn TargetProjectionKernel> {
    match kind {
        KernelKind::TargetUnivariate => Box::new(TargetUnivariateKernel),
        KernelKind::LinearAffine => Box::new(LinearAffineKernel),
        KernelKind::TargetRelationSearch => Box::new(TargetRelationSearchKernel),
        KernelKind::SparseResultantProjection => Box::new(SparseResultantProjectionKernel),
        KernelKind::TargetActionKrylov => Box::new(TargetActionKrylovKernel),
        KernelKind::NormTraceProjection => Box::new(NormTraceProjectionKernel),
        KernelKind::RegularChainProjection => Box::new(RegularChainProjectionKernel),
        KernelKind::SpecializationInterpolation => Box::new(SpecializationInterpolationKernel),
        KernelKind::UniversalTargetElimination => Box::new(UniversalTargetEliminationKernel),
    }
}

#[cfg(test)]
mod tests {
    use super::{all_kernels, kernel_by_kind};
    use crate::kernels::traits::KernelKind;

    #[test]
    fn p7_registry_lists_all_nine_kernels_in_appendix_order() {
        let kinds = all_kernels()
            .into_iter()
            .map(|kernel| kernel.kind())
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                KernelKind::TargetUnivariate,
                KernelKind::LinearAffine,
                KernelKind::TargetRelationSearch,
                KernelKind::SparseResultantProjection,
                KernelKind::TargetActionKrylov,
                KernelKind::NormTraceProjection,
                KernelKind::RegularChainProjection,
                KernelKind::SpecializationInterpolation,
                KernelKind::UniversalTargetElimination,
            ]
        );
        for kind in kinds {
            assert_eq!(kernel_by_kind(kind).kind(), kind);
        }
    }
}
