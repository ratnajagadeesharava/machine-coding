package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.BlockingBuffer;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Eventually;

class BehaviorTest {

    private BlockingBuffer<Integer> buffer(int capacity) {
        return Candidate.of(() -> Solution.<Integer>create(capacity));
    }

    private static Thread start(ThrowingRunnable body) {
        Thread t = new Thread(() -> {
            try {
                body.run();
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
        });
        t.setDaemon(true);
        t.start();
        return t;
    }

    private interface ThrowingRunnable {
        void run() throws InterruptedException;
    }

    @Test
    void shouldDeliverItemsInFifoOrder() throws Exception {
        BlockingBuffer<Integer> b = buffer(5);
        for (int i = 1; i <= 5; i++) {
            b.put(i);
        }

        for (int i = 1; i <= 5; i++) {
            assertThat(b.take()).hasValue(i);
        }
        assertThat(b.size()).isZero();
    }

    @Test
    void shouldReportSize() throws Exception {
        BlockingBuffer<Integer> b = buffer(3);
        assertThat(b.size()).isZero();

        b.put(1);
        b.put(2);
        assertThat(b.size()).isEqualTo(2);

        b.take();
        assertThat(b.size()).isEqualTo(1);
    }

    @Test
    void shouldBlockPutWhileFullUntilSpaceIsAvailable() throws Exception {
        BlockingBuffer<Integer> b = buffer(2);
        b.put(1);
        b.put(2);
        AtomicBoolean done = new AtomicBoolean();
        start(() -> {
            b.put(3);
            done.set(true);
        });

        Eventually.assertStays(() -> !done.get(), 150, "put must wait while the buffer is full");
        assertThat(b.size()).isEqualTo(2);

        assertThat(b.take()).hasValue(1);
        Eventually.assertTrue(done::get, 5_000, "put completes once there is space");
        assertThat(b.take()).hasValue(2);
        assertThat(b.take()).hasValue(3);
    }

    @Test
    void shouldBlockTakeWhileEmptyUntilAnItemArrives() throws Exception {
        BlockingBuffer<Integer> b = buffer(2);
        AtomicReference<Optional<Integer>> got = new AtomicReference<>();
        start(() -> got.set(b.take()));

        Eventually.assertStays(() -> got.get() == null, 150, "take must wait while the buffer is empty");

        b.put(42);

        Eventually.assertTrue(() -> got.get() != null, 5_000, "take returns the item");
        assertThat(got.get()).hasValue(42);
    }

    @Test
    void shouldTimeOutOfferWhileFull() throws Exception {
        BlockingBuffer<Integer> b = buffer(1);
        b.put(1);

        long t0 = System.nanoTime();
        boolean accepted = b.offer(2, 100);
        long elapsedMs = (System.nanoTime() - t0) / 1_000_000;

        assertThat(accepted).isFalse();
        assertThat(elapsedMs).isBetween(80L, 4_000L);
        assertThat(b.size()).isEqualTo(1);
        assertThat(b.take()).hasValue(1);
        assertThat(b.poll(0)).isEmpty();
    }

    @Test
    void shouldAcceptOfferWhenSpaceAppearsBeforeTheTimeout() throws Exception {
        BlockingBuffer<Integer> b = buffer(1);
        b.put(1);
        AtomicReference<Boolean> accepted = new AtomicReference<>();
        start(() -> accepted.set(b.offer(2, 30_000)));
        Eventually.assertStays(() -> accepted.get() == null, 100, "offer waits while full");

        assertThat(b.take()).hasValue(1);

        Eventually.assertTrue(() -> accepted.get() != null, 5_000, "offer returns once space exists");
        assertThat(accepted.get()).isTrue();
        assertThat(b.take()).hasValue(2);
    }

    @Test
    void shouldTimeOutPollWhileEmpty() throws Exception {
        BlockingBuffer<Integer> b = buffer(2);

        long t0 = System.nanoTime();
        Optional<Integer> result = b.poll(100);
        long elapsedMs = (System.nanoTime() - t0) / 1_000_000;

        assertThat(result).isEmpty();
        assertThat(elapsedMs).isBetween(80L, 4_000L);
    }

    @Test
    void shouldReturnPollResultWhenAnItemArrivesBeforeTheTimeout() throws Exception {
        BlockingBuffer<Integer> b = buffer(2);
        AtomicReference<Optional<Integer>> got = new AtomicReference<>();
        start(() -> got.set(b.poll(30_000)));
        Eventually.assertStays(() -> got.get() == null, 100, "poll waits while empty");

        b.put(7);

        Eventually.assertTrue(() -> got.get() != null, 5_000, "poll returns the item well before its timeout");
        assertThat(got.get()).hasValue(7);
    }

    @Test
    void shouldNotWaitWhenTheTimeoutIsZero() throws Exception {
        BlockingBuffer<Integer> b = buffer(1);

        assertThat(b.poll(0)).isEmpty();
        assertThat(b.offer(1, 0)).isTrue();
        assertThat(b.offer(2, 0)).isFalse();
        assertThat(b.poll(0)).hasValue(1);
        assertThat(b.size()).isZero();
    }

    @Test
    void shouldHandOutRemainingItemsAfterCloseAndThenReportNoMoreItems() throws Exception {
        BlockingBuffer<Integer> b = buffer(5);
        b.put(1);
        b.put(2);

        b.close();

        assertThat(b.size()).isEqualTo(2);
        assertThat(b.take()).hasValue(1);
        assertThat(b.poll(10)).hasValue(2);
        assertThat(b.take()).isEmpty();
        assertThat(b.poll(10)).isEmpty();
        assertThat(b.size()).isZero();
    }

    @Test
    void shouldRejectPutAndOfferAfterClose() throws Exception {
        BlockingBuffer<Integer> b = buffer(5);
        b.put(1);
        b.close();
        b.close();

        assertThatThrownBy(() -> b.put(2)).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> b.offer(3, 10)).isInstanceOf(IllegalStateException.class);
        assertThat(b.size()).isEqualTo(1);
        assertThat(b.take()).hasValue(1);
        assertThat(b.take()).isEmpty();
    }

