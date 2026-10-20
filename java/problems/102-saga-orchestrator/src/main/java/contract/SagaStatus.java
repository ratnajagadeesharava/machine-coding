package contract;

public enum SagaStatus {
    /** never seen by this log */
    NOT_FOUND,
    /** started, not finished (possibly being run right now) */
    RUNNING,
    /** every step executed */
    COMPLETED,
    /** a step failed for good and every earlier step was compensated */
    COMPENSATED,
    /** a compensation failed for good: needs a human */
    COMPENSATION_FAILED
}
