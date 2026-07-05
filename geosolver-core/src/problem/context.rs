use crate::result::diagnostics::DiagnosticRecord;
use crate::result::status::{SolverError, StageId};
use crate::solver::options::SolverOptions;
use crate::types::ids::IdCounter;

#[derive(Debug, Clone)]
pub struct SolverContext {
    pub options: SolverOptions,
    pub id_counter: IdCounter,
    pub hash_config: HashConfig,
    pub resource_meter: ResourceMeter,
    pub diagnostics: Vec<DiagnosticRecord>,
}

impl SolverContext {
    pub fn new(options: SolverOptions) -> SolverContext {
        SolverContext {
            options,
            id_counter: IdCounter::new(0),
            hash_config: HashConfig::default(),
            resource_meter: ResourceMeter::default(),
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HashConfig {
    pub domain_separator: String,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceMeter {
    pub observed_memory_bytes: Option<u64>,
}

pub fn new_context(options: SolverOptions) -> SolverContext {
    SolverContext::new(options)
}

pub fn check_resource(_ctx: &mut SolverContext, _stage: StageId) -> Result<(), SolverError> {
    Ok(())
}

pub fn push_diagnostic(ctx: &mut SolverContext, diag: DiagnosticRecord) {
    ctx.diagnostics.push(diag);
}
