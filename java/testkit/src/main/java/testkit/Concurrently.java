package testkit;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.function.IntConsumer;
import java.util.function.IntFunction;

/** Runs N tasks that all start at the same instant; fails the test on exception or timeout. */
public final class Concurrently {
    private Concurrently() {}

    public static void run(int threads, IntConsumer body) {
        collect(threads, i -> {
            body.accept(i);
            return null;
        });
    }

    public static <T> List<T> collect(int threads, IntFunction<T> body) {
        return collect(threads, 30_000, body);
    }

    public static <T> List<T> collect(int threads, long timeoutMs, IntFunction<T> body) {
        ExecutorService pool = Executors.newFixedThreadPool(threads);
        CountDownLatch ready = new CountDownLatch(threads);
        CountDownLatch go = new CountDownLatch(1);
        List<Future<T>> futures = new ArrayList<>();
        try {
            for (int i = 0; i < threads; i++) {
                final int idx = i;
                futures.add(pool.submit(() -> {
                    ready.countDown();
                    go.await();
                    return body.apply(idx);
                }));
            }
            ready.await();
            go.countDown();
            List<T> out = new ArrayList<>();
            long deadline = System.nanoTime() + TimeUnit.MILLISECONDS.toNanos(timeoutMs);
            for (Future<T> f : futures) {
                out.add(f.get(Math.max(1, deadline - System.nanoTime()), TimeUnit.NANOSECONDS));
            }
            return out;
        } catch (ExecutionException e) {
            throw new AssertionError("Worker thread failed: " + e.getCause(), e.getCause());
        } catch (TimeoutException e) {
            throw new AssertionError("Timed out after " + timeoutMs + "ms (deadlock or livelock?)");
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new AssertionError("Interrupted", e);
        } finally {
            pool.shutdownNow();
        }
    }
}
