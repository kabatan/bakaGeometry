use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::algebra::groebner::{
    groebner_elimination_basis, polynomial_in_keep_variables, GroebnerOptions,
};
use crate::algebra::monomial_order::elimination_order;
use crate::compose::message::{hash_projection_message, ProjectionMessage};
use crate::compose::separator_elimination::eliminate_separators_from_message_relations;
use crate::graph::projection_dag::TargetProjectionDAG;
use crate::problem::context::SolverContext;
use crate::result::cost_trace::CompositionCostTrace;
use crate::result::status::{AlgebraicReason, FailureKind, SolverError, SolverErrorKind, StageId};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{BlockId, VariableId};
use crate::types::polynomial::{
    poly_monomial_count, poly_total_degree, poly_variables, SparsePolynomialQ,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposedProjection {
    pub target: VariableId,
    pub root_block_id: BlockId,
    pub message_relations: Vec<SparsePolynomialQ>,
    pub root_relations: Vec<SparsePolynomialQ>,
    pub source_message_hashes: Vec<Hash>,
    pub separator_elimination_hashes: Vec<Hash>,
    pub separator_elimination_messages: Vec<ProjectionMessage>,
    pub composition_cost: CompositionCostTrace,
    pub composed_hash: Hash,
}

pub fn compose_projection_messages(
    dag: &TargetProjectionDAG,
    messages: Vec<ProjectionMessage>,
    target: VariableId,
    ctx: &mut SolverContext,
) -> Result<ComposedProjection, SolverError> {
    let blocks = dag
        .blocks
        .iter()
        .map(|block| (block.block_id, block))
        .collect::<BTreeMap<_, _>>();
    if !blocks.contains_key(&dag.root_block_id) {
        return Err(implementation_bug("composition DAG root block is missing"));
    }
    if messages.is_empty() && dag.blocks.iter().all(|block| block.relation_ids.is_empty()) {
        let mut composed = ComposedProjection {
            target,
            root_block_id: dag.root_block_id,
            message_relations: Vec::new(),
            root_relations: Vec::new(),
            source_message_hashes: Vec::new(),
            separator_elimination_hashes: Vec::new(),
            separator_elimination_messages: Vec::new(),
            composition_cost: CompositionCostTrace::default(),
            composed_hash: hash_sequence("composed-projection", &[]),
        };
        composed.composed_hash = hash_composed_projection(&composed);
        return Ok(composed);
    }

    let mut relations = Vec::new();
    let mut message_hashes = Vec::new();
    let mut seen_blocks = BTreeSet::new();
    for message in messages {
        let Some(_block) = blocks.get(&message.block_id) else {
            return Err(implementation_bug(
                "projection message references a block outside the DAG",
            ));
        };
        if !seen_blocks.insert(message.block_id) {
            return Err(implementation_bug(
                "composition received duplicate messages for a projection block",
            ));
        }
        validate_message_binding(&message)?;
        message_hashes.push(message.package_hash);
        relations.extend(
            message
                .relation_generators
                .into_iter()
                .filter(|relation| !relation.terms.is_empty()),
        );
    }
    let relation_count_before = relations.len();
    let mut root_relations = target_only_relations(&relations, target);
    let mut separator_hashes = Vec::new();
    let mut separator_messages = Vec::new();
    if root_relations.is_empty() {
        let all_variables = relations
            .iter()
            .flat_map(poly_variables)
            .collect::<BTreeSet<_>>();
        let keep = BTreeSet::from([target]);
        let separators = all_variables
            .difference(&keep)
            .copied()
            .collect::<BTreeSet<_>>();
        let separator_result = eliminate_separators_from_message_relations(
            relations.clone(),
            keep,
            separators,
            target,
            ctx,
        );
        match separator_result {
            Ok(message) => {
                separator_hashes.push(message.package_hash);
                root_relations.extend(target_only_relations(&message.relation_generators, target));
                separator_messages.push(message);
            }
            Err(err)
                if matches!(
                    err.kind,
                    SolverErrorKind::Failure(FailureKind::ImplementationBug { .. })
                ) =>
            {
                return Err(err);
            }
            Err(_err) => {
                if !message_relations_have_target_eliminant(&relations, target) {
                    return Err(_err);
                }
                // Final support construction has a separate composed-ideal membership route.
                // Continue only when the composed message ideal already has a target eliminant.
            }
        }
    }
    if root_relations.is_empty() && !message_relations_have_target_eliminant(&relations, target) {
        return Err(algorithmic_hard_case(
            target,
            dag.dag_hash,
            "composition produced no target-only root relation or composed-ideal target eliminant",
        ));
    }
    let mut composed = ComposedProjection {
        target,
        root_block_id: dag.root_block_id,
        message_relations: relations,
        root_relations,
        source_message_hashes: message_hashes,
        separator_elimination_hashes: separator_hashes,
        separator_elimination_messages: separator_messages,
        composition_cost: CompositionCostTrace {
            relation_count_before,
            relation_count_after: 0,
        },
        composed_hash: hash_sequence("composed-projection", &[]),
    };
    composed.composition_cost.relation_count_after = composed.root_relations.len();
    composed.composed_hash = hash_composed_projection(&composed);
    Ok(composed)
}

pub fn hash_composed_projection(composed: &ComposedProjection) -> Hash {
    let mut chunks = vec![
        composed.target.0.to_be_bytes().to_vec(),
        composed.root_block_id.0.to_be_bytes().to_vec(),
    ];
    for relation in &composed.message_relations {
        chunks.push(relation.hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for relation in &composed.root_relations {
        chunks.push(relation.hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for hash in &composed.source_message_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for hash in &composed.separator_elimination_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for message in &composed.separator_elimination_messages {
        chunks.push(message.package_hash.0.to_vec());
    }
    hash_sequence("composed-projection", &chunks)
}

impl ComposedProjection {
    #[cfg(test)]
    pub fn from_message_relations_for_test(
        target: VariableId,
        relations: Vec<SparsePolynomialQ>,
        source_message_hashes: Vec<Hash>,
    ) -> Self {
        let mut composed = Self {
            target,
            root_block_id: BlockId(0),
            root_relations: relations.clone(),
            message_relations: relations,
            source_message_hashes,
            separator_elimination_hashes: Vec::new(),
            separator_elimination_messages: Vec::new(),
            composition_cost: CompositionCostTrace::default(),
            composed_hash: hash_sequence("composed-projection", &[]),
        };
        composed.composition_cost.relation_count_before = composed.message_relations.len();
        composed.composition_cost.relation_count_after = composed.root_relations.len();
        composed.composed_hash = hash_composed_projection(&composed);
        composed
    }
}

fn validate_message_binding(message: &ProjectionMessage) -> Result<(), SolverError> {
    if hash_projection_message(message) != message.package_hash {
        return Err(implementation_bug(
            "projection message package hash mismatch",
        ));
    }
    let exported = message
        .exported_variables
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    for relation in &message.relation_generators {
        if !poly_variables(relation).is_subset(&exported) {
            return Err(implementation_bug(
                "projection message relation is outside exported variables",
            ));
        }
    }
    Ok(())
}

fn target_only_relations(
    relations: &[SparsePolynomialQ],
    target: VariableId,
) -> Vec<SparsePolynomialQ> {
    let target_set = BTreeSet::from([target]);
    relations
        .iter()
        .filter(|relation| {
            !relation.terms.is_empty() && poly_variables(relation).is_subset(&target_set)
        })
        .cloned()
        .collect()
}

fn message_relations_have_target_eliminant(
    relations: &[SparsePolynomialQ],
    target: VariableId,
) -> bool {
    if relations.is_empty() {
        return false;
    }
    let all_variables = relations
        .iter()
        .flat_map(poly_variables)
        .collect::<BTreeSet<_>>();
    if !all_variables.contains(&target) {
        return false;
    }
    let eliminate = all_variables
        .iter()
        .copied()
        .filter(|var| *var != target)
        .collect::<Vec<_>>();
    if relations.len() <= eliminate.len()
        || relations.iter().map(poly_monomial_count).sum::<usize>() > 64
        || relations
            .iter()
            .map(|relation| poly_total_degree(relation) as usize)
            .max()
            .unwrap_or(0)
            > 8
        || eliminate.len() > 4
    {
        return false;
    }
    let keep = BTreeSet::from([target]);
    let order = elimination_order(&eliminate, &[target]);
    groebner_elimination_basis(relations, &order, GroebnerOptions::default())
        .map(|basis| {
            basis.basis.iter().any(|entry| {
                !entry.polynomial.terms.is_empty()
                    && polynomial_in_keep_variables(&entry.polynomial, &keep)
                    && poly_variables(&entry.polynomial).contains(&target)
            })
        })
        .unwrap_or(false)
}

fn algorithmic_hard_case(
    target: VariableId,
    minimal_block_hash: Hash,
    reason: &str,
) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId("P10Composition".to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash,
        }),
    }
}

fn implementation_bug(message: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::ImplementationBug {
            invariant_violated: message.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::final_support::{
        build_final_support_or_nonfinite, build_global_support_polynomial,
        finalize_nonfinite_result, verify_nonfinite_certificate, FinalSupportComputation,
    };
    use crate::compose::message::{hash_projection_message, ProjectionMessage, ProjectionStrength};
    use crate::graph::projection_dag::{ProjectionBlock, TargetProjectionDAG};
    use crate::kernels::traits::KernelKind;
    use crate::preprocess::compression::CompressionTrace;
    use crate::problem::context::new_context;
    use crate::result::cost_trace::{GlobalCostTrace, ProjectionCostTrace};
    use crate::result::status::SolverStatus;
    use crate::solver::options::SolverOptions;
    use crate::types::hash::{hash_sequence, Hash};
    use crate::types::ids::{BlockId, PackageId, RelationId, VariableId};
    use crate::types::polynomial::{poly_sub, variable_poly, SparsePolynomialQ};
    use crate::types::rational::int_q;
    use crate::verify::certificates::KernelCertificate;
    use crate::verify::certificates::{kernel_certificate_binding_hash, KernelCertificatePayload};
    use crate::verify::verify_support::verify_global_support;
    use std::collections::BTreeSet;

    #[test]
    fn p10_multiblock_composition_eliminates_separator_from_message_relations_only() {
        let t = VariableId(0);
        let x = VariableId(1);
        let child = message(
            BlockId(1),
            PackageId(11),
            vec![x],
            poly_sub(&variable_poly(x), &constant(1)),
        );
        let root = message(
            BlockId(0),
            PackageId(10),
            vec![t, x],
            poly_sub(&variable_poly(t), &variable_poly(x)),
        );
        let dag = dag(t, x);
        let mut ctx = new_context(SolverOptions::default());

        let composed = compose_projection_messages(&dag, vec![root, child], t, &mut ctx).unwrap();
        assert_eq!(composed.separator_elimination_messages.len(), 1);
        assert!(composed.root_relations.iter().all(|relation| {
            crate::types::polynomial::poly_variables(relation).is_subset(&BTreeSet::from([t]))
        }));

        let support = build_global_support_polynomial(composed.clone(), t, &mut ctx).unwrap();
        verify_global_support(&support, &composed).unwrap();
        assert!(same_up_to_sign(
            &support.coeffs_low_to_high,
            &[int_q(-1), int_q(1)]
        ));
    }

    #[test]
    fn fcr_p9_support_verifier_replays_separator_elimination_certificate() {
        let t = VariableId(0);
        let x = VariableId(1);
        let child = message(
            BlockId(1),
            PackageId(11),
            vec![x],
            poly_sub(&variable_poly(x), &constant(1)),
        );
        let root = message(
            BlockId(0),
            PackageId(10),
            vec![t, x],
            poly_sub(&variable_poly(t), &variable_poly(x)),
        );
        let dag = dag(t, x);
        let mut ctx = new_context(SolverOptions::default());

        let composed = compose_projection_messages(&dag, vec![root, child], t, &mut ctx).unwrap();
        assert_eq!(composed.separator_elimination_messages.len(), 1);
        let support = build_global_support_polynomial(composed.clone(), t, &mut ctx).unwrap();
        verify_global_support(&support, &composed).unwrap();

        let mut tampered = composed.clone();
        let tampered_package_hash = {
            let separator_message = tampered
                .separator_elimination_messages
                .first_mut()
                .expect("separator elimination evidence");
            let KernelCertificatePayload::Membership(proof) =
                &mut separator_message.certificate.payload
            else {
                panic!("separator elimination must carry a membership proof");
            };
            let first_term = proof
                .output_memberships
                .first_mut()
                .and_then(|membership| membership.combination_terms.first_mut())
                .expect("separator membership term");
            first_term.multiplier = constant(0);
            separator_message.certificate.binding_hash =
                kernel_certificate_binding_hash(&separator_message.certificate);
            separator_message.package_hash = hash_projection_message(separator_message);
            separator_message.package_hash
        };
        tampered.separator_elimination_hashes[0] = tampered_package_hash;
        tampered.composed_hash = hash_composed_projection(&tampered);

        assert!(verify_global_support(&support, &tampered).is_err());
    }

    #[test]
    fn p12g_g6_multiseparator_composition_requires_child_message() {
        let t = VariableId(0);
        let x = VariableId(1);
        let child = message(
            BlockId(1),
            PackageId(11),
            vec![x],
            poly_sub(&variable_poly(x), &constant(1)),
        );
        let root = message(
            BlockId(0),
            PackageId(10),
            vec![t, x],
            poly_sub(&variable_poly(t), &variable_poly(x)),
        );
        let dag = dag(t, x);
        let mut ctx = new_context(SolverOptions::default());

        let composed =
            compose_projection_messages(&dag, vec![root.clone(), child], t, &mut ctx).unwrap();
        let support = build_global_support_polynomial(composed, t, &mut ctx).unwrap();
        assert!(same_up_to_sign(
            &support.coeffs_low_to_high,
            &[int_q(-1), int_q(1)]
        ));
        assert!(compose_projection_messages(&dag, vec![root], t, &mut ctx).is_err());
    }

    #[test]
    fn p10_tampered_message_relation_is_rejected_before_composition() {
        let t = VariableId(0);
        let x = VariableId(1);
        let mut root = message(
            BlockId(0),
            PackageId(10),
            vec![t, x],
            poly_sub(&variable_poly(t), &variable_poly(x)),
        );
        root.relation_generators[0] = poly_sub(&variable_poly(t), &constant(2));
        let dag = dag(t, x);
        let mut ctx = new_context(SolverOptions::default());

        let err = compose_projection_messages(&dag, vec![root], t, &mut ctx).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn p10_relation_search_exhaustion_is_hardcase_not_nonfinite() {
        let t = VariableId(0);
        let x = VariableId(1);
        let relation = poly_sub(
            &crate::types::polynomial::poly_mul(&variable_poly(x), &variable_poly(x)),
            &constant(-1),
        );
        let mut ctx = new_context(SolverOptions {
            max_relation_search_export_degree: Some(1),
            ..SolverOptions::default()
        });

        let err =
            crate::compose::separator_elimination::eliminate_separators_from_message_relations(
                vec![relation],
                BTreeSet::from([t]),
                BTreeSet::from([x]),
                t,
                &mut ctx,
            )
            .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::AlgorithmicHardCase);
    }

    #[test]
    fn p10_certified_nonfinite_system_uses_positive_certificate_only() {
        let t = VariableId(0);
        let x = VariableId(1);
        let relation = poly_sub(&variable_poly(x), &constant(1));
        let composed = ComposedProjection::from_message_relations_for_test(
            t,
            vec![relation],
            vec![Hash([7; 32])],
        );
        let mut ctx = new_context(SolverOptions::default());

        let outcome = build_final_support_or_nonfinite(composed.clone(), t, &mut ctx).unwrap();
        let FinalSupportComputation::CertifiedNonFinite(cert) = outcome else {
            panic!("expected certified nonfinite outcome");
        };
        verify_nonfinite_certificate(&cert, &composed).unwrap();
        let result =
            finalize_nonfinite_result(t, cert, &composed, Vec::new(), GlobalCostTrace::default())
                .unwrap();
        assert_eq!(result.status, SolverStatus::CertifiedNonFiniteTargetImage);
    }

    fn message(
        block_id: BlockId,
        package_id: PackageId,
        exported_variables: Vec<VariableId>,
        relation: SparsePolynomialQ,
    ) -> ProjectionMessage {
        let mut message = ProjectionMessage {
            package_id,
            block_id,
            kernel_kind: KernelKind::TargetRelationSearch,
            source_relation_ids: vec![RelationId(package_id.0)],
            eliminated_variables: Vec::new(),
            exported_variables,
            relation_generators: vec![relation],
            representation: crate::compose::message::MessageRepresentation::GeneratorSet,
            projection_strength: ProjectionStrength::CandidateCoverStrong,
            certificate: KernelCertificate::synthetic_for_tests(hash_sequence(
                "p10-test-cert",
                &[package_id.0.to_be_bytes().to_vec()],
            )),
            compression_trace: CompressionTrace::default(),
            cost_trace: ProjectionCostTrace::default(),
            package_hash: hash_sequence("projection-message-initial", &[]),
        };
        message.package_hash = hash_projection_message(&message);
        message
    }

    fn dag(t: VariableId, x: VariableId) -> TargetProjectionDAG {
        let root = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t, x]),
            relation_ids: Vec::new(),
            exported_variables: BTreeSet::from([t]),
            child_block_ids: vec![BlockId(1)],
            parent_block_id: None,
            authorization_hash: Hash([1; 32]),
            duplication_certificates: Vec::new(),
            block_hash: Hash([2; 32]),
        };
        let child = ProjectionBlock {
            block_id: BlockId(1),
            local_variables: BTreeSet::from([x]),
            relation_ids: Vec::new(),
            exported_variables: BTreeSet::from([x]),
            child_block_ids: Vec::new(),
            parent_block_id: Some(BlockId(0)),
            authorization_hash: Hash([3; 32]),
            duplication_certificates: Vec::new(),
            block_hash: Hash([4; 32]),
        };
        TargetProjectionDAG {
            blocks: vec![root, child],
            root_block_id: BlockId(0),
            dag_hash: Hash([5; 32]),
        }
    }

    fn constant(n: i64) -> SparsePolynomialQ {
        crate::types::polynomial::constant_poly(int_q(n))
    }

    fn same_up_to_sign(
        actual: &[crate::types::rational::RationalQ],
        expected: &[crate::types::rational::RationalQ],
    ) -> bool {
        actual == expected
            || actual
                .iter()
                .map(crate::types::rational::neg_q)
                .collect::<Vec<_>>()
                == expected
    }
}
