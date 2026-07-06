use crate::result::diagnostics::DiagnosticRecord;
use crate::result::status::{FailureKind, SolverError, SolverErrorKind, StageId};
use crate::solver::options::SolverOptions;
use crate::types::hash::Hash;
use crate::types::ids::BlockId;
use crate::types::ids::IdCounter;

#[derive(Debug, Clone)]
pub struct SolverContext {
    pub options: SolverOptions,
    pub id_counter: IdCounter,
    pub hash_config: HashConfig,
    pub resource_meter: ResourceMeter,
    pub diagnostics: Vec<DiagnosticRecord>,
    pub active_route_budget: Option<ActiveRouteBudget>,
}

impl SolverContext {
    pub fn new(options: SolverOptions) -> SolverContext {
        SolverContext {
            options,
            id_counter: IdCounter::new(0),
            hash_config: HashConfig::default(),
            resource_meter: ResourceMeter::default(),
            diagnostics: Vec::new(),
            active_route_budget: None,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveRouteBudget {
    pub block_id: BlockId,
    pub kernel_kind: String,
    pub plan_hash: Hash,
    pub route_budget_hash: Hash,
    pub algebraic_work_estimate_hash: Hash,
    pub max_elapsed_steps: usize,
    pub max_work_units: u128,
    pub consumed_steps: usize,
    pub consumed_work_units: u128,
}

pub fn new_context(options: SolverOptions) -> SolverContext {
    SolverContext::new(options)
}

pub fn begin_route_budget(ctx: &mut SolverContext, budget: ActiveRouteBudget) {
    ctx.active_route_budget = Some(budget);
}

pub fn end_route_budget(ctx: &mut SolverContext) -> Option<ActiveRouteBudget> {
    ctx.active_route_budget.take()
}

pub fn check_resource(ctx: &mut SolverContext, stage: StageId) -> Result<(), SolverError> {
    check_resource_work(ctx, stage, 1)
}

pub fn check_resource_work(
    ctx: &mut SolverContext,
    stage: StageId,
    work_units: u128,
) -> Result<(), SolverError> {
    let Some(active) = &mut ctx.active_route_budget else {
        return Ok(());
    };
    active.consumed_steps = active.consumed_steps.saturating_add(1);
    active.consumed_work_units = active.consumed_work_units.saturating_add(work_units.max(1));
    if active.consumed_steps > active.max_elapsed_steps
        || active.consumed_work_units > active.max_work_units
    {
        return Err(SolverError {
            target: None,
            kind: SolverErrorKind::Failure(FailureKind::FiniteResourceFailure {
                stage: StageId(format!(
                    "RouteBudget::{kernel}::cooperative checkpoint exceeded route budget::{stage}; plan_hash={plan:?}; budget_hash={budget:?}; estimate_hash={estimate:?}; consumed_steps={steps}; max_elapsed_steps={max_steps}; consumed_work_units={work}; max_work_units={max_work}",
                    kernel = active.kernel_kind,
                    stage = stage.0,
                    plan = active.plan_hash,
                    budget = active.route_budget_hash,
                    estimate = active.algebraic_work_estimate_hash,
                    steps = active.consumed_steps,
                    max_steps = active.max_elapsed_steps,
                    work = active.consumed_work_units,
                    max_work = active.max_work_units,
                )),
                block_id: Some(active.block_id),
                matrix_rows: None,
                matrix_cols: None,
                matrix_density: None,
                quotient_rank_estimate: None,
                coefficient_height_bits: None,
                memory_bytes: Some(active.max_work_units.min(u64::MAX as u128) as u64),
            }),
        });
    }
    Ok(())
}

pub fn push_diagnostic(ctx: &mut SolverContext, diag: DiagnosticRecord) {
    ctx.diagnostics.push(diag);
}
