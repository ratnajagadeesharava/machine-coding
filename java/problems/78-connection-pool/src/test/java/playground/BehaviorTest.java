package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Pool;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicIntegerArray;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.Eventually;

class BehaviorTest {

    private Pool pool(int size) {
        return Candidate.of(() -> Solution.create(size));
    }

    private static Optional<Integer> acquire(Pool p, long timeoutMs) {
        try {
            return p.acquire(timeoutMs);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new AssertionError("interrupted", e);
        }
    }

    private static Thread start(Runnable body) {
        Thread t = new Thread(body);
        t.setDaemon(true);
        t.start();
        return t;
    }

    @Test
    void shouldHandOutEachConnectionOnceAndThenRunDry() {
        Pool p = pool(3);

        Set<Integer> ids = new HashSet<>();
        for (int i = 0; i < 3; i++) {
            ids.add(acquire(p, 0).orElseThrow());
        }

        assertThat(ids).containsExactlyInAnyOrder(0, 1, 2);
        assertThat(acquire(p, 0)).isEmpty();
        assertThat(p.available()).isZero();
        assertThat(p.inUse()).isEqualTo(3);
    }

    @Test
    void shouldReuseAReleasedConnection() {
        Pool p = pool(1);
        int id = acquire(p, 0).orElseThrow();

        p.release(id);

        assertThat(p.available()).isEqualTo(1);
        assertThat(p.inUse()).isZero();
        assertThat(acquire(p, 0)).hasValue(id);
    }

    @Test
    void shouldGiveUpAfterTheTimeoutWhenNothingIsFree() {
        Pool p = pool(1);
        acquire(p, 0);

        long start = System.nanoTime();
        Optional<Integer> got = acquire(p, 200);
        long elapsedMs = (System.nanoTime() - start) / 1_000_000;

        assertThat(got).isEmpty();
        assertThat(elapsedMs).as("must wait roughly the timeout").isBetween(180L, 1_500L);
    }

    @Test
    void shouldWakeAWaiterPromptlyWhenAConnectionIsReleased() {
        Pool p = pool(1);
        int id = acquire(p, 0).orElseThrow();
        AtomicReference<Optional<Integer>> got = new AtomicReference<>();
        start(() -> got.set(acquire(p, 10_000)));
        Eventually.assertStays(() -> got.get() == null, 100, "the waiter must block while the pool is empty");

        p.release(id);

        Eventually.assertTrue(() -> got.get() != null, 2_000, "waiter should be woken long before its timeout");
        assertThat(got.get()).hasValue(id);
    }

    @Test
    void shouldRejectBadReleases() {
        Pool p = pool(2);
        int id = acquire(p, 0).orElseThrow();
        p.release(id);

        assertThatThrownBy(() -> p.release(id)).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> p.release(1 - id)).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> p.release(5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.release(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThat(p.available()).isEqualTo(2);
    }

    @Test
    void shouldNeverHandTheSameConnectionToTwoHolders() {
        int size = 4;
        Pool p = pool(size);
        AtomicIntegerArray held = new AtomicIntegerArray(size);
        AtomicInteger violations = new AtomicInteger();
        AtomicInteger holders = new AtomicInteger();
        AtomicInteger maxHolders = new AtomicInteger();
        AtomicInteger uses = new AtomicInteger();

        Concurrently.run(16, t -> {
            for (int i = 0; i < 500; i++) {
                int id = acquire(p, 10_000).orElseThrow(() -> new AssertionError("timed out waiting"));
                if (!held.compareAndSet(id, 0, 1)) {
                    violations.incrementAndGet();
                }
                maxHolders.accumulateAndGet(holders.incrementAndGet(), Math::max);
                uses.incrementAndGet();
                Thread.yield();
                holders.decrementAndGet();
                held.set(id, 0);
                p.release(id);
            }
        });

        assertThat(violations).hasValue(0);
        assertThat(maxHolders.get()).isLessThanOrEqualTo(size);
        assertThat(uses).hasValue(16 * 500);
        assertThat(p.available()).isEqualTo(size);
        assertThat(p.inUse()).isZero();
    }

    @Test
    void shouldNotLoseConnectionsWhenWaitersTimeOut() {
        Pool p = pool(2);
        int a = acquire(p, 0).orElseThrow();
        int b = acquire(p, 0).orElseThrow();

        List<Optional<Integer>> results = Concurrently.collect(20, t -> acquire(p, 20));

        assertThat(results).allMatch(Optional::isEmpty);
        p.release(a);
        p.release(b);
        assertThat(p.available()).isEqualTo(2);
        assertThat(acquire(p, 0)).isPresent();
        assertThat(acquire(p, 0)).isPresent();
        assertThat(acquire(p, 0)).isEmpty();
    }

    @Test
    void shouldNotLoseConnectionsWhenTimeoutsRaceWithReleases() {
        int size = 3;
        Pool p = pool(size);
        AtomicInteger acquired = new AtomicInteger();

        Concurrently.run(12, t -> {
            for (int i = 0; i < 400; i++) {
                Optional<Integer> id = acquire(p, 1); // tiny timeout: expires right when releases happen
                if (id.isPresent()) {
                    acquired.incrementAndGet();
                    Thread.yield();
                    p.release(id.get());
                }
            }
        });

        assertThat(acquired.get()).isPositive();
        assertThat(p.available()).as("a connection was lost or duplicated").isEqualTo(size);
        assertThat(p.inUse()).isZero();
    }

    @Test
    void shouldPassASingleConnectionBetweenManyThreads() {
        Pool p = pool(1);
        AtomicInteger uses = new AtomicInteger();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 300; i++) {
                int id = acquire(p, 20_000).orElseThrow(() -> new AssertionError("starved or lost wake-up"));
                uses.incrementAndGet();
                p.release(id);
            }
        });

        assertThat(uses).hasValue(8 * 300);
    }

    @Test
    void shouldFailWaitersAndNewCallersAfterClose() {
        Pool p = pool(1);
        int id = acquire(p, 0).orElseThrow();
        AtomicReference<Throwable> waiterOutcome = new AtomicReference<>();
        start(() -> {
            try {
                p.acquire(10_000);
                waiterOutcome.set(new AssertionError("should not get a connection"));
            } catch (Throwable t) {
                waiterOutcome.set(t);
            }
        });
        Eventually.assertStays(() -> waiterOutcome.get() == null, 100, "waiter blocks first");

        p.close();
        p.close();

        Eventually.assertTrue(() -> waiterOutcome.get() != null, 2_000, "close wakes the waiter");
        assertThat(waiterOutcome.get()).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> p.acquire(0)).isInstanceOf(IllegalStateException.class);
        p.release(id); // returning a connection after close is fine
        assertThat(p.inUse()).isZero();
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            int size = 1 + run % 4;
            Pool p = pool(size);
            AtomicInteger violations = new AtomicInteger();
            AtomicIntegerArray held = new AtomicIntegerArray(size);

            Concurrently.run(8, t -> {
                for (int i = 0; i < 100; i++) {
                    Optional<Integer> id = acquire(p, i % 3 == 0 ? 1 : 5_000);
                    if (id.isEmpty()) {
                        continue;
                    }
                    if (!held.compareAndSet(id.get(), 0, 1)) {
                        violations.incrementAndGet();
                    }
                    held.set(id.get(), 0);
                    p.release(id.get());
                }
            });

            assertThat(violations).as("run %d", run).hasValue(0);
            assertThat(p.available()).as("run %d", run).isEqualTo(size);
        }
    }
}
