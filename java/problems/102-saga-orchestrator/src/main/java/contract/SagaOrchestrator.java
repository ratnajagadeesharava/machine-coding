package contract;

import java.util.List;

/**
 * Runs sagas: a sequence of steps, each with a compensation.
 *
 * <p>Rules (the tests check all of them):
 * <ul>
 *   <li>Steps run in order. A step is attempted at most 3 times; before attempt 2 and 3 the orchestrator calls the
 *       sleeper with 100 and 200 (ms). A step that fails 3 times fails the saga.</li>
 *   <li>Then the steps that completed are compensated in REVERSE order (the failed step is not compensated). A
 *       compensation is attempted at most 5 times, sleeping 100, 200, 400, 800 between attempts. If one still fails,
 *       the saga ends as COMPENSATION_FAILED and the remaining compensations are NOT run.</li>
 *   <li>Progress is written to the {@link SagaLog} so that a new orchestrator on the same log continues where the
 *       old one stopped: a finished step is not executed again, a finished compensation is not run again.</li>
 *   <li>{@code run} on a saga that already finished returns its final status without calling any step. {@code run}
 *       on a saga that is running right now (another thread) returns RUNNING immediately.</li>
 *   <li>Different sagas run concurrently; the orchestrator must be thread-safe.</li>
 * </ul>
 */
public interface SagaOrchestrator {

    /** Runs (or continues) the saga to its end on the calling thread and returns the final status. */
    SagaStatus run(String sagaId, List<SagaStep> steps);

    /** The status derived from the log (also valid right after a restart). */
    SagaStatus status(String sagaId);
}
