package contract;

import java.util.List;

/**
 * Stable storage of one node (provided by the tests; it survives a restart of the node: a restarted node is a NEW
 * instance created with the SAME log). Appending is atomic and durable as soon as the call returns.
 */
public interface DurableLog {

    void append(String record);

    /** Everything appended so far, oldest first. */
    List<String> records();
}
