package contract;

import java.util.function.BooleanSupplier;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface CircuitBreakerApi {

    /**
     * Runs the operation unless refused. The operation returns true for success; throwing counts as failure.
     * Returns "SUCCESS", "FAILURE" or "REJECTED" (refused, operation not run).
     */
    String call(BooleanSupplier operation);

    /** "CLOSED", "OPEN" or "HALF_OPEN", at the current time. */
    String state();
}
