package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.WorkerPoolApi;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.concurrent.Callable;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.RejectedExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.Eventually;

class BehaviorTest {

    /** Jobs that report when they start and then wait until released. */
    private static final class Gate {
        final CountDownLatch release = new CountDownLatch(1);
        final AtomicInteger started = new AtomicInteger();
        final AtomicInteger running = new AtomicInteger();
        final AtomicInteger maxRunning = new AtomicInteger();

        Callable<Long> job(long value) {
            return () -> {
                started.incrementAndGet();
                int now = running.incrementAndGet();
                maxRunning.accumulateAndGet(now, Math::max);
                try {
                    release.await(30, TimeUnit.SECONDS);
                } finally {
                    running.decrementAndGet();
                }
                return value;
            };
        }

        void open() {
            release.countDown();
        }

        void awaitStarted(int n) {
            Eventually.assertTrue(() -> started.get() >= n, 10_000, n + " jobs started");
        }
    }

    private WorkerPoolApi pool(int workers, int capacity) {
        return Candidate.of(() -> Solution.create(workers, capacity));
    }

    private static long get(CompletableFuture<Long> f) throws Exception {
        return f.get(10, TimeUnit.SECONDS);
    }

    @Test
    void shouldRunASubmittedJobAndReturnItsResult() throws Exception {
        WorkerPoolApi p = pool(2, 2);

        CompletableFuture<Long> f = p.submit(() -> 21L * 2);

        assertThat(get(f)).isEqualTo(42L);
        p.shutdown();
    }

    @Test
    void shouldNeverRunMoreJobsInParallelThanWorkersAndReachThatParallelism() throws Exception {
        WorkerPoolApi p = pool(3, 20);
        Gate gate = new Gate();
        List<CompletableFuture<Long>> futures = new ArrayList<>();
        for (int i = 0; i < 10; i++) {
            futures.add(p.submit(gate.job(i)));
        }

        gate.awaitStarted(3);
        Eventually.assertStays(() -> gate.started.get() == 3 && gate.running.get() == 3, 200, "exactly 3 run");

        gate.open();
        for (int i = 0; i < 10; i++) {
            assertThat(get(futures.get(i))).isEqualTo(i);
        }
        assertThat(gate.maxRunning.get()).isEqualTo(3);
        p.shutdown();
    }

    @Test
    void shouldRejectWhenTheQueueIsFullAndAllWorkersAreBusy() throws Exception {
        WorkerPoolApi p = pool(2, 3);
        Gate gate = new Gate();
        List<CompletableFuture<Long>> accepted = new ArrayList<>();
        accepted.add(p.submit(gate.job(0)));
        accepted.add(p.submit(gate.job(1)));
        gate.awaitStarted(2);
        for (int i = 2; i < 5; i++) {
            accepted.add(p.submit(gate.job(i)));
        }
        AtomicInteger rejectedRuns = new AtomicInteger();

        assertThatThrownBy(() -> p.submit(() -> {
                    rejectedRuns.incrementAndGet();
                    return 99L;
                }))
                .isInstanceOf(RejectedExecutionException.class);

        gate.open();
        for (int i = 0; i < 5; i++) {
            assertThat(get(accepted.get(i))).isEqualTo(i);
        }
        assertThat(gate.started.get()).isEqualTo(5);
        assertThat(rejectedRuns).hasValue(0);
        p.shutdown();
    }

    @Test
    void shouldAcceptAgainOnceCapacityFreesUp() throws Exception {
        WorkerPoolApi p = pool(1, 1);
        Gate gate = new Gate();
        CompletableFuture<Long> first = p.submit(gate.job(1));
        gate.awaitStarted(1);
        CompletableFuture<Long> second = p.submit(gate.job(2));
        assertThatThrownBy(() -> p.submit(() -> 3L)).isInstanceOf(RejectedExecutionException.class);

        gate.open();
        assertThat(get(first)).isEqualTo(1);
        assertThat(get(second)).isEqualTo(2);

        List<CompletableFuture<Long>> later = new ArrayList<>();
        Eventually.assertTrue(
                () -> {
                    try {
                        later.add(p.submit(() -> 4L));
                        return true;
                    } catch (RejectedExecutionException e) {
                        return false;
                    }
                },
                10_000,
                "a new job is accepted after capacity frees up");
        assertThat(get(later.get(0))).isEqualTo(4);
        p.shutdown();
    }

