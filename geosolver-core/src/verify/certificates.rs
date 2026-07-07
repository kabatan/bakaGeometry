use serde::{Deserialize, Serialize};

use crate::algebra::elimination::LocalEliminationResult;
use crate::algebra::interpolation::InterpolationCertificate;
use crate::algebra::krylov::{AnnihilatorCertificate, CoverageCertificate};
use crate::algebra::norm_trace::TowerPlanDescription;
use crate::algebra::normal_form::MembershipCertificate;
use crate::algebra::quotient::ProductionQuotientHandleInput;
use crate::algebra::regular_chain::{ProjectionGenerators, RegularChainDAG};
use crate::algebra::resultant::SparseResultantCertificate;
use crate::planner::algebraic_cost::SaturatingCount;
use crate::planner::cost_model::RouteCostClass;
use crate::planner::kernel_plan::{
    hash_kernel_execution_plan, AffineEliminationStep, CertificateRoute, KernelExecutionPlan,
    UniversalStrategy,
};
use crate::types::hash::Hash;
use crate::types::ids::RelationId;
use crate::types::ids::VariableId;
use crate::types::polynomial::SparsePolynomialQ;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelCertificate {
    pub certificate_hash: Hash,
    pub certificate_route: CertificateRoute,
    pub plan_hash: Hash,
    pub source_relation_hashes: Vec<Hash>,
    pub output_relation_hashes: Vec<Hash>,
    pub exported_variables: Vec<VariableId>,
    pub binding_hash: Hash,
    pub payload: KernelCertificatePayload,
}

impl KernelCertificate {
    pub fn from_execution_plan(
        plan: &KernelExecutionPlan,
        output_relations: &[SparsePolynomialQ],
        certificate_hash: Hash,
    ) -> Self {
        Self::from_execution_plan_with_payload(
            plan,
            output_relations,
            certificate_hash,
            KernelCertificatePayload::BindingOnly,
        )
    }

    pub fn from_execution_plan_with_payload(
        plan: &KernelExecutionPlan,
        output_relations: &[SparsePolynomialQ],
        certificate_hash: Hash,
        payload: KernelCertificatePayload,
    ) -> Self {
        let mut cert = Self {
            certificate_hash,
            certificate_route: plan.certificate_route,
            plan_hash: hash_kernel_execution_plan(plan),
            source_relation_hashes: plan.source_relation_hashes.clone(),
            output_relation_hashes: output_relations
                .iter()
                .map(|relation| relation.hash)
                .collect(),
            exported_variables: plan.exported_variables.clone(),
            binding_hash: crate::types::hash::hash_sequence("kernel-certificate-binding", &[]),
            payload,
        };
        cert.binding_hash = kernel_certificate_binding_hash(&cert);
        cert
    }

