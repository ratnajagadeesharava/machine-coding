package contract;

/** The same idempotency key was reused for a different request body. Provided. */
public class KeyConflictException extends RuntimeException {
    public KeyConflictException(String message) {
        super(message);
    }
}
