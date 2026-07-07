#[derive(Clone, Debug)]
pub struct SolverOptions {
    pub resource_limits: ResourceLimits,
    pub exact_image_mode: ExactImageMode,
}

#[derive(Clone, Debug)]
pub struct ResourceLimits {
    pub max_window_degree: Option<usize>,
    pub max_proof_weight: Option<usize>,
    pub max_matrix_rows: Option<usize>,
    pub max_matrix_cols: Option<usize>,
    pub max_candidate_count: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExactImageMode {
    CoverOnly,
    TryExactImage,
    RequireExactImage,
}
