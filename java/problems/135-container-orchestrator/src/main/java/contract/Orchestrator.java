package contract;

import java.util.List;
import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Every error below is an {@link IllegalArgumentException}. Ids are compared as exact strings and sorted with
 * {@code String.compareTo}. A failed call changes nothing.
 */
public interface Orchestrator {

    /**
     * Registers a node. Error if the id is null/blank or already exists, or if cpuMillis or memMb is not positive.
     * Afterwards pending containers are retried.
     */
    void addNode(String nodeId, int cpuMillis, int memMb);

    /**
     * Node failure / decommission. Error if the node is unknown. Every container on it becomes PENDING and is
     * immediately rescheduled (all pending containers are retried, in retry order, by the active strategy, without
     * preemption) onto the remaining nodes. Returns one entry per container that was on the node, sorted by
     * container id, formatted {@code "containerId->nodeId"} or {@code "containerId->PENDING"} (the state after
     * rescheduling).
     */
    List<String> removeNode(String nodeId);

    /** Changes the strategy used for every later placement. Error if null. Running containers do not move. */
    void setStrategy(Strategy strategy);

    /**
     * Submits a container and returns the node it was placed on, or empty if it is PENDING.
     * Error if the id is null/blank or already known (running or pending), or cpuMillis/memMb is not positive.
     */
    Optional<String> submit(String containerId, int cpuMillis, int memMb, int priority);

    /** Removes a running or pending container. Error if unknown. Capacity it held is freed and pending ones are retried. */
    void remove(String containerId);

    /** The node id the container runs on, or the literal {@code "PENDING"}. Error if the container is unknown. */
    String placement(String containerId);

    /** {@code "cpuUsed/cpuTotal,memUsed/memTotal"}, e.g. {@code "500/1000,256/2048"}. Error if the node is unknown. */
    String nodeUsage(String nodeId);

    /** Ids of the containers running on the node, sorted ascending. Error if the node is unknown. */
    List<String> containersOn(String nodeId);

    /** Ids of the PENDING containers in the order they would be retried. */
    List<String> pending();
}