    #[cfg(test)]
    pub fn synthetic_for_tests(certificate_hash: Hash) -> Self {
        let mut cert = Self {
            certificate_hash,
            certificate_route: CertificateRoute::SourceMembershipCertificate,
            plan_hash: crate::types::hash::hash_sequence("synthetic-kernel-plan", &[]),
            source_relation_hashes: Vec::new(),
            output_relation_hashes: Vec::new(),
            exported_variables: Vec::new(),
            binding_hash: crate::types::hash::hash_sequence("kernel-certificate-binding", &[]),
            payload: KernelCertificatePayload::SyntheticForTests,
        };
        cert.binding_hash = kernel_certificate_binding_hash(&cert);
        cert
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelCertificatePayload {
    TargetOnlySupport(TargetOnlySupportCertificate),
    Membership(MembershipProjectionCertificate),
    GuardedAffine(GuardedAffineProjectionCertificate),
    SparseResultant(SparseResultantProjectionCertificate),
    TargetAction(TargetActionProjectionCertificate),
    RegularChain(RegularChainProjectionCertificate),
    NormTrace(NormTraceProjectionCertificate),
    SpecializationInterpolation(SpecializationInterpolationProjectionCertificate),
    Universal(UniversalProjectionCertificate),
    BindingOnly,
    SyntheticForTests,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetOnlySupportCertificate {
    pub target: VariableId,
    pub source_relations: Vec<SparsePolynomialQ>,
    pub support_relation: SparsePolynomialQ,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipProjectionCertificate {
    pub source_relations: Vec<SparsePolynomialQ>,
    pub output_memberships: Vec<MembershipCertificate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardedAffineProjectionCertificate {
    pub source_relation_ids: Vec<RelationId>,
    pub source_relations: Vec<SparsePolynomialQ>,
    pub steps: Vec<AffineEliminationStep>,
    pub output_relations: Vec<SparsePolynomialQ>,
    pub affine_order_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparseResultantProjectionCertificate {
    pub source_relations: Vec<SparsePolynomialQ>,
    pub output_relations: Vec<SparsePolynomialQ>,
    pub resultant_certificates: Vec<SparseResultantCertificate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetActionProjectionCertificate {
    pub target: VariableId,
    pub quotient_input: ProductionQuotientHandleInput,
    pub output_relation: SparsePolynomialQ,
    pub coverage: CoverageCertificate,
    pub annihilator: AnnihilatorCertificate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegularChainProjectionCertificate {
    pub source_relations: Vec<SparsePolynomialQ>,
    pub variables: Vec<VariableId>,
    pub exported_variables: Vec<VariableId>,
    pub guards: Vec<SparsePolynomialQ>,
    pub dag: RegularChainDAG,
    pub projections: Vec<ProjectionGenerators>,
    pub output_relations: Vec<SparsePolynomialQ>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormTraceProjectionCertificate {
    pub tower: TowerPlanDescription,
    pub output_relation: SparsePolynomialQ,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecializationInterpolationProjectionCertificate {
    pub source_relations: Vec<SparsePolynomialQ>,
    pub output_relation: SparsePolynomialQ,
    pub interpolation_certificate: InterpolationCertificate,
    pub elimination_result: LocalEliminationResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniversalProjectionCertificate {
    pub stage_hash: Hash,
    pub stage_certificate_hash: Hash,
    pub attempted_strategies: Vec<UniversalStrategy>,
    pub strategy_records: Vec<UniversalStrategyTraceRecord>,
    pub skipped_cost_prohibited_strategy_hashes: Vec<Hash>,
    pub chosen_strategy: UniversalStrategy,
    pub failed_strategy_hashes: Vec<Hash>,
    pub executed_failed_strategy_hashes: Vec<Hash>,
    pub output_relations: Vec<SparsePolynomialQ>,
    pub inner_payload: Option<Box<KernelCertificatePayload>>,
    pub output_memberships: Vec<MembershipCertificate>,
    pub source_relations: Vec<SparsePolynomialQ>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniversalStrategyTraceRecord {
    pub strategy: UniversalStrategy,
    pub stage_hash: Hash,
    pub enabled: bool,
    pub skip_reason: Option<String>,
    pub cost_class: RouteCostClass,
    pub algebraic_work_estimate_hash: Hash,
    pub route_budget_hash: Hash,
    pub predicted_work_units: SaturatingCount,
    pub route_budget_max_work_units: SaturatingCount,
    pub route_budget_max_elapsed_steps: usize,
}

pub fn kernel_certificate_binding_hash(cert: &KernelCertificate) -> Hash {
    let mut chunks = vec![
        cert.certificate_hash.0.to_vec(),
        format!("{:?}", cert.certificate_route).into_bytes(),
        cert.plan_hash.0.to_vec(),
    ];
    for hash in &cert.source_relation_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for hash in &cert.output_relation_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for variable in &cert.exported_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(format!("{:?}", cert.payload).into_bytes());
    crate::types::hash::hash_sequence("kernel-certificate-binding", &chunks)
}
