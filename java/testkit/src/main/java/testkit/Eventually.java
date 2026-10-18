package testkit;

import java.util.function.BooleanSupplier;

/** Polling helpers for asynchronous behavior. */
public final class Eventually {
    private Eventually() {}

    public static boolean await(BooleanSupplier condition, long timeoutMs) {
        long deadline = System.nanoTime() + timeoutMs * 1_000_000L;
        while (System.nanoTime() < deadline) {
            if (condition.getAsBoolean()) {
                return true;
            }
            sleep();
        }
        return condition.getAsBoolean();
    }

    public static void assertTrue(BooleanSupplier condition, long timeoutMs, String message) {
        if (!await(condition, timeoutMs)) {
            throw new AssertionError("Condition not met within " + timeoutMs + "ms: " + message);
        }
    }

    /** Asserts that the condition stays true for the whole window (e.g. "this thread is still blocked"). */
    public static void assertStays(BooleanSupplier condition, long windowMs, String message) {
        long deadline = System.nanoTime() + windowMs * 1_000_000L;
        while (System.nanoTime() < deadline) {
            if (!condition.getAsBoolean()) {
                throw new AssertionError("Condition did not hold: " + message);
            }
            sleep();
        }
    }

    private static void sleep() {
        try {
            Thread.sleep(2);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
}
