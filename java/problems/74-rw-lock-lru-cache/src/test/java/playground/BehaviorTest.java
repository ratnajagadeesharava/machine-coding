package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SharedCache;
import java.util.Optional;
import java.util.concurrent.ThreadLocalRandom;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private SharedCache<Integer, String> cache(int capacity) {
        return Candidate.of(() -> Solution.<Integer, String>create(capacity));
    }

    private static String value(int key) {
        return "v" + key;
    }

    /** In a quiet moment: how many keys of the universe are present, read with peek (no side effects). */
    private static int countPresent(SharedCache<Integer, String> c, int universe) {
        int present = 0;
        for (int k = 0; k < universe; k++) {
            Optional<String> v = c.peek(k);
            if (v.isPresent()) {
                assertThat(v.get()).isEqualTo(value(k));
                present++;
            }
        }
        return present;
    }

    @Test
    void shouldEvictTheLeastRecentlyUsedEntry() {
        SharedCache<Integer, String> c = cache(2);
        c.put(1, "a");
        c.put(2, "b");

        c.put(3, "c");

        assertThat(c.peek(1)).isEmpty();
        assertThat(c.peek(2)).hasValue("b");
        assertThat(c.peek(3)).hasValue("c");
        assertThat(c.size()).isEqualTo(2);
    }

    @Test
    void shouldTreatGetAsAUse() {
        SharedCache<Integer, String> c = cache(2);
        c.put(1, "a");
        c.put(2, "b");

        assertThat(c.get(1)).hasValue("a");
        c.put(3, "c");

        assertThat(c.peek(2)).isEmpty();
        assertThat(c.peek(1)).hasValue("a");
    }

    @Test
    void shouldNotTreatPeekAsAUse() {
        SharedCache<Integer, String> c = cache(2);
        c.put(1, "a");
        c.put(2, "b");

        assertThat(c.peek(1)).hasValue("a");
        c.put(3, "c");

        assertThat(c.peek(1)).isEmpty();
        assertThat(c.peek(2)).hasValue("b");
    }

    @Test
    void shouldReplaceAnExistingKeyAndRefreshIt() {
        SharedCache<Integer, String> c = cache(2);
        c.put(1, "a");
        c.put(2, "b");

        c.put(1, "a2");
        c.put(3, "c");

        assertThat(c.peek(1)).hasValue("a2");
        assertThat(c.peek(2)).isEmpty();
        assertThat(c.size()).isEqualTo(2);
    }

    @Test
    void shouldRemoveEntries() {
        SharedCache<Integer, String> c = cache(3);
        c.put(1, "a");

        assertThat(c.remove(1)).isTrue();
        assertThat(c.remove(1)).isFalse();

        assertThat(c.get(1)).isEmpty();
        assertThat(c.size()).isZero();
    }

    @Test
    void shouldWorkWithCapacityOne() {
        SharedCache<Integer, String> c = cache(1);

        c.put(1, "a");
        c.put(2, "b");

        assertThat(c.peek(1)).isEmpty();
        assertThat(c.peek(2)).hasValue("b");
    }

    @Test
    void shouldLoseNoUpdatesWhenManyThreadsInsertDistinctKeys() {
        SharedCache<Integer, String> c = cache(100_000);

        Concurrently.run(8, t -> {
            for (int i = 0; i < 1_000; i++) {
                int key = t * 1_000 + i;
                c.put(key, value(key));
            }
        });

        assertThat(c.size()).isEqualTo(8_000);
        assertThat(countPresent(c, 8_000)).isEqualTo(8_000);
    }

    @Test
    void shouldNeverExceedCapacityAndStayConsistentUnderMixedLoad() {
        int capacity = 50;
        int universe = 500;
        SharedCache<Integer, String> c = cache(capacity);
        AtomicInteger oversize = new AtomicInteger();
        AtomicInteger wrongValues = new AtomicInteger();

        Concurrently.run(12, t -> {
            ThreadLocalRandom rnd = ThreadLocalRandom.current();
            for (int i = 0; i < 20_000; i++) {
                int key = rnd.nextInt(universe);
                switch (rnd.nextInt(10)) {
                    case 0, 1, 2, 3 -> c.put(key, value(key));
                    case 4, 5, 6 -> c.get(key).filter(v -> !v.equals(value(key))).ifPresent(v -> wrongValues.incrementAndGet());
                    case 7, 8 -> c.peek(key).filter(v -> !v.equals(value(key))).ifPresent(v -> wrongValues.incrementAndGet());
                    default -> c.remove(key);
                }
                if (c.size() > capacity) {
                    oversize.incrementAndGet();
                }
            }
        });

        assertThat(oversize).as("size() above capacity").hasValue(0);
        assertThat(wrongValues).as("a key returned another key's value").hasValue(0);
        assertThat(countPresent(c, universe)).as("size() disagrees with what is actually stored").isEqualTo(c.size());
    }

    @Test
    void shouldStayFullAfterManyConcurrentInsertsOfNewKeys() {
        int capacity = 50;
        SharedCache<Integer, String> c = cache(capacity);

        Concurrently.run(8, t -> {
            for (int i = 0; i < 5_000; i++) {
                int key = t * 5_000 + i;
                c.put(key, value(key));
            }
        });

        assertThat(c.size()).isEqualTo(capacity);
        assertThat(countPresent(c, 40_000)).isEqualTo(capacity);
    }

    @Test
    void shouldSurviveManyThreadsReadingTheSameKeysAtOnce() {
        int capacity = 100;
        SharedCache<Integer, String> c = cache(capacity);
        for (int k = 0; k < capacity; k++) {
            c.put(k, value(k));
        }
        AtomicInteger misses = new AtomicInteger();

        Concurrently.run(16, t -> {
            ThreadLocalRandom rnd = ThreadLocalRandom.current();
            for (int i = 0; i < 20_000; i++) {
                int key = rnd.nextInt(capacity);
                // get() reorders the usage list even though it "only reads": it must be safe in parallel
                if (c.get(key).isEmpty()) {
                    misses.incrementAndGet();
                }
            }
        });

        assertThat(misses).hasValue(0);
        assertThat(c.size()).isEqualTo(capacity);
    }

    @Test
    void shouldKeepAHotKeyAliveWhileOtherThreadsOnlyRead() {
        SharedCache<Integer, String> c = cache(10);
        c.put(-1, "hot");
        AtomicBoolean writerDone = new AtomicBoolean();
        AtomicInteger hotMisses = new AtomicInteger();

        Concurrently.run(5, t -> {
            if (t == 0) { // the only writer: touches the hot key after every insert
                for (int i = 0; i < 20_000; i++) {
                    c.put(i, value(i));
                    if (c.get(-1).isEmpty()) {
                        hotMisses.incrementAndGet();
                        c.put(-1, "hot");
                    }
                }
                writerDone.set(true);
                return;
            }
            while (!writerDone.get()) {
                c.peek(-1);
                c.get(-1);
            }
        });

        assertThat(hotMisses).as("the most recently used key was evicted").hasValue(0);
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            int capacity = 5 + run % 10;
            int universe = 40;
            SharedCache<Integer, String> c = cache(capacity);
            AtomicInteger oversize = new AtomicInteger();

            Concurrently.run(8, t -> {
                ThreadLocalRandom rnd = ThreadLocalRandom.current();
                for (int i = 0; i < 1_000; i++) {
                    int key = rnd.nextInt(universe);
                    int op = rnd.nextInt(4);
                    if (op == 0) {
                        c.put(key, value(key));
                    } else if (op == 1) {
                        c.remove(key);
                    } else if (op == 2) {
                        c.get(key);
                    } else {
                        c.peek(key);
                    }
                    if (c.size() > capacity) {
                        oversize.incrementAndGet();
                    }
                }
            });

            assertThat(oversize).as("run %d", run).hasValue(0);
            assertThat(countPresent(c, universe)).as("run %d", run).isEqualTo(c.size());
        }
    }
}