    @Test
    void shouldStartQueuedJobsInSubmissionOrder() throws Exception {
        WorkerPoolApi p = pool(1, 10);
        Gate gate = new Gate();
        CompletableFuture<Long> blocker = p.submit(gate.job(-1));
        gate.awaitStarted(1);
        List<Integer> order = Collections.synchronizedList(new ArrayList<>());
        List<CompletableFuture<Long>> futures = new ArrayList<>();
        for (int i = 0; i < 10; i++) {
            int id = i;
            futures.add(p.submit(() -> {
                order.add(id);
                return (long) id;
            }));
        }

        gate.open();
        get(blocker);
        for (CompletableFuture<Long> f : futures) {
            get(f);
        }

        assertThat(order).containsExactly(0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
        p.shutdown();
    }

    @Test
    void shouldReportAJobFailureToItsCallerAndKeepTheWorkerAlive() throws Exception {
        WorkerPoolApi p = pool(1, 10);
        List<CompletableFuture<Long>> failing = new ArrayList<>();
        for (int i = 0; i < 5; i++) {
            failing.add(p.submit(() -> {
                throw new IllegalStateException("boom");
            }));
        }
        CompletableFuture<Long> good = p.submit(() -> 7L);

        for (CompletableFuture<Long> f : failing) {
            assertThatThrownBy(() -> f.get(10, TimeUnit.SECONDS))
                    .isInstanceOf(ExecutionException.class)
                    .hasRootCauseMessage("boom");
        }
        assertThat(get(good)).isEqualTo(7L);
        p.shutdown();
    }

    @Test
    void shouldRejectNewJobsAfterShutdown() throws Exception {
        WorkerPoolApi p = pool(2, 2);

        p.shutdown();

        assertThatThrownBy(() -> p.submit(() -> 1L)).isInstanceOf(RejectedExecutionException.class);
    }

    @Test
    void shouldCompleteRunningAndQueuedJobsOnShutdown() throws Exception {
        WorkerPoolApi p = pool(1, 5);
        Gate gate = new Gate();
        List<CompletableFuture<Long>> futures = new ArrayList<>();
        futures.add(p.submit(gate.job(0)));
        gate.awaitStarted(1);
        for (int i = 1; i <= 3; i++) {
            futures.add(p.submit(gate.job(i)));
        }

        p.shutdown();

        assertThatThrownBy(() -> p.submit(() -> 9L)).isInstanceOf(RejectedExecutionException.class);
        assertThat(p.awaitTermination(100)).isFalse();
        gate.open();
        assertThat(p.awaitTermination(10_000)).isTrue();
        for (int i = 0; i <= 3; i++) {
            assertThat(futures.get(i).isDone()).isTrue();
            assertThat(get(futures.get(i))).isEqualTo(i);
        }
    }

    @Test
    void shouldTolerateRepeatedShutdown() throws Exception {
        WorkerPoolApi p = pool(2, 2);
        CompletableFuture<Long> f = p.submit(() -> 5L);

        p.shutdown();
        p.shutdown();

        assertThat(get(f)).isEqualTo(5L);
        assertThat(p.awaitTermination(10_000)).isTrue();
        p.shutdown();
        assertThat(p.awaitTermination(10)).isTrue();
    }

    @Test
    void shouldNotTerminateWithoutShutdown() throws Exception {
        WorkerPoolApi p = pool(2, 2);

        long t0 = System.nanoTime();
        boolean terminated = p.awaitTermination(100);
        long elapsedMs = (System.nanoTime() - t0) / 1_000_000;

        assertThat(terminated).isFalse();
        assertThat(elapsedMs).isBetween(80L, 4_000L);
        assertThat(get(p.submit(() -> 1L))).isEqualTo(1L);
        p.shutdown();
    }

    @Test
    void shouldDeliverEachResultToTheRightCallerWhenManyThreadsSubmit() throws Exception {
        WorkerPoolApi p = pool(4, 1_000);
        AtomicInteger runs = new AtomicInteger();

        List<List<CompletableFuture<Long>>> perThread = Concurrently.collect(8, t -> {
            List<CompletableFuture<Long>> fs = new ArrayList<>();
            for (int i = 0; i < 100; i++) {
                long value = t * 1_000L + i;
                fs.add(p.submit(() -> {
                    runs.incrementAndGet();
                    return value * 2;
                }));
            }
            return fs;
        });

        for (int t = 0; t < 8; t++) {
            for (int i = 0; i < 100; i++) {
                assertThat(get(perThread.get(t).get(i))).isEqualTo((t * 1_000L + i) * 2);
            }
        }
        assertThat(runs).hasValue(800);
        p.shutdown();
        assertThat(p.awaitTermination(10_000)).isTrue();
    }
}
