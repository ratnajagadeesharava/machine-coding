package contract;

import java.util.concurrent.Callable;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.RejectedExecutionException;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface WorkerPoolApi {

    /** Never blocks. Throws RejectedExecutionException when full or shut down; a failing job completes the future exceptionally. */
    CompletableFuture<Long> submit(Callable<Long> job) throws RejectedExecutionException;

    /** Stops accepting jobs; accepted jobs still complete. Returns immediately; idempotent. */
    void shutdown();

    /** True iff shutdown was called and all accepted jobs finished within the timeout. */
    boolean awaitTermination(long timeoutMs) throws InterruptedException;
}
