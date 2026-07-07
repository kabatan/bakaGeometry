use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::preprocess::binomial::simplify_binomial_relations;
use crate::preprocess::definitional::apply_definitional_elimination;
use crate::preprocess::independent::mark_target_independent_components;
use crate::preprocess::linear_affine::eliminate_linear_affine_variables;
use crate::preprocess::saturation::apply_explicit_saturations;
use crate::problem::canonicalize::{CanonicalRelationQ, CanonicalSystemQ, RelationSource};
use crate::problem::context::SolverContext;
use crate::problem::semantic::RealConstraintEncoding;
use crate::result::diagnostics::DiagnosticRecord;
use crate::result::status::{FailureKind, SolverError, SolverErrorKind};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{RelationId, VariableId};
use crate::types::monomial::normalize_monomial;
use crate::types::polynomial::{
    clear_denominators_primitive, constant_poly, max_poly_coefficient_height_bits, normalize_poly,
    poly_add, poly_monomial_count, poly_mul, poly_total_degree, poly_variables, substitute_poly,
    SparsePolynomialQ, SubstitutionMap, TermQ,
};
use crate::types::rational::int_q;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressionTrace {
    pub step_count: usize,
    pub ordered_steps: Vec<CompressionStepKind>,
    pub substitution_count: usize,
    pub guarded_rational_affine_pivot_count: usize,
    pub guard_count: usize,
    pub saturation_count: usize,
    pub target_independent_component_count: usize,
    pub monomial_count_before: usize,
    pub monomial_count_after: usize,
    pub coefficient_height_before_bits: usize,
    pub coefficient_height_after_bits: usize,
    pub trace_hash: Hash,
}

