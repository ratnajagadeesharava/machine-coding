package contract;

/** A service behind the gateway. Provided by the tests; thread-safe; fails by throwing. */
@FunctionalInterface
public interface Backend {
    String handle(String path) throws Exception;
}
