use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::hash::Hash;
use crate::types::ids::{BlockId, VariableId};
use crate::types::rational::RationalQ;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolverStatus {
    CertifiedCandidateCover,
    CertifiedNonFiniteTargetImage,
    FiniteResourceFailure,
    AlgorithmicHardCase,
    CertificateDesignGap,
    ImplementationBug,
    InvalidInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgebraicReason(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureKind {
    FiniteResourceFailure {
        stage: StageId,
        block_id: Option<BlockId>,
        matrix_rows: Option<usize>,
        matrix_cols: Option<usize>,
        matrix_density: Option<RationalQ>,
        quotient_rank_estimate: Option<usize>,
        coefficient_height_bits: Option<usize>,
        memory_bytes: Option<u64>,
    },
    AlgorithmicHardCase {
        stage: StageId,
        reason: AlgebraicReason,
        minimal_block_hash: Hash,
    },
    CertificateDesignGap {
        constructed_object_hash: Hash,
        missing_certificate_kind: String,
    },
    ImplementationBug {
        invariant_violated: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolverErrorKind {
    InvalidInput { message: String },
    Failure(FailureKind),
}

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[error("{kind:?}")]
pub struct SolverError {
    pub target: Option<VariableId>,
    pub kind: SolverErrorKind,
}

impl SolverError {
    pub fn invalid_input(target: Option<VariableId>, message: impl Into<String>) -> Self {
        Self {
            target,
            kind: SolverErrorKind::InvalidInput {
                message: message.into(),
            },
        }
    }

    pub fn public_status(&self) -> SolverStatus {
        match &self.kind {
            SolverErrorKind::InvalidInput { .. } => SolverStatus::InvalidInput,
            SolverErrorKind::Failure(FailureKind::FiniteResourceFailure { .. }) => {
                SolverStatus::FiniteResourceFailure
            }
            SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase { .. }) => {
                SolverStatus::AlgorithmicHardCase
            }
            SolverErrorKind::Failure(FailureKind::CertificateDesignGap { .. }) => {
                SolverStatus::CertificateDesignGap
            }
            SolverErrorKind::Failure(FailureKind::ImplementationBug { .. }) => {
                SolverStatus::ImplementationBug
            }
        }
    }
}
