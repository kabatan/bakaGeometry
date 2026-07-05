use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolverOptions {
    pub exact_image_mode: bool,
    pub max_memory_bytes: Option<u64>,
    pub max_matrix_rows: Option<usize>,
    pub max_matrix_cols: Option<usize>,
    pub max_coefficient_height_bits: Option<usize>,
    pub root_isolation_method: RootIsolationMethod,
    pub certificate_level: CertificateLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RootIsolationMethod {
    Sturm,
    Descartes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateLevel {
    Minimal,
    Full,
}

impl Default for SolverOptions {
    fn default() -> Self {
        SolverOptions {
            exact_image_mode: false,
            max_memory_bytes: None,
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_coefficient_height_bits: None,
            root_isolation_method: RootIsolationMethod::Sturm,
            certificate_level: CertificateLevel::Minimal,
        }
    }
}
