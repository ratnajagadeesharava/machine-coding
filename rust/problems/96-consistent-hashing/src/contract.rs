//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`): lookups run while nodes join and leave.
//!
//! The mapping must depend only on the set of nodes (names and weights) and the key - never on the order in which
//! nodes were added or on a random seed - so that two processes holding the same membership agree on every key.

#[derive(Debug, PartialEq, Eq)]
pub enum RingError {
    /// the node is already on the ring, the name is blank, or the weight is 0
    InvalidNode,
}

pub trait HashRing: Send + Sync {
    /// Adds a node. A node of weight w owns about w times as many keys as a node of weight 1 (weight >= 1).
    fn add_node(&self, node: &str, weight: u32) -> Result<(), RingError>;

    /// Returns false if the node was not on the ring. Its keys move to the remaining nodes.
    fn remove_node(&self, node: &str) -> bool;

    /// The node responsible for the key; `None` if the ring is empty.
    fn node_for(&self, key: &str) -> Option<String>;

    /// Up to `n` DISTINCT nodes for replication: first `node_for(key)`, then the following distinct nodes clockwise
    /// on the ring. Fewer than `n` if the ring has fewer nodes. Empty for an empty ring.
    fn nodes_for(&self, key: &str, n: usize) -> Vec<String>;

    /// All nodes, sorted.
    fn nodes(&self) -> Vec<String>;
}
