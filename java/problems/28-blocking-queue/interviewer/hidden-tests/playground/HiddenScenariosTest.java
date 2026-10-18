package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.BlockingBuffer;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Eventually;

class HiddenScenariosTest {

    private BlockingBuffer<Integer> buffer(int capacity) {
        return Candidate.of(() -> Solution.<Integer>create(capacity));
    }

    private interface ThrowingRunnable {
        void run() throws InterruptedException;
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

    @Test
    void capacityOneShouldHandOverEveryItemInOrder() throws Exception {
        BlockingBuffer<Integer> b = buffer(1);
        int n = 2_000;
        start(() -> {
            for (int i = 0; i < n; i++) {
                b.put(i);
            }
            b.close();
        });
        List<Integer> got = new ArrayList<>();

        Optional<Integer> item;
        while ((item = b.poll(10_000)).isPresent()) {
            got.add(item.get());
        }

        assertThat(got).hasSize(n);
        for (int i = 0; i < n; i++) {
            assertThat(got.get(i)).isEqualTo(i);
        }
    }

    @Test
    void putOnAFullClosedBufferShouldFailImmediately() throws Exception {
        BlockingBuffer<Integer> b = buffer(1);
        b.put(1);
        b.close();
        AtomicReference<Throwable> failure = new AtomicReference<>();

        start(() -> {
            try {
                b.put(2);
            } catch (IllegalStateException e) {
                failure.set(e);
            }
        });

        Eventually.assertTrue(() -> failure.get() != null, 5_000, "put must not wait for space on a closed buffer");
        assertThatThrownBy(() -> b.offer(3, 30_000)).isInstanceOf(IllegalStateException.class);
        assertThat(b.size()).isEqualTo(1);
    }

    @Test
    void severalConsumersDrainingAClosedBufferShouldGetEveryItemExactlyOnce() throws Exception {
        BlockingBuffer<Integer> b = buffer(100);
        for (int i = 0; i < 100; i++) {
            b.put(i);
        }
        b.close();
        List<Integer> got = new CopyOnWriteArrayList<>();
        AtomicInteger finished = new AtomicInteger();
        for (int c = 0; c < 4; c++) {
            start(() -> {
                Optional<Integer> item;
                while ((item = b.take()).isPresent()) {
                    got.add(item.get());
                }
                finished.incrementAndGet();
            });
        }

        Eventually.assertTrue(() -> finished.get() == 4, 10_000, "all consumers see the end");

        Set<Integer> unique = new HashSet<>(got);
        assertThat(got).hasSize(100);
        assertThat(unique).hasSize(100);
    }

    @Test
    void sizeShouldNeverExceedCapacityWhileProducersRace() throws Exception {
        BlockingBuffer<Integer> b = buffer(3);
        AtomicInteger observedMax = new AtomicInteger();
        AtomicInteger produced = new AtomicInteger();
        List<Thread> producers = new ArrayList<>();
        for (int p = 0; p < 6; p++) {
            producers.add(start(() -> {
                for (int i = 0; i < 2_000; i++) {
                    b.put(i);
                    produced.incrementAndGet();
                }
            }));
        }
        int consumed = 0;
        while (consumed < 12_000) {
            observedMax.accumulateAndGet(b.size(), Math::max);
            if (b.poll(5_000).isEmpty()) {
                break;
            }
            consumed++;
        }

        assertThat(consumed).isEqualTo(12_000);
        assertThat(observedMax.get()).isLessThanOrEqualTo(3);
        assertThat(b.size()).isZero();
    }

    @Test
    void closeIsIdempotentAndAnEmptyClosedBufferNeverBlocks() throws Exception {
        BlockingBuffer<Integer> b = buffer(2);

        b.close();
        b.close();

        assertThat(b.take()).isEmpty();
        assertThat(b.poll(30_000)).isEmpty();
        assertThatThrownBy(() -> b.put(1)).isInstanceOf(IllegalStateException.class);
        assertThat(b.size()).isZero();
    }
}
