package contract;

import java.util.Optional;
import java.util.Set;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface JobScheduler {

    /** Criterion name: among eligible machines pick the one with the fewest unfinished jobs. */
    String FEWEST_UNFINISHED = "FEWEST_UNFINISHED";

    /** Criterion name: among eligible machines pick the one with the most finished jobs. */
    String MOST_FINISHED = "MOST_FINISHED";

    /**
     * Registers a machine with the capabilities it offers (may be empty). Its counters start at 0.
     * IllegalArgumentException if the id is null/blank, already registered, or the set is null or holds a null/blank
     * capability. The scheduler must not be affected by later changes to the caller's set.
     */
    void addMachine(String machineId, Set<String> capabilities);

    /**
     * Sets the criterion used by {@link #submitJob(String, Set)}. Until it is called the default is
     * {@link #FEWEST_UNFINISHED}. IllegalArgumentException for an unknown criterion name (the previous default stays).
     */
    void setDefaultCriterion(String criterion);

    /** Same as {@link #submitJob(String, Set, String)} using the scheduler's default criterion. */
    Optional<String> submitJob(String jobId, Set<String> requiredCapabilities);

    /**
     * Assigns the job to an eligible machine and returns its id. A machine is eligible when it offers ALL required
     * capabilities (an empty requirement fits every machine). Among eligible machines the criterion decides; ties are
     * broken by the lexicographically smallest machine id. The chosen machine's unfinished count goes up by one.
     * Returns empty when no machine is eligible; in that case nothing is recorded and the job id stays unused.
     * IllegalArgumentException for a null/blank job id, a job id that is already assigned (finished or not), a null
     * capability set or one holding a null/blank entry, or an unknown/null criterion. Nothing changes on error.
     */
    Optional<String> submitJob(String jobId, Set<String> requiredCapabilities, String criterion);

    /**
     * Marks an assigned job as finished: its machine's unfinished count goes down by one and its finished count goes
     * up by one. IllegalArgumentException for an unknown job id (never assigned); IllegalStateException if the job was
     * already finished. Nothing changes on error.
     */
    void finishJob(String jobId);

    /** Number of jobs assigned to the machine and not yet finished. IllegalArgumentException for an unknown machine. */
    int unfinishedCount(String machineId);

    /** Number of jobs the machine has finished. IllegalArgumentException for an unknown machine. */
    int finishedCount(String machineId);

    /** The machine a job was assigned to. IllegalArgumentException for an unknown (never assigned) job id. */
    String machineOf(String jobId);
}
