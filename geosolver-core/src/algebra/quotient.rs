use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::algebra::normal_form::{verify_membership_by_certificate, MembershipCertificate};
use crate::result::status::{FailureKind, SolverError, SolverErrorKind};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::matrix::VectorQ;
use crate::types::polynomial::{
    constant_poly, poly_add, poly_mul, poly_sub, poly_variables, substitute_poly, variable_poly,
    SparsePolynomialQ, SubstitutionMap,
};
use crate::types::rational::{add_q, int_q, is_zero_q, mul_q, rational_to_bytes, zero_q};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BasisHandleId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BasisScope {
    TargetRelevant { variables: Vec<VariableId> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugQuotientHandleInput {
    pub basis_id: BasisHandleId,
    pub basis_scope: BasisScope,
    pub basis_polynomials: Vec<SparsePolynomialQ>,
    pub variable_action_columns: BTreeMap<VariableId, Vec<VectorQ>>,
    pub no_coordinate_roots_exported: bool,
    pub no_full_coordinate_rur_exported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalFormBasisCertificate {
    pub basis_hash: Hash,
    pub source_relation_authorization_hash: Hash,
    pub basis_element_certificates: Vec<BasisElementNormalFormCertificate>,
    pub certificate_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasisElementNormalFormCertificate {
    pub basis_index: usize,
    pub basis_polynomial_hash: Hash,
    pub normal_form_vector: VectorQ,
    pub normal_form_certificate: NormalFormCertificate,
    pub source_relation_authorization_hash: Hash,
    pub certificate_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalFormCertificate {
    pub input_polynomial_hash: Hash,
    pub represented_polynomial_hash: Hash,
    pub membership_certificate: MembershipCertificate,
    pub certificate_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionColumnCertificate {
    pub variable: VariableId,
    pub basis_index: usize,
    pub input_polynomial_hash: Hash,
    pub normal_form_vector: VectorQ,
    pub normal_form_certificate: NormalFormCertificate,
    pub source_relation_authorization_hash: Hash,
    pub column_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionQuotientHandleInput {
    pub basis_id: BasisHandleId,
    pub basis_scope: BasisScope,
    pub authorized_relation_hash: Hash,
    pub authorized_relations: Vec<SparsePolynomialQ>,
    pub basis_polynomials: Vec<SparsePolynomialQ>,
    pub normal_form_basis_certificate: NormalFormBasisCertificate,
    pub action_columns: BTreeMap<VariableId, Vec<ActionColumnCertificate>>,
    pub no_coordinate_roots_exported: bool,
    pub no_full_coordinate_rur_exported: bool,
}

pub trait TargetQuotientHandle {
    fn basis_id(&self) -> BasisHandleId;
    fn basis_size(&self) -> usize;
    fn basis_scope(&self) -> BasisScope;
    fn normal_form(&self, p: &SparsePolynomialQ) -> Result<VectorQ, SolverError>;
    fn multiply_by_variable(&self, v: &VectorQ, var: VariableId) -> Result<VectorQ, SolverError>;
    fn no_coordinate_solution_export(&self) -> bool;
    fn basis_polynomial(&self, index: usize) -> Option<SparsePolynomialQ>;
    fn basis_hash(&self) -> Hash;
    fn quotient_handle_hash(&self) -> Hash;
    fn is_production_provenanced(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugExplicitTargetQuotientHandle {
    input: DebugQuotientHandleInput,
    basis_hash: Hash,
    handle_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionProvenancedTargetQuotientHandle {
    input: ProductionQuotientHandleInput,
    basis_hash: Hash,
    handle_hash: Hash,
    action_vectors: BTreeMap<VariableId, Vec<VectorQ>>,
}

pub fn build_debug_explicit_target_quotient_handle(
    input: DebugQuotientHandleInput,
) -> Result<DebugExplicitTargetQuotientHandle, SolverError> {
    validate_debug_input(&input)?;
    let basis_hash = hash_basis(&input.basis_polynomials);
    let handle_hash = hash_debug_handle(&input, basis_hash);
    Ok(DebugExplicitTargetQuotientHandle {
        input,
        basis_hash,
        handle_hash,
    })
}

pub fn build_production_target_relevant_quotient_handle(
    input: ProductionQuotientHandleInput,
) -> Result<ProductionProvenancedTargetQuotientHandle, SolverError> {
    validate_production_input(&input)?;
    let basis_hash = hash_basis(&input.basis_polynomials);
    let action_vectors = input
        .action_columns
        .iter()
        .map(|(var, certs)| {
            (
                *var,
                certs
                    .iter()
                    .map(|cert| cert.normal_form_vector.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let handle_hash = hash_production_handle(&input, basis_hash);
    Ok(ProductionProvenancedTargetQuotientHandle {
        input,
        basis_hash,
        handle_hash,
        action_vectors,
    })
}

impl ProductionProvenancedTargetQuotientHandle {
    pub fn authorized_relation_hash(&self) -> Hash {
        self.input.authorized_relation_hash
    }

    pub fn action_column_certificate(
        &self,
        variable: VariableId,
        basis_index: usize,
    ) -> Option<&ActionColumnCertificate> {
        self.input
            .action_columns
            .get(&variable)
            .and_then(|columns| columns.get(basis_index))
    }
}

impl TargetQuotientHandle for DebugExplicitTargetQuotientHandle {
    fn basis_id(&self) -> BasisHandleId {
        self.input.basis_id
    }

    fn basis_size(&self) -> usize {
        self.input.basis_polynomials.len()
    }

    fn basis_scope(&self) -> BasisScope {
        self.input.basis_scope.clone()
    }

    fn normal_form(&self, p: &SparsePolynomialQ) -> Result<VectorQ, SolverError> {
        normal_form_via_action_columns(
            p,
            &self.input.basis_polynomials,
            &self.input.variable_action_columns,
        )
    }

    fn multiply_by_variable(&self, v: &VectorQ, var: VariableId) -> Result<VectorQ, SolverError> {
        multiply_by_variable_columns(
            v,
            var,
            self.basis_size(),
            &self.input.variable_action_columns,
        )
    }

    fn no_coordinate_solution_export(&self) -> bool {
        self.input.no_coordinate_roots_exported && self.input.no_full_coordinate_rur_exported
    }

    fn basis_polynomial(&self, index: usize) -> Option<SparsePolynomialQ> {
        self.input.basis_polynomials.get(index).cloned()
    }

    fn basis_hash(&self) -> Hash {
        self.basis_hash
    }

    fn quotient_handle_hash(&self) -> Hash {
        self.handle_hash
    }

    fn is_production_provenanced(&self) -> bool {
        false
    }
}

impl TargetQuotientHandle for ProductionProvenancedTargetQuotientHandle {
    fn basis_id(&self) -> BasisHandleId {
        self.input.basis_id
    }

    fn basis_size(&self) -> usize {
        self.input.basis_polynomials.len()
    }

    fn basis_scope(&self) -> BasisScope {
        self.input.basis_scope.clone()
    }

    fn normal_form(&self, p: &SparsePolynomialQ) -> Result<VectorQ, SolverError> {
        normal_form_via_action_columns(p, &self.input.basis_polynomials, &self.action_vectors)
    }

    fn multiply_by_variable(&self, v: &VectorQ, var: VariableId) -> Result<VectorQ, SolverError> {
        multiply_by_variable_columns(v, var, self.basis_size(), &self.action_vectors)
    }

    fn no_coordinate_solution_export(&self) -> bool {
        self.input.no_coordinate_roots_exported && self.input.no_full_coordinate_rur_exported
    }

    fn basis_polynomial(&self, index: usize) -> Option<SparsePolynomialQ> {
        self.input.basis_polynomials.get(index).cloned()
    }

    fn basis_hash(&self) -> Hash {
        self.basis_hash
    }

    fn quotient_handle_hash(&self) -> Hash {
        self.handle_hash
    }

    fn is_production_provenanced(&self) -> bool {
        true
    }
}

pub fn unit_vector(width: usize, index: usize) -> VectorQ {
    let mut entries = vec![zero_q(); width];
    if index < width {
        entries[index] = int_q(1);
    }
    VectorQ { entries }
}

pub fn zero_vector(width: usize) -> VectorQ {
    VectorQ {
        entries: vec![zero_q(); width],
    }
}

pub fn vector_add(a: &VectorQ, b: &VectorQ) -> VectorQ {
    assert_eq!(a.entries.len(), b.entries.len(), "vector width mismatch");
    VectorQ {
        entries: a
            .entries
            .iter()
            .zip(&b.entries)
            .map(|(x, y)| add_q(x, y))
            .collect(),
    }
}

pub fn vector_scale(v: &VectorQ, c: &crate::types::rational::RationalQ) -> VectorQ {
    VectorQ {
        entries: v.entries.iter().map(|x| mul_q(x, c)).collect(),
    }
}

pub fn vector_hash(v: &VectorQ) -> Hash {
    hash_sequence(
        "vector-q",
        &v.entries.iter().map(rational_to_bytes).collect::<Vec<_>>(),
    )
}

pub fn hash_authorized_relations(relations: &[SparsePolynomialQ]) -> Hash {
    hash_sequence(
        "authorized-quotient-relations",
        &relations
            .iter()
            .map(|poly| poly.hash.0.to_vec())
            .collect::<Vec<_>>(),
    )
}

pub fn normal_form_basis_certificate(
    basis_polynomials: &[SparsePolynomialQ],
    source_relation_authorization_hash: Hash,
) -> NormalFormBasisCertificate {
    let basis_hash = hash_basis(basis_polynomials);
    let basis_element_certificates = basis_polynomials
        .iter()
        .enumerate()
        .map(|(basis_index, basis_poly)| {
            let normal_form_vector = unit_vector(basis_polynomials.len(), basis_index);
            let represented = vector_to_polynomial(&normal_form_vector, basis_polynomials)
                .expect("unit vector width matches basis size");
            let normal_form_certificate = normal_form_certificate(
                basis_poly.hash,
                represented.hash,
                MembershipCertificate {
                    combination_terms: Vec::new(),
                },
            );
            let certificate_hash = hash_basis_element_certificate(
                basis_index,
                basis_poly.hash,
                &normal_form_vector,
                &normal_form_certificate,
                source_relation_authorization_hash,
            );
            BasisElementNormalFormCertificate {
                basis_index,
                basis_polynomial_hash: basis_poly.hash,
                normal_form_vector,
                normal_form_certificate,
                source_relation_authorization_hash,
                certificate_hash,
            }
        })
        .collect::<Vec<_>>();
    let certificate_hash = hash_sequence(
        "normal-form-basis-certificate",
        &std::iter::once(basis_hash.0.to_vec())
            .chain(std::iter::once(
                source_relation_authorization_hash.0.to_vec(),
            ))
            .chain(
                basis_element_certificates
                    .iter()
                    .map(|cert| cert.certificate_hash.0.to_vec()),
            )
            .collect::<Vec<_>>(),
    );
    NormalFormBasisCertificate {
        basis_hash,
        source_relation_authorization_hash,
        basis_element_certificates,
        certificate_hash,
    }
}

pub fn make_action_column_certificate(
    variable: VariableId,
    basis_index: usize,
    basis_polynomials: &[SparsePolynomialQ],
    authorized_relations: &[SparsePolynomialQ],
    source_relation_authorization_hash: Hash,
    normal_form_vector: VectorQ,
    membership_certificate: MembershipCertificate,
) -> Result<ActionColumnCertificate, SolverError> {
    let input_polynomial = action_input_polynomial(variable, basis_index, basis_polynomials)?;
    let represented = vector_to_polynomial(&normal_form_vector, basis_polynomials)?;
    let difference = poly_sub(&input_polynomial, &represented);
    if !verify_membership_by_certificate(&difference, &membership_certificate, authorized_relations)
    {
        return Err(certificate_design_gap(
            "action column normal-form certificate failed exact relation authorization check",
        ));
    }
    let normal_form_certificate = normal_form_certificate(
        input_polynomial.hash,
        represented.hash,
        membership_certificate,
    );
    let column_hash = vector_hash(&normal_form_vector);
    Ok(ActionColumnCertificate {
        variable,
        basis_index,
        input_polynomial_hash: input_polynomial.hash,
        normal_form_vector,
        normal_form_certificate,
        source_relation_authorization_hash,
        column_hash,
    })
}

pub fn monomial_basis_polynomials(var: VariableId, size: usize) -> Vec<SparsePolynomialQ> {
    let mut basis = Vec::with_capacity(size);
    let mut current = constant_poly(int_q(1));
    for _ in 0..size {
        basis.push(current.clone());
        current = poly_mul(&current, &variable_poly(var));
    }
    basis
}

fn normal_form_certificate(
    input_polynomial_hash: Hash,
    represented_polynomial_hash: Hash,
    membership_certificate: MembershipCertificate,
) -> NormalFormCertificate {
    let mut cert = NormalFormCertificate {
        input_polynomial_hash,
        represented_polynomial_hash,
        membership_certificate,
        certificate_hash: hash_sequence("normal-form-certificate-initial", &[]),
    };
    cert.certificate_hash = hash_normal_form_certificate(&cert);
    cert
}

fn hash_normal_form_certificate(cert: &NormalFormCertificate) -> Hash {
    let mut chunks = vec![
        cert.input_polynomial_hash.0.to_vec(),
        cert.represented_polynomial_hash.0.to_vec(),
    ];
    for term in &cert.membership_certificate.combination_terms {
        chunks.push(term.relation_id.to_be_bytes().to_vec());
        chunks.push(term.multiplier.hash.0.to_vec());
    }
    hash_sequence("normal-form-certificate", &chunks)
}

fn validate_debug_input(input: &DebugQuotientHandleInput) -> Result<(), SolverError> {
    validate_common_shape(
        &input.basis_polynomials,
        &input.variable_action_columns,
        input.no_coordinate_roots_exported,
        input.no_full_coordinate_rur_exported,
    )
}

fn validate_production_input(input: &ProductionQuotientHandleInput) -> Result<(), SolverError> {
    if input.authorized_relation_hash != hash_authorized_relations(&input.authorized_relations) {
        return Err(certificate_design_gap(
            "production quotient handle authorization hash does not match source relations",
        ));
    }
    let basis_hash = hash_basis(&input.basis_polynomials);
    if input.normal_form_basis_certificate.basis_hash != basis_hash
        || input
            .normal_form_basis_certificate
            .source_relation_authorization_hash
            != input.authorized_relation_hash
        || input.normal_form_basis_certificate.certificate_hash
            != hash_normal_form_basis_certificate(&input.normal_form_basis_certificate)
    {
        return Err(certificate_design_gap(
            "normal-form basis certificate is not bound to the authorized relations",
        ));
    }
    verify_normal_form_basis_certificate(
        &input.normal_form_basis_certificate,
        &input.basis_polynomials,
        &input.authorized_relations,
        input.authorized_relation_hash,
    )?;
    if !input.no_coordinate_roots_exported || !input.no_full_coordinate_rur_exported {
        return Err(SolverError::invalid_input(
            None,
            "quotient handle must not expose coordinate roots or full coordinate RUR",
        ));
    }
    let n = input.basis_polynomials.len();
    if n == 0 {
        return Err(SolverError::invalid_input(
            None,
            "quotient basis must be nonempty",
        ));
    }
    for (var, columns) in &input.action_columns {
        if columns.len() != n {
            return Err(SolverError::invalid_input(
                Some(*var),
                "quotient action column count must match basis size",
            ));
        }
        for (basis_index, cert) in columns.iter().enumerate() {
            verify_action_column_certificate(
                cert,
                *var,
                basis_index,
                &input.basis_polynomials,
                &input.authorized_relations,
                input.authorized_relation_hash,
            )?;
        }
    }
    Ok(())
}

fn validate_common_shape(
    basis_polynomials: &[SparsePolynomialQ],
    columns_by_variable: &BTreeMap<VariableId, Vec<VectorQ>>,
    no_coordinate_roots_exported: bool,
    no_full_coordinate_rur_exported: bool,
) -> Result<(), SolverError> {
    let n = basis_polynomials.len();
    if n == 0 {
        return Err(SolverError::invalid_input(
            None,
            "quotient basis must be nonempty",
        ));
    }
    if !no_coordinate_roots_exported || !no_full_coordinate_rur_exported {
        return Err(SolverError::invalid_input(
            None,
            "quotient handle must not expose coordinate roots or full coordinate RUR",
        ));
    }
    for (var, columns) in columns_by_variable {
        if columns.len() != n {
            return Err(SolverError::invalid_input(
                Some(*var),
                "quotient action column count must match basis size",
            ));
        }
        for column in columns {
            if column.entries.len() != n {
                return Err(SolverError::invalid_input(
                    Some(*var),
                    "quotient action column width must match basis size",
                ));
            }
        }
    }
    Ok(())
}

fn verify_action_column_certificate(
    cert: &ActionColumnCertificate,
    variable: VariableId,
    basis_index: usize,
    basis_polynomials: &[SparsePolynomialQ],
    authorized_relations: &[SparsePolynomialQ],
    source_relation_authorization_hash: Hash,
) -> Result<(), SolverError> {
    if cert.variable != variable
        || cert.basis_index != basis_index
        || cert.source_relation_authorization_hash != source_relation_authorization_hash
        || cert.column_hash != vector_hash(&cert.normal_form_vector)
    {
        return Err(certificate_design_gap(
            "action column certificate metadata is not bound to the requested column",
        ));
    }
    let input_polynomial = action_input_polynomial(variable, basis_index, basis_polynomials)?;
    if cert.input_polynomial_hash != input_polynomial.hash {
        return Err(certificate_design_gap(
            "action column certificate input hash mismatch",
        ));
    }
    let represented = vector_to_polynomial(&cert.normal_form_vector, basis_polynomials)?;
    let difference = poly_sub(&input_polynomial, &represented);
    if cert.normal_form_certificate.certificate_hash
        != hash_normal_form_certificate(&cert.normal_form_certificate)
    {
        return Err(certificate_design_gap(
            "action column normal-form certificate hash mismatch",
        ));
    }
    if cert.normal_form_certificate.input_polynomial_hash != input_polynomial.hash
        || cert.normal_form_certificate.represented_polynomial_hash != represented.hash
        || !verify_membership_by_certificate(
            &difference,
            &cert.normal_form_certificate.membership_certificate,
            authorized_relations,
        )
    {
        return Err(certificate_design_gap(
            "action column normal-form certificate failed exact verification",
        ));
    }
    Ok(())
}

fn verify_normal_form_basis_certificate(
    cert: &NormalFormBasisCertificate,
    basis_polynomials: &[SparsePolynomialQ],
    authorized_relations: &[SparsePolynomialQ],
    source_relation_authorization_hash: Hash,
) -> Result<(), SolverError> {
    if cert.basis_element_certificates.len() != basis_polynomials.len() {
        return Err(certificate_design_gap(
            "normal-form basis certificate does not cover every basis element",
        ));
    }
    for (basis_index, basis_poly) in basis_polynomials.iter().enumerate() {
        let element = &cert.basis_element_certificates[basis_index];
        if element.basis_index != basis_index
            || element.basis_polynomial_hash != basis_poly.hash
            || element.source_relation_authorization_hash != source_relation_authorization_hash
            || element.certificate_hash
                != hash_basis_element_certificate(
                    element.basis_index,
                    element.basis_polynomial_hash,
                    &element.normal_form_vector,
                    &element.normal_form_certificate,
                    element.source_relation_authorization_hash,
                )
            || element.normal_form_certificate.certificate_hash
                != hash_normal_form_certificate(&element.normal_form_certificate)
        {
            return Err(certificate_design_gap(
                "normal-form basis element certificate metadata mismatch",
            ));
        }
        if element.normal_form_vector != unit_vector(basis_polynomials.len(), basis_index) {
            return Err(certificate_design_gap(
                "normal-form basis element certificate is not the expected unit vector",
            ));
        }
        let represented = vector_to_polynomial(&element.normal_form_vector, basis_polynomials)?;
        let difference = poly_sub(basis_poly, &represented);
        if element.normal_form_certificate.input_polynomial_hash != basis_poly.hash
            || element.normal_form_certificate.represented_polynomial_hash != represented.hash
            || !verify_membership_by_certificate(
                &difference,
                &element.normal_form_certificate.membership_certificate,
                authorized_relations,
            )
        {
            return Err(certificate_design_gap(
                "normal-form basis element certificate failed exact verification",
            ));
        }
    }
    Ok(())
}

fn action_input_polynomial(
    variable: VariableId,
    basis_index: usize,
    basis_polynomials: &[SparsePolynomialQ],
) -> Result<SparsePolynomialQ, SolverError> {
    let basis = basis_polynomials
        .get(basis_index)
        .ok_or_else(|| certificate_design_gap("missing quotient basis polynomial"))?;
    Ok(poly_mul(&variable_poly(variable), basis))
}

fn vector_to_polynomial(
    vector: &VectorQ,
    basis_polynomials: &[SparsePolynomialQ],
) -> Result<SparsePolynomialQ, SolverError> {
    if vector.entries.len() != basis_polynomials.len() {
        return Err(SolverError::invalid_input(
            None,
            "normal-form vector width must match basis size",
        ));
    }
    let mut acc = constant_poly(zero_q());
    for (coeff, basis) in vector.entries.iter().zip(basis_polynomials) {
        if is_zero_q(coeff) {
            continue;
        }
        acc = poly_add(&acc, &crate::types::polynomial::poly_scale(basis, coeff));
    }
    Ok(acc)
}

fn normal_form_via_action_columns(
    p: &SparsePolynomialQ,
    basis_polynomials: &[SparsePolynomialQ],
    action_columns: &BTreeMap<VariableId, Vec<VectorQ>>,
) -> Result<VectorQ, SolverError> {
    let subst = basis_variables(basis_polynomials)
        .into_iter()
        .map(|var| (var, variable_poly(var)))
        .collect::<SubstitutionMap>();
    let normalized = substitute_poly(p, &subst);
    let mut acc = zero_vector(basis_polynomials.len());
    for term in &normalized.terms {
        let mut v = unit_vector(basis_polynomials.len(), 0);
        for (var, exp) in &term.monomial.exponents {
            for _ in 0..*exp {
                v = multiply_by_variable_columns(
                    &v,
                    *var,
                    basis_polynomials.len(),
                    action_columns,
                )?;
            }
        }
        acc = vector_add(&acc, &vector_scale(&v, &term.coeff));
    }
    Ok(acc)
}

fn multiply_by_variable_columns(
    v: &VectorQ,
    var: VariableId,
    basis_size: usize,
    action_columns: &BTreeMap<VariableId, Vec<VectorQ>>,
) -> Result<VectorQ, SolverError> {
    if v.entries.len() != basis_size {
        return Err(SolverError::invalid_input(
            Some(var),
            "quotient vector dimension does not match basis size",
        ));
    }
    let columns = action_columns
        .get(&var)
        .ok_or_else(|| SolverError::invalid_input(Some(var), "missing quotient variable action"))?;
    let mut acc = zero_vector(basis_size);
    for (coeff, column) in v.entries.iter().zip(columns) {
        if is_zero_q(coeff) {
            continue;
        }
        acc = vector_add(&acc, &vector_scale(column, coeff));
    }
    Ok(acc)
}

fn basis_variables(basis_polynomials: &[SparsePolynomialQ]) -> Vec<VariableId> {
    let mut vars = poly_variables(
        &basis_polynomials
            .iter()
            .fold(constant_poly(zero_q()), |acc, p| poly_add(&acc, p)),
    )
    .into_iter()
    .collect::<Vec<_>>();
    vars.sort();
    vars
}

fn hash_basis(basis: &[SparsePolynomialQ]) -> Hash {
    hash_sequence(
        "quotient-basis",
        &basis
            .iter()
            .map(|poly| poly.hash.0.to_vec())
            .collect::<Vec<_>>(),
    )
}

fn hash_basis_element_certificate(
    basis_index: usize,
    basis_polynomial_hash: Hash,
    normal_form_vector: &VectorQ,
    normal_form_certificate: &NormalFormCertificate,
    source_relation_authorization_hash: Hash,
) -> Hash {
    hash_sequence(
        "basis-element-normal-form-certificate",
        &[
            basis_index.to_be_bytes().to_vec(),
            basis_polynomial_hash.0.to_vec(),
            vector_hash(normal_form_vector).0.to_vec(),
            normal_form_certificate.certificate_hash.0.to_vec(),
            source_relation_authorization_hash.0.to_vec(),
        ],
    )
}

fn hash_normal_form_basis_certificate(cert: &NormalFormBasisCertificate) -> Hash {
    hash_sequence(
        "normal-form-basis-certificate",
        &std::iter::once(cert.basis_hash.0.to_vec())
            .chain(std::iter::once(
                cert.source_relation_authorization_hash.0.to_vec(),
            ))
            .chain(
                cert.basis_element_certificates
                    .iter()
                    .map(|element| element.certificate_hash.0.to_vec()),
            )
            .collect::<Vec<_>>(),
    )
}

fn hash_debug_handle(input: &DebugQuotientHandleInput, basis_hash: Hash) -> Hash {
    let mut chunks = vec![
        input.basis_id.0.to_be_bytes().to_vec(),
        basis_hash.0.to_vec(),
        vec![0],
    ];
    for (var, columns) in &input.variable_action_columns {
        chunks.push(var.0.to_be_bytes().to_vec());
        chunks.extend(columns.iter().map(|column| vector_hash(column).0.to_vec()));
    }
    hash_sequence("debug-quotient-handle", &chunks)
}

fn hash_production_handle(input: &ProductionQuotientHandleInput, basis_hash: Hash) -> Hash {
    let mut chunks = vec![
        input.basis_id.0.to_be_bytes().to_vec(),
        basis_hash.0.to_vec(),
        input.authorized_relation_hash.0.to_vec(),
        input
            .normal_form_basis_certificate
            .certificate_hash
            .0
            .to_vec(),
    ];
    for (var, columns) in &input.action_columns {
        chunks.push(var.0.to_be_bytes().to_vec());
        chunks.extend(
            columns
                .iter()
                .map(|column| column.normal_form_certificate.certificate_hash.0.to_vec()),
        );
    }
    hash_sequence("production-quotient-handle", &chunks)
}

fn certificate_design_gap(message: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::CertificateDesignGap {
            constructed_object_hash: hash_sequence(
                "certificate-gap",
                &[message.as_bytes().to_vec()],
            ),
            missing_certificate_kind: message.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::algebra::normal_form::{MembershipCertificate, MembershipTerm};
    use crate::result::status::SolverStatus;
    use crate::types::polynomial::{constant_poly, poly_mul, poly_sub, variable_poly};

    use super::*;

    fn production_input(target: VariableId) -> ProductionQuotientHandleInput {
        let basis = monomial_basis_polynomials(target, 2);
        let relation = poly_add(
            &poly_sub(
                &poly_mul(&variable_poly(target), &variable_poly(target)),
                &poly_mul(&constant_poly(int_q(3)), &variable_poly(target)),
            ),
            &constant_poly(int_q(2)),
        );
        let relations = vec![relation];
        let auth_hash = hash_authorized_relations(&relations);
        let col0 = make_action_column_certificate(
            target,
            0,
            &basis,
            &relations,
            auth_hash,
            VectorQ {
                entries: vec![int_q(0), int_q(1)],
            },
            MembershipCertificate {
                combination_terms: Vec::new(),
            },
        )
        .unwrap();
        let col1 = make_action_column_certificate(
            target,
            1,
            &basis,
            &relations,
            auth_hash,
            VectorQ {
                entries: vec![int_q(-2), int_q(3)],
            },
            MembershipCertificate {
                combination_terms: vec![MembershipTerm {
                    relation_id: 0,
                    multiplier: constant_poly(int_q(1)),
                }],
            },
        )
        .unwrap();
        ProductionQuotientHandleInput {
            basis_id: BasisHandleId(1),
            basis_scope: BasisScope::TargetRelevant {
                variables: vec![target],
            },
            authorized_relation_hash: auth_hash,
            authorized_relations: relations,
            basis_polynomials: basis.clone(),
            normal_form_basis_certificate: normal_form_basis_certificate(&basis, auth_hash),
            action_columns: BTreeMap::from([(target, vec![col0, col1])]),
            no_coordinate_roots_exported: true,
            no_full_coordinate_rur_exported: true,
        }
    }

    #[test]
    fn debug_explicit_handle_is_not_production_provenanced() {
        let t = VariableId(7);
        let input = DebugQuotientHandleInput {
            basis_id: BasisHandleId(1),
            basis_scope: BasisScope::TargetRelevant { variables: vec![t] },
            basis_polynomials: monomial_basis_polynomials(t, 2),
            variable_action_columns: BTreeMap::from([(
                t,
                vec![
                    VectorQ {
                        entries: vec![int_q(0), int_q(1)],
                    },
                    VectorQ {
                        entries: vec![int_q(-2), int_q(3)],
                    },
                ],
            )]),
            no_coordinate_roots_exported: true,
            no_full_coordinate_rur_exported: true,
        };
        let handle = build_debug_explicit_target_quotient_handle(input).unwrap();
        let t_squared = poly_mul(&variable_poly(t), &variable_poly(t));

        assert!(!handle.is_production_provenanced());
        assert_eq!(
            handle.normal_form(&t_squared).unwrap(),
            VectorQ {
                entries: vec![int_q(-2), int_q(3)]
            }
        );
    }

    #[test]
    fn production_handle_verifies_action_columns_from_authorized_relations() {
        let target = VariableId(5);
        let handle =
            build_production_target_relevant_quotient_handle(production_input(target)).unwrap();

        assert!(handle.is_production_provenanced());
        assert!(handle.no_coordinate_solution_export());
        assert!(handle.action_column_certificate(target, 1).is_some());
    }

    #[test]
    fn malicious_injected_action_column_is_rejected_by_production_builder() {
        let target = VariableId(5);
        let mut input = production_input(target);
        let tampered = &mut input.action_columns.get_mut(&target).unwrap()[1];
        tampered.normal_form_vector = VectorQ {
            entries: vec![int_q(99), int_q(3)],
        };
        tampered.column_hash = vector_hash(&tampered.normal_form_vector);
        let err = build_production_target_relevant_quotient_handle(input).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::CertificateDesignGap);
    }

    #[test]
    fn tampered_authorization_hash_is_rejected() {
        let target = VariableId(5);
        let mut input = production_input(target);
        input.authorized_relation_hash = hash_sequence("tampered", &[b"x".to_vec()]);
        let err = build_production_target_relevant_quotient_handle(input).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::CertificateDesignGap);
    }

    #[test]
    fn tampered_action_certificate_hash_is_rejected() {
        let target = VariableId(5);
        let mut input = production_input(target);
        input.action_columns.get_mut(&target).unwrap()[1]
            .normal_form_certificate
            .certificate_hash = hash_sequence("tampered-action-cert", &[b"x".to_vec()]);
        let err = build_production_target_relevant_quotient_handle(input).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::CertificateDesignGap);
    }

    #[test]
    fn tampered_basis_certificate_hash_is_rejected() {
        let target = VariableId(5);
        let mut input = production_input(target);
        input
            .normal_form_basis_certificate
            .basis_element_certificates[0]
            .certificate_hash = hash_sequence("tampered-basis-cert", &[b"x".to_vec()]);
        input.normal_form_basis_certificate.certificate_hash =
            hash_normal_form_basis_certificate(&input.normal_form_basis_certificate);
        let err = build_production_target_relevant_quotient_handle(input).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::CertificateDesignGap);
    }
}
