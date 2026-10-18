package contract;

/** An output destination supplied by the caller. Receives complete lines (no trailing newline). */
@FunctionalInterface
public interface LogSink {
    void write(String line);
}
