package testkit;

import java.util.function.Supplier;
import org.junit.jupiter.api.Assumptions;

/**
 * Creates the system under test through the candidate's factory.
 * While the factory still throws UnsupportedOperationException the test is SKIPPED, so a fresh checkout builds green.
 * Set STRICT=1 to turn "not implemented" into a failure.
 */
public final class Candidate {
    private Candidate() {}

    public static boolean strict() {
        return "1".equals(System.getenv("STRICT"));
    }

    public static <T> T of(Supplier<T> factory) {
        try {
            return factory.get();
        } catch (UnsupportedOperationException e) {
            if (strict()) {
                throw new AssertionError("Not implemented (STRICT=1): wire your design in solution.Solution", e);
            }
            Assumptions.assumeTrue(false, "Not implemented yet: wire your design in solution.Solution");
            throw e; // unreachable
        }
    }
}
