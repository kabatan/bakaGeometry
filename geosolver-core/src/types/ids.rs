use serde::{Deserialize, Serialize};

use crate::types::hash::hash_tagged;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VariableId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RelationId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BlockId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PackageId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KernelPlanId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StableId(pub [u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdCounter {
    next: u32,
}

impl IdCounter {
    pub fn new(start: u32) -> Self {
        Self { next: start }
    }
}

pub fn fresh_variable_id(counter: &mut IdCounter) -> VariableId {
    let id = VariableId(counter.next);
    counter.next = counter.next.checked_add(1).expect("id counter overflow");
    id
}

pub fn fresh_relation_id(counter: &mut IdCounter) -> RelationId {
    let id = RelationId(counter.next);
    counter.next = counter.next.checked_add(1).expect("id counter overflow");
    id
}

pub fn fresh_block_id(counter: &mut IdCounter) -> BlockId {
    let id = BlockId(counter.next);
    counter.next = counter.next.checked_add(1).expect("id counter overflow");
    id
}

pub fn fresh_package_id(counter: &mut IdCounter) -> PackageId {
    let id = PackageId(counter.next);
    counter.next = counter.next.checked_add(1).expect("id counter overflow");
    id
}

pub fn fresh_kernel_plan_id(counter: &mut IdCounter) -> KernelPlanId {
    let id = KernelPlanId(counter.next);
    counter.next = counter.next.checked_add(1).expect("id counter overflow");
    id
}

pub fn stable_id_from_name(name: &str, namespace: &str) -> StableId {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(namespace.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(name.as_bytes());
    StableId(hash_tagged("stable-id", &bytes).0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_ids_are_monotone() {
        let mut counter = IdCounter::new(7);
        assert_eq!(fresh_variable_id(&mut counter), VariableId(7));
        assert_eq!(fresh_variable_id(&mut counter), VariableId(8));
        assert_eq!(fresh_relation_id(&mut counter), RelationId(9));
        assert_eq!(fresh_block_id(&mut counter), BlockId(10));
        assert_eq!(fresh_package_id(&mut counter), PackageId(11));
        assert_eq!(fresh_kernel_plan_id(&mut counter), KernelPlanId(12));
    }

    #[test]
    fn stable_ids_are_namespace_bound() {
        assert_eq!(
            stable_id_from_name("x", "vars"),
            stable_id_from_name("x", "vars")
        );
        assert_ne!(
            stable_id_from_name("x", "vars"),
            stable_id_from_name("x", "relations")
        );
    }
}
