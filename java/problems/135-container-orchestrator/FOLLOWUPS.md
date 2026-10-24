# Follow-ups — 135 Container Orchestrator

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Affinity and anti-affinity
A container can ask to run on the same node as another container (affinity) or never on a node that hosts a container from a given group (anti-affinity, e.g. replicas of one service). Where does the constraint live, how does it interact with the three strategies, with preemption (can you evict a container another one has affinity to?) and with node failure? What happens to a pending container whose constraint can never be satisfied?

### V3 — Taints and tolerations
Nodes can carry taints ("gpu", "maintenance") and a container only runs on a node whose taints it tolerates. Add a way to taint and untaint a live node. What happens to containers already running on a node that just got a taint they do not tolerate (evict now, never, or at next retry)? How does this change the retry pass and the "fits nowhere" test?

### V4 — Replicas, deployments and rolling updates
A deployment asks for N identical replicas of a container spec. Scale up/down, and roll out a new spec without ever running more than N + 1 or fewer than N - 1 replicas. Who owns the replica ids? How do you keep the rollout correct when a node fails halfway through, or when there is not enough capacity to start the extra replica?

### V5 — Autoscaling and overcommit
Add a node-pool autoscaler: when containers have been pending, add a node of a configured size; when nodes are empty, remove them (injectable clock, no real sleeping). Then allow CPU overcommit with a ratio (nodes advertise `cpu * 1.5` schedulable CPU while memory stays strict). What changes in the fit test, in the usage string, and in what a node failure means for the invariants?
