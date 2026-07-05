use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::graph::hypergraph::{
    connected_components, HypergraphComponent, RelationVariableHypergraph,
};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetInfluenceGraph {
    pub target: VariableId,
    pub target_component: HypergraphComponent,
    pub independent_components: Vec<HypergraphComponent>,
    pub influence_hash: Hash,
}

pub fn build_target_influence_graph(
    h: &RelationVariableHypergraph,
    target: VariableId,
) -> TargetInfluenceGraph {
    let components = connected_components(h);
    let target_component = components
        .iter()
        .find(|component| component.variables.contains(&target))
        .cloned()
        .unwrap_or_else(|| HypergraphComponent {
            relation_ids: BTreeSet::new(),
            variables: BTreeSet::from([target]),
            component_hash: hash_sequence(
                "hypergraph-component",
                &[target.0.to_be_bytes().to_vec()],
            ),
        });
    let independent_components = components
        .into_iter()
        .filter(|component| component.component_hash != target_component.component_hash)
        .collect::<Vec<_>>();
    let influence_hash = hash_influence(target, &target_component, &independent_components);
    TargetInfluenceGraph {
        target,
        target_component,
        independent_components,
        influence_hash,
    }
}

fn hash_influence(
    target: VariableId,
    target_component: &HypergraphComponent,
    independent_components: &[HypergraphComponent],
) -> Hash {
    let mut chunks = vec![
        target.0.to_be_bytes().to_vec(),
        target_component.component_hash.0.to_vec(),
    ];
    for component in independent_components {
        chunks.push(component.component_hash.0.to_vec());
    }
    hash_sequence("target-influence", &chunks)
}
