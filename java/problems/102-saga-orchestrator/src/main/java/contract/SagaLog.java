package contract;

import java.util.List;

/**
 * Stable storage of the orchestrator (provided by the tests; it survives a restart: a restarted orchestrator is a NEW
 * instance created with the SAME log). Appending is atomic and durable as soon as the call returns.
 */
public interface SagaLog {

    void append(String record);

    /** Everything appended so far, oldest first. */
    List<String> records();
}
