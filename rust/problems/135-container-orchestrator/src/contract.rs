//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//! Ids are compared as exact strings and sorted with the default `String` ordering. A failed call changes nothing.

#[derive(Debug, PartialEq, Eq)]
pub enum OrchError {
    /// the node id is not registered
    UnknownNode,
    /// the container id is not known (neither running nor pending)
    UnknownContainer,
    /// node id / container id already in use
    AlreadyExists,
    /// blank id, or cpu / memory equal to zero
    InvalidArgument,
}

/// Node selection strategy; see the README for the exact rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    FirstFit,
    BestFit,
    WorstFit,
}

pub trait Orchestrator: Send + Sync {
    /// Registers a node. `InvalidArgument` for a blank id or zero cpu / memory, `AlreadyExists` for a known id.
    /// Afterwards pending containers are retried.
    fn add_node(&self, node_id: &str, cpu_millis: u32, mem_mb: u32) -> Result<(), OrchError>;

    /// Node failure / decommission. `UnknownNode` if the node is not registered. Every container on it becomes
    /// PENDING and is immediately rescheduled (all pending containers are retried, in retry order, by the active
    /// strategy, without preemption) onto the remaining nodes. Returns one entry per container that was on the
    /// node, sorted by container id, formatted `"containerId->nodeId"` or `"containerId->PENDING"` (the state after
    /// rescheduling).
    fn remove_node(&self, node_id: &str) -> Result<Vec<String>, OrchError>;

    /// Changes the strategy used for every later placement. Running containers do not move.
    fn set_strategy(&self, strategy: Strategy);

    /// Submits a container and returns the node it was placed on, or `None` if it is PENDING.
    /// `InvalidArgument` for a blank id or zero cpu / memory, `AlreadyExists` if the id is known (running or pending).
    fn submit(
        &self,
        container_id: &str,
        cpu_millis: u32,
        mem_mb: u32,
        priority: i32,
    ) -> Result<Option<String>, OrchError>;

    /// Removes a running or pending container (`UnknownContainer` if unknown). Capacity it held is freed and
    /// pending containers are retried.
    fn remove(&self, container_id: &str) -> Result<(), OrchError>;

    /// The node id the container runs on, or the literal `"PENDING"`. `UnknownContainer` if unknown.
    fn placement(&self, container_id: &str) -> Result<String, OrchError>;

    /// `"cpuUsed/cpuTotal,memUsed/memTotal"`, e.g. `"500/1000,256/2048"`. `UnknownNode` if unknown.
    fn node_usage(&self, node_id: &str) -> Result<String, OrchError>;

    /// Ids of the containers running on the node, sorted ascending. `UnknownNode` if unknown.
    fn containers_on(&self, node_id: &str) -> Result<Vec<String>, OrchError>;

    /// Ids of the PENDING containers in the order they would be retried.
    fn pending(&self) -> Vec<String>;
}
