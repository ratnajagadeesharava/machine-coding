package contract;

/** Thrown by {@link DocStore#edit} when a stale edit overlaps a change made since its base version. Provided. */
public class MergeConflictException extends RuntimeException {
    public MergeConflictException(String message) {
        super(message);
    }
}
