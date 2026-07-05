#[cfg(test)]
pub mod action_krylov;
pub mod linear_affine;
#[cfg(test)]
pub mod norm_trace_projection;
#[cfg(test)]
pub mod regular_chain_projection;
#[cfg(test)]
pub mod sparse_resultant;
#[cfg(test)]
pub mod specialization_interpolation;
pub mod target_relation_search;
pub mod target_univariate;
pub mod traits;
#[cfg(test)]
pub mod universal_elimination;

pub use traits::*;

use crate::kernels::linear_affine::LinearAffineKernel;
use crate::kernels::target_univariate::TargetUnivariateKernel;

#[cfg(test)]
use crate::kernels::action_krylov::TargetActionKrylovKernel;
#[cfg(test)]
use crate::kernels::norm_trace_projection::NormTraceProjectionKernel;
#[cfg(test)]
use crate::kernels::regular_chain_projection::RegularChainProjectionKernel;
#[cfg(test)]
use crate::kernels::sparse_resultant::SparseResultantProjectionKernel;
#[cfg(test)]
use crate::kernels::specialization_interpolation::SpecializationInterpolationKernel;
#[cfg(test)]
use crate::kernels::target_relation_search::TargetRelationSearchKernel;
#[cfg(test)]
use crate::kernels::universal_elimination::UniversalTargetEliminationKernel;

pub fn all_kernels() -> Vec<Box<dyn TargetProjectionKernel>> {
    vec![
        Box::new(TargetUnivariateKernel),
        Box::new(LinearAffineKernel),
    ]
}

#[cfg(test)]
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
    fn fcr_p2_production_registry_excludes_partial_kernels() {
        let kinds = all_kernels()
            .into_iter()
            .map(|kernel| kernel.kind())
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![KernelKind::TargetUnivariate, KernelKind::LinearAffine]
        );
    }

    #[test]
    fn fcr_p7_advanced_projection_kernels_are_not_completion_registry_claims() {
        let kinds = all_kernels()
            .into_iter()
            .map(|kernel| kernel.kind())
            .collect::<Vec<_>>();
        for advanced in [
            KernelKind::SparseResultantProjection,
            KernelKind::NormTraceProjection,
            KernelKind::RegularChainProjection,
            KernelKind::SpecializationInterpolation,
        ] {
            assert!(
                !kinds.contains(&advanced),
                "{advanced:?} must remain outside production all_kernels until generic contract is claimed"
            );
        }
    }

    #[test]
    fn test_lookup_still_constructs_quarantined_kernel_kinds() {
        for kind in [
            KernelKind::TargetUnivariate,
            KernelKind::LinearAffine,
            KernelKind::TargetRelationSearch,
            KernelKind::SparseResultantProjection,
            KernelKind::TargetActionKrylov,
            KernelKind::NormTraceProjection,
            KernelKind::RegularChainProjection,
            KernelKind::SpecializationInterpolation,
            KernelKind::UniversalTargetElimination,
        ] {
            assert_eq!(kernel_by_kind(kind).kind(), kind);
        }
    }
}
