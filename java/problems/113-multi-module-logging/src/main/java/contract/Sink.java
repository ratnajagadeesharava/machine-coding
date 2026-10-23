package contract;

/** A simulated destination (a log file or a monitoring tool). Implemented by the tests, not by you. */
public interface Sink {

    /**
     * Writes one line. Returns {@code true} when the line was durably written, {@code false} when the write failed.
     * An implementation may also throw a RuntimeException, which counts as a failed write too.
     * The line has the format documented on {@link LogService}.
     */
    boolean write(String line);
}