impl Default for CompressionTrace {
    fn default() -> Self {
        Self {
            step_count: 0,
            ordered_steps: Vec::new(),
            substitution_count: 0,
            guarded_rational_affine_pivot_count: 0,
            guard_count: 0,
            saturation_count: 0,
            target_independent_component_count: 0,
            monomial_count_before: 0,
            monomial_count_after: 0,
            coefficient_height_before_bits: 0,
            coefficient_height_after_bits: 0,
            trace_hash: hash_sequence("compression-trace", &[]),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionStepKind {
    DefinitionalElimination,
    LinearAffineElimination,
    BinomialSimplification,
    ExplicitSaturation,
    TargetIndependentComponentMarking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressedSystemQ {
    pub variables: Vec<VariableId>,
    pub target: VariableId,
    pub relations: Vec<CanonicalRelationQ>,
    pub relation_order: Vec<RelationId>,
    pub semantic_encodings: Vec<RealConstraintEncoding>,
    pub substitutions: Vec<CompressionSubstitution>,
    pub guards: Vec<GuardRecord>,
    pub rational_affine_transformations: Vec<RationalAffineTransformationCertificate>,
    pub saturations: Vec<SaturationRecord>,
    pub feasibility_obligations: Vec<FeasibilityObligation>,
    pub diagnostics: Vec<DiagnosticRecord>,
    pub compression_trace: CompressionTrace,
    pub compressed_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressionState {
    pub variables: Vec<VariableId>,
    pub target: VariableId,
    pub relations: Vec<CanonicalRelationQ>,
    pub relation_order: Vec<RelationId>,
    pub semantic_encodings: Vec<RealConstraintEncoding>,
    pub substitutions: Vec<CompressionSubstitution>,
    pub guards: Vec<GuardRecord>,
    pub rational_affine_transformations: Vec<RationalAffineTransformationCertificate>,
    pub saturations: Vec<SaturationRecord>,
    pub target_independent_components: Vec<Component>,
    pub feasibility_obligations: Vec<FeasibilityObligation>,
    pub diagnostics: Vec<DiagnosticRecord>,
    pub trace: CompressionTrace,
    next_relation_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressionSubstitution {
    pub eliminated_variable: VariableId,
    pub expression: CompressionExpression,
    pub denominator_guard: Option<SparsePolynomialQ>,
    pub source_relation_id: RelationId,
    pub kind: SubstitutionKind,
    pub substitution_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionExpression {
    Polynomial(SparsePolynomialQ),
    Rational(RationalExpressionQ),
}

impl CompressionExpression {
    pub fn expression_hash(&self) -> Hash {
        match self {
            CompressionExpression::Polynomial(poly) => poly.hash,
            CompressionExpression::Rational(expr) => expr.expression_hash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RationalExpressionQ {
    pub numerator: SparsePolynomialQ,
    pub denominator: SparsePolynomialQ,
    pub denominator_guard: GuardRecord,
    pub source_witness_relation_ids: Vec<RelationId>,
    pub expression_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RationalAffineTransformationCertificate {
    pub original_relation_id: RelationId,
    pub transformed_relation_id: RelationId,
    pub pivot_relation_id: RelationId,
    pub eliminated_variable: VariableId,
    pub numerator: SparsePolynomialQ,
    pub denominator: SparsePolynomialQ,
    pub denominator_clearing_power: u32,
    pub denominator_guard: GuardRecord,
    pub source_witness_relation_ids: Vec<RelationId>,
    pub original_relation_hash: Hash,
    pub transformed_relation_hash: Hash,
    pub transformation_hash: Hash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubstitutionKind {
    Definitional,
    LinearAffineConstantPivot,
    LinearAffineGuardedPivot,
    Binomial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardRecord {
    pub factor: SparsePolynomialQ,
    pub source_relation_ids: Vec<RelationId>,
    pub guard_kind: GuardKind,
    pub guard_hash: Hash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardKind {
    ConstantNonZeroPivot,
    ExplicitNonZeroWitness,
    AffineDenominator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaturationRecord {
    pub factor: SparsePolynomialQ,
    pub witness_relation_id: RelationId,
    pub slack_variable: VariableId,
    pub saturation_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub component_id: usize,
    pub relation_ids: Vec<RelationId>,
    pub variables: Vec<VariableId>,
    pub contains_target: bool,
    pub component_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeasibilityObligation {
    pub component_id: usize,
    pub relation_ids: Vec<RelationId>,
    pub variables: Vec<VariableId>,
    pub obligation_hash: Hash,
}

pub fn pre_kernel_compress(
    system: CanonicalSystemQ,
    ctx: &mut SolverContext,
) -> Result<CompressedSystemQ, SolverError> {
    let mut state = CompressionState::from_system(system);
    state.trace.monomial_count_before = total_monomial_count(&state.relations);
    state.trace.coefficient_height_before_bits = max_coefficient_height_bits(&state.relations);

    state = apply_definitional_elimination(state, &[], ctx)?;
    state.record_step(CompressionStepKind::DefinitionalElimination);

    state = eliminate_linear_affine_variables(state, ctx)?;
    state.record_step(CompressionStepKind::LinearAffineElimination);

    state = simplify_binomial_relations(state, ctx)?;
    state.record_step(CompressionStepKind::BinomialSimplification);

    state = apply_explicit_saturations(state, ctx)?;
    state.record_step(CompressionStepKind::ExplicitSaturation);

    state = mark_target_independent_components(state, ctx)?;
    state.record_step(CompressionStepKind::TargetIndependentComponentMarking);

    state.trace.monomial_count_after = total_monomial_count(&state.relations);
    state.trace.coefficient_height_after_bits = max_coefficient_height_bits(&state.relations);
    state.trace.substitution_count = state.substitutions.len();
    state.trace.guarded_rational_affine_pivot_count = state
        .substitutions
        .iter()
        .filter(|sub| matches!(sub.expression, CompressionExpression::Rational(_)))
        .count();
    state.trace.guard_count = state.guards.len();
    state.trace.saturation_count = state.saturations.len();
    state.trace.target_independent_component_count = state.target_independent_components.len();
    state.trace.step_count = state.trace.ordered_steps.len();
    enforce_height_limit(&state, ctx)?;
    Ok(state.to_compressed_system())
}

pub fn compress_system(
    system: CanonicalSystemQ,
    ctx: &mut SolverContext,
) -> Result<CompressedSystemQ, SolverError> {
    pre_kernel_compress(system, ctx)
}

impl CompressionState {
    pub fn from_system(system: CanonicalSystemQ) -> Self {
        let next_relation_id = system
            .relations
            .iter()
            .map(|relation| relation.id.0)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        Self {
            variables: system.variables,
            target: system.target,
            relations: system.relations,
            relation_order: system.relation_order,
            semantic_encodings: system.semantic_encodings,
            substitutions: Vec::new(),
            guards: Vec::new(),
            rational_affine_transformations: Vec::new(),
            saturations: Vec::new(),
            target_independent_components: Vec::new(),
            feasibility_obligations: Vec::new(),
            diagnostics: system.diagnostics,
            trace: CompressionTrace::default(),
            next_relation_id,
        }
    }

    pub fn to_canonical_system(&self) -> CanonicalSystemQ {
        CanonicalSystemQ {
            variables: self.variables.clone(),
            target: self.target,
            relations: self.relations.clone(),
            relation_order: self.relation_order.clone(),
            variable_order: crate::problem::canonicalize::canonical_variable_order(
                &self.variables,
                self.target,
            ),
            semantic_encodings: self.semantic_encodings.clone(),
            canonical_hash: self.state_hash("canonical-view"),
            diagnostics: self.diagnostics.clone(),
        }
    }

    pub fn to_compressed_system(mut self) -> CompressedSystemQ {
        self.rehash();
        let compressed_hash = self.state_hash("compressed-system");
        CompressedSystemQ {
            variables: self.variables,
            target: self.target,
            relations: self.relations,
            relation_order: self.relation_order,
            semantic_encodings: self.semantic_encodings,
            substitutions: self.substitutions,
            guards: self.guards,
            rational_affine_transformations: self.rational_affine_transformations,
            saturations: self.saturations,
            feasibility_obligations: self.feasibility_obligations,
            diagnostics: self.diagnostics,
            compression_trace: self.trace,
            compressed_hash,
        }
    }

    pub fn record_step(&mut self, step: CompressionStepKind) {
        self.trace.ordered_steps.push(step);
        self.trace.step_count = self.trace.ordered_steps.len();
    }

    pub fn add_substitution(
        &mut self,
        eliminated_variable: VariableId,
        expression: SparsePolynomialQ,
        denominator_guard: Option<SparsePolynomialQ>,
        source_relation_id: RelationId,
        kind: SubstitutionKind,
    ) {
        self.add_substitution_expression(
            eliminated_variable,
            CompressionExpression::Polynomial(expression),
            denominator_guard,
            source_relation_id,
            kind,
        );
    }

    pub fn add_rational_substitution(
        &mut self,
        eliminated_variable: VariableId,
        expression: RationalExpressionQ,
        denominator_guard: Option<SparsePolynomialQ>,
        source_relation_id: RelationId,
        kind: SubstitutionKind,
    ) {
        self.add_substitution_expression(
            eliminated_variable,
            CompressionExpression::Rational(expression),
            denominator_guard,
            source_relation_id,
            kind,
        );
    }

    fn add_substitution_expression(
        &mut self,
        eliminated_variable: VariableId,
        expression: CompressionExpression,
        denominator_guard: Option<SparsePolynomialQ>,
        source_relation_id: RelationId,
        kind: SubstitutionKind,
    ) {
        let substitution_hash = hash_substitution(
            eliminated_variable,
            &expression,
            denominator_guard.as_ref(),
            source_relation_id,
            kind,
        );
        self.substitutions.push(CompressionSubstitution {
            eliminated_variable,
            expression,
            denominator_guard,
            source_relation_id,
            kind,
            substitution_hash,
        });
        self.variables.retain(|var| *var != eliminated_variable);
    }

    pub fn add_guard(
        &mut self,
        factor: SparsePolynomialQ,
        source_relation_ids: Vec<RelationId>,
        guard_kind: GuardKind,
    ) -> GuardRecord {
        let factor = clear_denominators_primitive(&factor);
        let guard_hash = hash_guard(&factor, &source_relation_ids, guard_kind);
        if let Some(existing) = self
            .guards
            .iter()
            .find(|guard| guard.guard_hash == guard_hash)
        {
            return existing.clone();
        }
        let record = GuardRecord {
            factor,
            source_relation_ids,
            guard_kind,
            guard_hash,
        };
        self.guards.push(record.clone());
        record
    }

    pub fn add_saturation(
        &mut self,
        factor: SparsePolynomialQ,
        witness_relation_id: RelationId,
        slack_variable: VariableId,
    ) {
        let factor = clear_denominators_primitive(&factor);
        let saturation_hash = hash_saturation(&factor, witness_relation_id, slack_variable);
        if !self
            .saturations
            .iter()
            .any(|sat| sat.saturation_hash == saturation_hash)
        {
            self.saturations.push(SaturationRecord {
                factor,
                witness_relation_id,
                slack_variable,
                saturation_hash,
            });
        }
    }

    pub fn apply_polynomial_substitution(
        &mut self,
        eliminated_variable: VariableId,
        expression: &SparsePolynomialQ,
        source_relation_id: RelationId,
    ) {
        let mut subst = SubstitutionMap::new();
        subst.insert(eliminated_variable, expression.clone());
        self.relations = self
            .relations
            .iter()
            .filter(|relation| relation.id != source_relation_id)
            .filter_map(|relation| {
                let substituted =
                    clear_denominators_primitive(&substitute_poly(&relation.polynomial, &subst));
                if substituted.terms.is_empty() {
                    None
                } else {
                    Some(relation_with_polynomial(
                        relation.id,
                        substituted,
                        relation.source.clone(),
                    ))
                }
            })
            .collect();
        self.relation_order = self.relations.iter().map(|relation| relation.id).collect();
    }

    pub fn apply_rational_affine_substitution(
        &mut self,
        eliminated_variable: VariableId,
        numerator: &SparsePolynomialQ,
        denominator: &SparsePolynomialQ,
        source_relation_id: RelationId,
        denominator_guard: GuardRecord,
        source_witness_relation_ids: Vec<RelationId>,
    ) {
        self.relations = self
            .relations
            .iter()
            .filter(|relation| relation.id != source_relation_id)
            .filter_map(|relation| {
                let (raw_transformed, denominator_clearing_power) =
                    substitute_rational_and_clear_with_power(
                        &relation.polynomial,
                        eliminated_variable,
                        numerator,
                        denominator,
                    );
                let transformed = clear_denominators_primitive(&raw_transformed);
                if transformed.terms.is_empty() {
                    None
                } else {
                    let transformed_relation =
                        relation_with_polynomial(relation.id, transformed, relation.source.clone());
                    self.rational_affine_transformations.push(
                        rational_affine_transformation_certificate(
                            relation.id,
                            relation.id,
                            source_relation_id,
                            eliminated_variable,
                            numerator.clone(),
                            denominator.clone(),
                            denominator_clearing_power,
                            denominator_guard.clone(),
                            source_witness_relation_ids.clone(),
                            relation.hash,
                            transformed_relation.hash,
                        ),
                    );
                    Some(transformed_relation)
                }
            })
            .collect();
        self.relation_order = self.relations.iter().map(|relation| relation.id).collect();
    }

    pub fn replace_relations(&mut self, relations: Vec<CanonicalRelationQ>) {
        self.relations = relations;
        self.relation_order = self.relations.iter().map(|relation| relation.id).collect();
    }

    pub fn rehash(&mut self) {
        self.relation_order = self.relations.iter().map(|relation| relation.id).collect();
        self.trace.trace_hash = self.state_hash("compression-trace");
    }

    pub fn state_hash(&self, tag: &str) -> Hash {
        let mut chunks = Vec::new();
        chunks.push(self.target.0.to_be_bytes().to_vec());
        for variable in &self.variables {
            chunks.push(variable.0.to_be_bytes().to_vec());
        }
        for relation in &self.relations {
            chunks.push(relation.hash.0.to_vec());
        }
        for encoding in &self.semantic_encodings {
            chunks.push(encoding.semantic_hash.0.to_vec());
        }
        for substitution in &self.substitutions {
            chunks.push(substitution.substitution_hash.0.to_vec());
        }
        for guard in &self.guards {
            chunks.push(guard.guard_hash.0.to_vec());
        }
        for transform in &self.rational_affine_transformations {
            chunks.push(transform.transformation_hash.0.to_vec());
        }
        for sat in &self.saturations {
            chunks.push(sat.saturation_hash.0.to_vec());
        }
        for obligation in &self.feasibility_obligations {
            chunks.push(obligation.obligation_hash.0.to_vec());
        }
        hash_sequence(tag, &chunks)
    }
}

pub fn relation_with_polynomial(
    id: RelationId,
    polynomial: SparsePolynomialQ,
    source: RelationSource,
) -> CanonicalRelationQ {
    let polynomial = clear_denominators_primitive(&polynomial);
    let hash = hash_sequence(
        "canonical-relation",
        &[id.0.to_be_bytes().to_vec(), polynomial.hash.0.to_vec()],
    );
    CanonicalRelationQ {
        id,
        polynomial,
        source,
        hash,
    }
}

pub fn total_monomial_count(relations: &[CanonicalRelationQ]) -> usize {
    relations
        .iter()
        .map(|relation| poly_monomial_count(&relation.polynomial))
        .sum()
}

pub fn max_total_degree(relations: &[CanonicalRelationQ]) -> usize {
    relations
        .iter()
        .map(|relation| poly_total_degree(&relation.polynomial) as usize)
        .max()
        .unwrap_or(0)
}

pub fn max_coefficient_height_bits(relations: &[CanonicalRelationQ]) -> usize {
    max_poly_coefficient_height_bits(
        &relations
            .iter()
            .map(|relation| relation.polynomial.clone())
            .collect::<Vec<_>>(),
    )
}

pub fn component_hash(
    component_id: usize,
    relation_ids: &[RelationId],
    variables: &[VariableId],
    contains_target: bool,
) -> Hash {
    let mut chunks = vec![
        component_id.to_be_bytes().to_vec(),
        vec![contains_target as u8],
    ];
    for relation in relation_ids {
        chunks.push(relation.0.to_be_bytes().to_vec());
    }
    for variable in variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    hash_sequence("compression-component", &chunks)
}

pub fn obligation_hash(
    component_id: usize,
    relation_ids: &[RelationId],
    variables: &[VariableId],
) -> Hash {
    let mut chunks = vec![component_id.to_be_bytes().to_vec()];
    for relation in relation_ids {
        chunks.push(relation.0.to_be_bytes().to_vec());
    }
    for variable in variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    hash_sequence("feasibility-obligation", &chunks)
}

pub fn polynomial_variable_map(
    relations: &[CanonicalRelationQ],
) -> BTreeMap<VariableId, Vec<RelationId>> {
    let mut map: BTreeMap<VariableId, Vec<RelationId>> = BTreeMap::new();
    for relation in relations {
        for variable in poly_variables(&relation.polynomial) {
            map.entry(variable).or_default().push(relation.id);
        }
    }
    map
}

pub fn sort_dedup_variables(vars: impl IntoIterator<Item = VariableId>) -> Vec<VariableId> {
    vars.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn affine_parts_in_variable(
    poly: &SparsePolynomialQ,
    variable: VariableId,
) -> Option<(SparsePolynomialQ, SparsePolynomialQ)> {
    let mut coefficient_terms = Vec::new();
    let mut rest_terms = Vec::new();
    for term in &poly.terms {
        let exponent = term
            .monomial
            .exponents
            .iter()
            .find(|(var, _)| *var == variable)
            .map_or(0, |(_, exp)| *exp);
        match exponent {
            0 => rest_terms.push(term.clone()),
            1 => {
                let reduced = normalize_monomial(
                    term.monomial
                        .exponents
                        .iter()
                        .filter_map(|(var, exp)| {
                            if *var == variable {
                                None
                            } else {
                                Some((*var, *exp))
                            }
                        })
                        .collect(),
                );
                coefficient_terms.push(TermQ {
                    coeff: term.coeff.clone(),
                    monomial: reduced,
                });
            }
            _ => return None,
        }
    }
    let coefficient = normalize_poly(SparsePolynomialQ {
        terms: coefficient_terms,
        hash: hash_sequence("poly", &[]),
    });
    if coefficient.terms.is_empty() {
        return None;
    }
    let rest = normalize_poly(SparsePolynomialQ {
        terms: rest_terms,
        hash: hash_sequence("poly", &[]),
    });
    Some((coefficient, rest))
}

fn enforce_height_limit(state: &CompressionState, ctx: &SolverContext) -> Result<(), SolverError> {
    let Some(limit) = ctx.options.max_coefficient_height_bits else {
        return Ok(());
    };
    let observed = max_coefficient_height_bits(&state.relations);
    if observed <= limit {
        return Ok(());
    }
    Err(SolverError {
        target: Some(state.target),
        kind: SolverErrorKind::Failure(FailureKind::FiniteResourceFailure {
            stage: crate::result::status::StageId("PreKernelAlgebraicCompression".to_owned()),
            block_id: None,
            matrix_rows: None,
            matrix_cols: None,
            matrix_density: None,
            quotient_rank_estimate: None,
            coefficient_height_bits: Some(observed),
            memory_bytes: None,
        }),
    })
}

fn hash_substitution(
    eliminated_variable: VariableId,
    expression: &CompressionExpression,
    denominator_guard: Option<&SparsePolynomialQ>,
    source_relation_id: RelationId,
    kind: SubstitutionKind,
) -> Hash {
    let mut chunks = vec![
        eliminated_variable.0.to_be_bytes().to_vec(),
        source_relation_id.0.to_be_bytes().to_vec(),
        vec![kind as u8],
        expression.expression_hash().0.to_vec(),
    ];
    if let Some(guard) = denominator_guard {
        chunks.push(guard.hash.0.to_vec());
    }
    hash_sequence("compression-substitution", &chunks)
}

pub fn rational_expression(
    numerator: SparsePolynomialQ,
    denominator: SparsePolynomialQ,
    denominator_guard: GuardRecord,
    source_witness_relation_ids: Vec<RelationId>,
) -> RationalExpressionQ {
    let expression_hash = hash_sequence(
        "rational-expression-q",
        &[
            numerator.hash.0.to_vec(),
            denominator.hash.0.to_vec(),
            denominator_guard.guard_hash.0.to_vec(),
        ],
    );
    RationalExpressionQ {
        numerator,
        denominator,
        denominator_guard,
        source_witness_relation_ids,
        expression_hash,
    }
}

pub fn rational_affine_transformation_certificate(
    original_relation_id: RelationId,
    transformed_relation_id: RelationId,
    pivot_relation_id: RelationId,
    eliminated_variable: VariableId,
    numerator: SparsePolynomialQ,
    denominator: SparsePolynomialQ,
    denominator_clearing_power: u32,
    denominator_guard: GuardRecord,
    source_witness_relation_ids: Vec<RelationId>,
    original_relation_hash: Hash,
    transformed_relation_hash: Hash,
) -> RationalAffineTransformationCertificate {
    let transformation_hash = hash_sequence(
        "rational-affine-transformation-certificate",
        &[
            original_relation_id.0.to_be_bytes().to_vec(),
            transformed_relation_id.0.to_be_bytes().to_vec(),
            pivot_relation_id.0.to_be_bytes().to_vec(),
            eliminated_variable.0.to_be_bytes().to_vec(),
            numerator.hash.0.to_vec(),
            denominator.hash.0.to_vec(),
            denominator_clearing_power.to_be_bytes().to_vec(),
            denominator_guard.guard_hash.0.to_vec(),
            original_relation_hash.0.to_vec(),
            transformed_relation_hash.0.to_vec(),
        ],
    );
    RationalAffineTransformationCertificate {
        original_relation_id,
        transformed_relation_id,
        pivot_relation_id,
        eliminated_variable,
        numerator,
        denominator,
        denominator_clearing_power,
        denominator_guard,
        source_witness_relation_ids,
        original_relation_hash,
        transformed_relation_hash,
        transformation_hash,
    }
}

pub fn substitute_rational_and_clear(
    poly: &SparsePolynomialQ,
    variable: VariableId,
    numerator: &SparsePolynomialQ,
    denominator: &SparsePolynomialQ,
) -> SparsePolynomialQ {
    substitute_rational_and_clear_with_power(poly, variable, numerator, denominator).0
}

pub fn substitute_rational_and_clear_with_power(
    poly: &SparsePolynomialQ,
    variable: VariableId,
    numerator: &SparsePolynomialQ,
    denominator: &SparsePolynomialQ,
) -> (SparsePolynomialQ, u32) {
    let max_exp = poly
        .terms
        .iter()
        .map(|term| variable_exponent(term, variable))
        .max()
        .unwrap_or(0);
    let mut acc = crate::types::polynomial::zero_poly();
    for term in &poly.terms {
        let exp = variable_exponent(term, variable);
        let without_variable = term_without_variable(term, variable);
        let numerator_part = poly_pow(numerator, exp);
        let denominator_part = poly_pow(denominator, max_exp.saturating_sub(exp));
        let transformed_term = poly_mul(
            &poly_mul(&without_variable, &numerator_part),
            &denominator_part,
        );
        acc = poly_add(&acc, &transformed_term);
    }
    (normalize_poly(acc), max_exp)
}

fn variable_exponent(term: &TermQ, variable: VariableId) -> u32 {
    term.monomial
        .exponents
        .iter()
        .find(|(var, _)| *var == variable)
        .map_or(0, |(_, exp)| *exp)
}

fn term_without_variable(term: &TermQ, variable: VariableId) -> SparsePolynomialQ {
    normalize_poly(SparsePolynomialQ {
        terms: vec![TermQ {
            coeff: term.coeff.clone(),
            monomial: normalize_monomial(
                term.monomial
                    .exponents
                    .iter()
                    .filter_map(|(var, exp)| {
                        if *var == variable {
                            None
                        } else {
                            Some((*var, *exp))
                        }
                    })
                    .collect(),
            ),
        }],
        hash: hash_sequence("poly", &[]),
    })
}

fn poly_pow(base: &SparsePolynomialQ, exp: u32) -> SparsePolynomialQ {
    let mut result = constant_poly(int_q(1));
    for _ in 0..exp {
        result = poly_mul(&result, base);
    }
    result
}

fn hash_guard(
    factor: &SparsePolynomialQ,
    source_relation_ids: &[RelationId],
    guard_kind: GuardKind,
) -> Hash {
    let mut chunks = vec![vec![guard_kind as u8], factor.hash.0.to_vec()];
    for id in source_relation_ids {
        chunks.push(id.0.to_be_bytes().to_vec());
    }
    hash_sequence("compression-guard", &chunks)
}

fn hash_saturation(
    factor: &SparsePolynomialQ,
    witness_relation_id: RelationId,
    slack_variable: VariableId,
) -> Hash {
    hash_sequence(
        "compression-saturation",
        &[
            factor.hash.0.to_vec(),
            witness_relation_id.0.to_be_bytes().to_vec(),
            slack_variable.0.to_be_bytes().to_vec(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::semantic::{register_slack_encoding, RealConstraintKind};
    use crate::problem::validate::validate_input;
    use crate::solver::options::SolverOptions;
    use crate::types::ids::{RelationId, VariableId};
    use crate::types::polynomial::{constant_poly, poly_mul, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    use super::*;

    #[test]
    fn pre_kernel_compression_runs_steps_in_required_order() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let problem = make_problem(
            vec![t, x, y],
            t,
            vec![
                poly_sub(&variable_poly(y), &variable_poly(x)),
                poly_sub(&variable_poly(t), &variable_poly(y)),
            ],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let mut ctx = crate::problem::context::new_context(SolverOptions::default());
        let compressed = pre_kernel_compress(canonical, &mut ctx).unwrap();
        assert_eq!(
            compressed.compression_trace.ordered_steps,
            vec![
                CompressionStepKind::DefinitionalElimination,
                CompressionStepKind::LinearAffineElimination,
                CompressionStepKind::BinomialSimplification,
                CompressionStepKind::ExplicitSaturation,
                CompressionStepKind::TargetIndependentComponentMarking,
            ]
        );
        assert_eq!(compressed.substitutions.len(), 2);
        assert!(compressed.relations.is_empty());
    }

    #[test]
    fn explicit_saturation_and_independent_obligation_survive_pipeline() {
        let t = VariableId(0);
        let a = VariableId(1);
        let s = VariableId(2);
        let u = VariableId(3);
        let witness = poly_sub(
            &poly_mul(&variable_poly(a), &variable_poly(s)),
            &constant_poly(int_q(1)),
        );
        let problem = make_problem(
            vec![t, a, s, u],
            t,
            vec![
                witness,
                poly_sub(
                    &poly_mul(&variable_poly(u), &variable_poly(u)),
                    &constant_poly(int_q(2)),
                ),
            ],
            vec![register_slack_encoding(
                RealConstraintKind::NonZero,
                vec![RelationId(0)],
                vec![s],
            )],
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let mut ctx = crate::problem::context::new_context(SolverOptions::default());
        let compressed = pre_kernel_compress(canonical, &mut ctx).unwrap();
        assert!(!compressed.saturations.is_empty());
        assert!(!compressed.feasibility_obligations.is_empty());
    }

    #[test]
    fn compression_hash_binds_semantic_provenance() {
        let t = VariableId(0);
        let s = VariableId(1);
        let equation = variable_poly(t);
        let without_semantic = make_problem(vec![t, s], t, vec![equation.clone()], Vec::new());
        let with_semantic = make_problem(
            vec![t, s],
            t,
            vec![equation],
            vec![register_slack_encoding(
                RealConstraintKind::Positive,
                vec![RelationId(0)],
                vec![s],
            )],
        );

        let without_semantic =
            canonicalize_system(validate_input(without_semantic).unwrap()).unwrap();
        let with_semantic = canonicalize_system(validate_input(with_semantic).unwrap()).unwrap();
        let mut ctx = crate::problem::context::new_context(SolverOptions::default());
        let compressed_without = pre_kernel_compress(without_semantic, &mut ctx).unwrap();
        let mut ctx = crate::problem::context::new_context(SolverOptions::default());
        let compressed_with = pre_kernel_compress(with_semantic, &mut ctx).unwrap();

        assert_ne!(
            compressed_without.compressed_hash,
            compressed_with.compressed_hash
        );
        assert_eq!(compressed_with.semantic_encodings.len(), 1);
    }
}
