package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.WorkerPoolApi;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.RejectedExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private WorkerPoolApi pool(int workers, int capacity) {
        return Candidate.of(() -> Solution.create(workers, capacity));
    }

    private static long get(CompletableFuture<Long> f) throws Exception {
        return f.get(10, TimeUnit.SECONDS);
    }

    private static void await(CountDownLatch latch) {
        try {
            latch.await(30, TimeUnit.SECONDS);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }

    @Test
    void zeroQueueCapacityShouldAcceptOnlyAsManyJobsAsThereAreFreeWorkers() throws Exception {
        WorkerPoolApi p = pool(2, 0);
        CountDownLatch release = new CountDownLatch(1);
        CountDownLatch started = new CountDownLatch(2);

        CompletableFuture<Long> a = p.submit(() -> {
            started.countDown();
            await(release);
            return 1L;
        });
        CompletableFuture<Long> b = p.submit(() -> {
            started.countDown();
            await(release);
            return 2L;
        });
        assertThat(started.await(10, TimeUnit.SECONDS)).isTrue();

        assertThatThrownBy(() -> p.submit(() -> 3L)).isInstanceOf(RejectedExecutionException.class);
        release.countDown();
        assertThat(get(a)).isEqualTo(1L);
        assertThat(get(b)).isEqualTo(2L);
        p.shutdown();
    }

    @Test
    void shouldKeepFullParallelismAfterManyFailingJobs() throws Exception {
        WorkerPoolApi p = pool(3, 100);
        List<CompletableFuture<Long>> failing = new ArrayList<>();
        for (int i = 0; i < 30; i++) {
            if (i % 2 == 0) {
                failing.add(p.submit(() -> {
                    throw new RuntimeException("bad");
                }));
            } else {
                failing.add(p.submit(() -> 1L));
            }
        }
        for (int i = 0; i < 30; i++) {
            CompletableFuture<Long> f = failing.get(i);
            if (i % 2 == 0) {
                assertThatThrownBy(() -> f.get(10, TimeUnit.SECONDS)).isInstanceOf(ExecutionException.class);
            } else {
                assertThat(get(f)).isEqualTo(1L);
            }
        }

        CountDownLatch allThree = new CountDownLatch(3);
        List<CompletableFuture<Long>> meeting = new ArrayList<>();
        for (int i = 0; i < 3; i++) {
            meeting.add(p.submit(() -> {
                allThree.countDown();
                return allThree.await(10, TimeUnit.SECONDS) ? 1L : 0L;
            }));
        }

        for (CompletableFuture<Long> f : meeting) {
            assertThat(get(f)).as("three workers must still run in parallel").isEqualTo(1L);
        }
        p.shutdown();
    }

    @Test
    void aJobShouldBeAbleToSubmitFurtherJobsToTheSamePool() throws Exception {
        WorkerPoolApi p = pool(1, 1);
        AtomicReference<CompletableFuture<Long>> inner = new AtomicReference<>();

        CompletableFuture<Long> outer = p.submit(() -> {
            inner.set(p.submit(() -> 10L));
            return 1L;
        });

        assertThat(get(outer)).isEqualTo(1L);
        assertThat(get(inner.get())).isEqualTo(10L);
        p.shutdown();
        assertThat(p.awaitTermination(10_000)).isTrue();
    }

    @Test
    void shutdownOnAnIdlePoolShouldTerminateImmediatelyAndRejectLaterJobs() throws Exception {
        WorkerPoolApi p = pool(3, 3);

        p.shutdown();

        assertThat(p.awaitTermination(10_000)).isTrue();
        assertThatThrownBy(() -> p.submit(() -> 1L)).isInstanceOf(RejectedExecutionException.class);
    }

    @Test
    void shouldRunAcceptedQueuedJobsEvenWhenShutdownIsCalledBeforeAnyWorkerFreesUp() throws Exception {
        WorkerPoolApi p = pool(2, 10);
        CountDownLatch release = new CountDownLatch(1);
        List<CompletableFuture<Long>> futures = new ArrayList<>();
        for (int i = 0; i < 12; i++) {
            long v = i;
            futures.add(p.submit(() -> {
                await(release);
                return v;
            }));
        }
        assertThatThrownBy(() -> p.submit(() -> 0L)).isInstanceOf(RejectedExecutionException.class);

        p.shutdown();
        release.countDown();

        assertThat(p.awaitTermination(10_000)).isTrue();
        for (int i = 0; i < 12; i++) {
            assertThat(get(futures.get(i))).isEqualTo(i);
        }
    }
}
