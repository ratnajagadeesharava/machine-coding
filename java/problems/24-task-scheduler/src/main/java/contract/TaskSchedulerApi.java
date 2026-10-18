package contract;

import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface TaskSchedulerApi {

    /** Runs the task once, when its due time (now + delay) has been reached by a runDue call. */
    String scheduleOnce(long delayMillis, Runnable task);

    /** Empty if periodMillis <= 0. Fixed-rate: occurrence k is due at now + initialDelay + k * period. */
    Optional<String> scheduleRecurring(long initialDelayMillis, long periodMillis, Runnable task);

    /** True if it stopped a task that was still going to run. */
    boolean cancel(String id);

    /** Runs everything due on the calling thread; returns the number of executions performed. */
    int runDue();
}
