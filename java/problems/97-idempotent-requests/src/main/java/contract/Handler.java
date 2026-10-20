package contract;

/** The business operation behind an idempotency key. Provided by the tests; it fails by throwing an unchecked exception. */
@FunctionalInterface
public interface Handler {
    String handle(String requestBody);
}
