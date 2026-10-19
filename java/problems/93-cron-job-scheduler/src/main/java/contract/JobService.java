package contract;

import java.util.List;
import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Time comes from the clock handed to the factory. A "minute" is epoch milliseconds / 60,000 (UTC); epoch minute 0
 * is Thursday 1970-01-01 00:00. Nothing runs by itself: the caller drives the scheduler by calling {@link #runDue()}.
 */
public interface JobService {

    /**
     * Defines a job. {@code cron} has THREE fields "minute hour weekday" (weekday 0-6, 0 = Sunday); each field is
     * {@code *}, {@code *}/n, a number, a range a-b or a comma list of those. The first run is the first matching
     * minute strictly after the minute of the definition. A failing run is retried up to {@code maxAttempts} attempts
     * in total; after failure number k (k = 1, 2, ...) the next attempt is due {@code min(initialBackoffMs * 2^(k-1),
     * maxBackoffMs)} later. IllegalArgumentException for a duplicate name, a bad cron expression, maxAttempts < 1 or
     * negative/zero backoffs.
     */
    void define(String name, String cron, Job job, int maxAttempts, long initialBackoffMs, long maxBackoffMs);

    /**
     * Runs, on the calling thread, everything that is due now and returns the number of executions (attempts) made.
     * Order: by due time, then by definition order. Per job at most ONE execution per call, and
     * <ul>
     *   <li>a retry that is due runs (attempt k+1 of the same slot);</li>
     *   <li>otherwise if the next cron slot is due it runs, for the LATEST matching minute not after now (missed
     *       earlier minutes are not run and not recorded);</li>
     *   <li>if a cron slot is due while the job is still waiting for a retry, the slot is SKIPPED (recorded, not run);
     *       no two slots overlap.</li>
     * </ul>
     * After a slot ran or was skipped, the next slot is the first matching minute after the current minute.
     * Failures never escape and never stop other jobs. Concurrent callers must never execute the same attempt twice.
     */
    int runDue();

    /** Paused jobs neither run nor retry. */
    void pause(String name);

    /** The next slot is recomputed from now (first matching minute strictly after the current minute). */
    void resume(String name);

    void remove(String name);

    /** Epoch minute of the next cron slot; empty if the job is paused. IllegalArgumentException if unknown. */
    Optional<Long> nextSlot(String name);

    /**
     * Everything that happened for the job, oldest first: "slot:attempt:OUTCOME" with OUTCOME one of OK, FAILED (will
     * be retried), DEAD (last attempt failed), SKIPPED (attempt is 0).
     */
    List<String> history(String name);

    /** Jobs that ran out of attempts, as "name:slot", in the order they died. */
    List<String> deadLetters();
}
