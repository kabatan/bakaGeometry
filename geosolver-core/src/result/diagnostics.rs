use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::result::status::{SolverError, SolverErrorKind, StageId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticRecord {
    pub name: String,
    pub message: String,
    pub stage: Option<StageId>,
    pub details: BTreeMap<String, String>,
}

impl DiagnosticRecord {
    pub fn new(
        name: impl Into<String>,
        message: impl Into<String>,
        stage: Option<StageId>,
    ) -> DiagnosticRecord {
        DiagnosticRecord {
            name: name.into(),
            message: message.into(),
            stage,
            details: BTreeMap::new(),
        }
    }

    pub fn from_solver_error(err: &SolverError) -> DiagnosticRecord {
        match &err.kind {
            SolverErrorKind::InvalidInput { message } => DiagnosticRecord {
                name: "InvalidInput".to_string(),
                message: message.clone(),
                stage: None,
                details: BTreeMap::new(),
            },
            SolverErrorKind::Failure(kind) => DiagnosticRecord {
                name: "FailureKind".to_string(),
                message: format!("{kind:?}"),
                stage: None,
                details: BTreeMap::new(),
            },
        }
    }
}