    @Test
    void shouldWakeBlockedTakersWhenClosed() throws Exception {
        BlockingBuffer<Integer> b = buffer(2);
        List<AtomicReference<Optional<Integer>>> results = new ArrayList<>();
        for (int i = 0; i < 3; i++) {
            AtomicReference<Optional<Integer>> r = new AtomicReference<>();
            results.add(r);
            start(() -> r.set(b.take()));
        }
        Eventually.assertStays(() -> results.stream().allMatch(r -> r.get() == null), 150, "takers wait");

        b.close();

        Eventually.assertTrue(
                () -> results.stream().allMatch(r -> r.get() != null), 5_000, "close wakes every blocked taker");
        assertThat(results).allSatisfy(r -> assertThat(r.get()).isEmpty());
    }

    @Test
    void shouldFailBlockedProducersWithoutStoringTheirItemsWhenClosed() throws Exception {
        BlockingBuffer<Integer> b = buffer(1);
        b.put(1);
        AtomicReference<Throwable> putFailure = new AtomicReference<>();
        AtomicReference<Throwable> offerFailure = new AtomicReference<>();
        start(() -> {
            try {
                b.put(2);
            } catch (IllegalStateException e) {
                putFailure.set(e);
            }
        });
        start(() -> {
            try {
                b.offer(3, 60_000);
            } catch (IllegalStateException e) {
                offerFailure.set(e);
            }
        });
        Eventually.assertStays(
                () -> putFailure.get() == null && offerFailure.get() == null, 150, "producers wait while full");

        b.close();

        Eventually.assertTrue(
                () -> putFailure.get() != null && offerFailure.get() != null,
                5_000,
                "close fails every blocked producer");
        assertThat(b.size()).isEqualTo(1);
        assertThat(b.take()).hasValue(1);
        assertThat(b.take()).isEmpty();
    }

    @Test
    void shouldThrowInterruptedExceptionWhenInterruptedWhileWaiting() throws Exception {
        // Java only.
        BlockingBuffer<Integer> b = buffer(1);
        AtomicBoolean interrupted = new AtomicBoolean();
        CountDownLatch finished = new CountDownLatch(1);
        Thread t = new Thread(() -> {
            try {
                b.take();
            } catch (InterruptedException e) {
                interrupted.set(true);
            }
            finished.countDown();
        });
        t.setDaemon(true);
        t.start();
        Eventually.assertStays(() -> finished.getCount() == 1, 100, "taker waits");

        t.interrupt();

        assertThat(finished.await(5, java.util.concurrent.TimeUnit.SECONDS)).isTrue();
        assertThat(interrupted).isTrue();
        assertThat(b.size()).isZero();
    }

    @Test
    void shouldDeliverEveryItemExactlyOnceWithManyProducersAndConsumers() throws Exception {
        int producers = 4;
        int consumers = 4;
        int perProducer = 10_000;
        BlockingBuffer<Integer> b = buffer(16);
        List<List<Integer>> received = new ArrayList<>();
        List<Thread> consumerThreads = new ArrayList<>();
        for (int c = 0; c < consumers; c++) {
            List<Integer> mine = new ArrayList<>();
            received.add(mine);
            consumerThreads.add(start(() -> {
                Optional<Integer> item;
                while ((item = b.take()).isPresent()) {
                    mine.add(item.get());
                }
            }));
        }
        List<Thread> producerThreads = new ArrayList<>();
        AtomicInteger maxSize = new AtomicInteger();
        for (int p = 0; p < producers; p++) {
            int base = p * perProducer;
            producerThreads.add(start(() -> {
                for (int i = 0; i < perProducer; i++) {
                    b.put(base + i);
                    maxSize.accumulateAndGet(b.size(), Math::max);
                }
            }));
        }

        for (Thread t : producerThreads) {
            t.join(60_000);
            assertThat(t.isAlive()).as("producer finished").isFalse();
        }
        b.close();
        for (Thread t : consumerThreads) {
            t.join(60_000);
            assertThat(t.isAlive()).as("consumer finished").isFalse();
        }

        Set<Integer> all = new HashSet<>();
        int total = 0;
        for (List<Integer> mine : received) {
            total += mine.size();
            all.addAll(mine);
            int[] last = new int[producers];
            java.util.Arrays.fill(last, -1);
            for (int v : mine) {
                int p = v / perProducer;
                assertThat(v).as("per-producer order within one consumer").isGreaterThan(last[p]);
                last[p] = v;
            }
        }
        assertThat(total).isEqualTo(producers * perProducer);
        assertThat(all).hasSize(producers * perProducer);
        assertThat(maxSize.get()).isLessThanOrEqualTo(16);
    }
}
