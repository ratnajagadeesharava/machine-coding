package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.CacheApi;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.Eventually;
import testkit.FakeClock;

class BehaviorTest {
    private static final long TTL = 1_000;

    private final FakeClock clock = new FakeClock(5_000);

    private CacheApi cache(int capacity) {
        return Candidate.of(() -> Solution.create(capacity, TTL, clock));
    }

    @Test
    void shouldReturnEmptyForMissingKey() {
        CacheApi cache = cache(3);

        assertThat(cache.get("nope")).isEmpty();
        assertThat(cache.size()).isZero();
    }

    @Test
    void shouldReturnStoredValueAndOverwrite() {
        CacheApi cache = cache(3);

        cache.put("a", "1");
        assertThat(cache.get("a")).hasValue("1");
        cache.put("a", "2");

        assertThat(cache.get("a")).hasValue("2");
        assertThat(cache.size()).isEqualTo(1);
    }

    @Test
    void shouldExpireExactlyAtTtl() {
        CacheApi cache = cache(3);
        cache.put("a", "1");

        clock.advance(TTL - 1);
        assertThat(cache.get("a")).hasValue("1");

        clock.advance(1);
        assertThat(cache.get("a")).isEmpty();
    }

    @Test
    void shouldNotExtendTtlOnRead() {
        CacheApi cache = cache(3);
        cache.put("a", "1");

        clock.advance(600);
        assertThat(cache.get("a")).hasValue("1");
        clock.advance(400);

        assertThat(cache.get("a")).isEmpty();
    }

    @Test
    void shouldRestartTtlWhenOverwritten() {
        CacheApi cache = cache(3);
        cache.put("a", "1");
        clock.advance(600);
        cache.put("a", "2");

        clock.advance(600); // 1200 after the first put, 600 after the second
        assertThat(cache.get("a")).hasValue("2");
        clock.advance(400);
        assertThat(cache.get("a")).isEmpty();
    }

    @Test
    void shouldCountOnlyLiveEntriesInSize() {
        CacheApi cache = cache(3);
        cache.put("a", "1");
        clock.advance(600);
        cache.put("b", "2");
        assertThat(cache.size()).isEqualTo(2);

        clock.advance(400); // a expired (age 1000), b alive (age 400)
        assertThat(cache.size()).isEqualTo(1);

        clock.advance(600); // b expired
        assertThat(cache.size()).isZero();
    }

    @Test
    void shouldEvictLeastRecentlyUsedWhenFull() {
        CacheApi cache = cache(2);
        cache.put("a", "1");
        cache.put("b", "2");
        cache.get("a");

        cache.put("c", "3");

        assertThat(cache.get("b")).isEmpty();
        assertThat(cache.get("a")).hasValue("1");
        assertThat(cache.get("c")).hasValue("3");
        assertThat(cache.size()).isEqualTo(2);
    }

    @Test
    void shouldNotEvictWhenOverwritingAnExistingKeyAtCapacity() {
        CacheApi cache = cache(2);
        cache.put("a", "1");
        cache.put("b", "2");

        cache.put("a", "9");

        assertThat(cache.get("a")).hasValue("9");
        assertThat(cache.get("b")).hasValue("2");
    }

    @Test
    void shouldDropExpiredEntriesBeforeEvictingLiveOnes() {
        CacheApi cache = cache(2);
        cache.put("a", "1");          // t0
        clock.advance(600);
        cache.put("b", "2");          // t0+600
        cache.get("a");               // a is now the most recently used, b the least
        clock.advance(400);           // a expires (age 1000); b is alive (age 400)

        cache.put("c", "3");

        assertThat(cache.get("b")).hasValue("2");
        assertThat(cache.get("c")).hasValue("3");
        assertThat(cache.size()).isEqualTo(2);
    }

    @Test
    void shouldComputeOnceAndServeLaterCallsFromTheCache() {
        CacheApi cache = cache(3);
        AtomicInteger calls = new AtomicInteger();

        String first = cache.getOrCompute("k", key -> {
            calls.incrementAndGet();
            return "v:" + key;
        });
        String second = cache.getOrCompute("k", key -> {
            calls.incrementAndGet();
            return "other";
        });

        assertThat(first).isEqualTo("v:k");
        assertThat(second).isEqualTo("v:k");
        assertThat(calls).hasValue(1);
        assertThat(cache.get("k")).hasValue("v:k");
    }

    @Test
    void shouldRecomputeAnExpiredValue() {
        CacheApi cache = cache(3);
        cache.put("k", "old");
        clock.advance(TTL);

        assertThat(cache.getOrCompute("k", key -> "new")).isEqualTo("new");
        assertThat(cache.get("k")).hasValue("new");
    }

    @Test
    void shouldCountComputeHitsAsUse() {
        CacheApi cache = cache(2);
        cache.put("a", "1");
        cache.put("b", "2");

        assertThat(cache.getOrCompute("a", key -> {
            throw new AssertionError("loader must not run on a hit");
        })).isEqualTo("1");
        cache.put("c", "3");

        assertThat(cache.get("a")).hasValue("1");
        assertThat(cache.get("b")).isEmpty();
    }

    @Test
    void shouldPropagateLoaderFailureCacheNothingAndRetryNextTime() {
        CacheApi cache = cache(3);
        AtomicInteger calls = new AtomicInteger();

        assertThatThrownBy(() -> cache.getOrCompute("k", key -> {
            calls.incrementAndGet();
            throw new IllegalStateException("boom");
        })).isInstanceOf(IllegalStateException.class).hasMessage("boom");
        assertThat(cache.get("k")).isEmpty();
        assertThat(cache.size()).isZero();

        String value = cache.getOrCompute("k", key -> {
            calls.incrementAndGet();
            return "ok";
        });

        assertThat(value).isEqualTo("ok");
        assertThat(calls).hasValue(2);
    }

    @Test
    void shouldRunTheLoaderOnceForConcurrentCallersOfTheSameKey() {
        CacheApi cache = cache(10);
        int threads = 8;
        AtomicInteger entered = new AtomicInteger();
        AtomicInteger calls = new AtomicInteger();

        List<String> results = Concurrently.collect(threads, i -> {
            entered.incrementAndGet();
            return cache.getOrCompute("k", key -> {
                calls.incrementAndGet();
                Eventually.await(() -> entered.get() == threads, 5_000);
                return "v:" + key;
            });
        });

        assertThat(calls).hasValue(1);
        assertThat(results).hasSize(threads).containsOnly("v:k");
    }

    @Test
    void shouldShareALoaderFailureWithConcurrentCallers() {
        CacheApi cache = cache(10);
        int threads = 6;
        AtomicInteger entered = new AtomicInteger();
        AtomicInteger calls = new AtomicInteger();

        List<String> outcomes = Concurrently.collect(threads, i -> {
            entered.incrementAndGet();
            try {
                return cache.getOrCompute("k", key -> {
                    calls.incrementAndGet();
                    Eventually.await(() -> entered.get() == threads, 5_000);
                    pause(150); // let every caller attach to this computation
                    throw new IllegalStateException("boom");
                });
            } catch (IllegalStateException e) {
                return "failed:" + e.getMessage();
            }
        });

        assertThat(outcomes).hasSize(threads).containsOnly("failed:boom");
        assertThat(calls).hasValue(1);
        assertThat(cache.get("k")).isEmpty();
        assertThat(cache.getOrCompute("k", key -> "recovered")).isEqualTo("recovered");
    }

    @Test
    void shouldComputeDifferentKeysInParallel() {
        CacheApi cache = cache(10);
        CountDownLatch bothRunning = new CountDownLatch(2);

        List<String> results = Concurrently.collect(2, i -> cache.getOrCompute("k" + i, key -> {
            bothRunning.countDown();
            try {
                if (!bothRunning.await(5, TimeUnit.SECONDS)) {
                    throw new IllegalStateException("loaders for different keys did not overlap");
                }
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                throw new IllegalStateException(e);
            }
            return "v:" + key;
        }));

        assertThat(results).containsExactly("v:k0", "v:k1");
    }

    @Test
    void shouldAllowALoaderToUseTheCacheForOtherKeys() {
        CacheApi cache = cache(10);
        cache.put("c", "cv");

        String result = Concurrently.<String>collect(1, i -> cache.getOrCompute("a", key -> {
            String b = cache.getOrCompute("b", inner -> "bv");
            return key + b + cache.get("c").orElse("?");
        })).get(0);

        assertThat(result).isEqualTo("abvcv");
        assertThat(cache.size()).isEqualTo(3);
    }

    @Test
    void shouldStayWithinCapacityUnderMixedConcurrentLoad() {
        CacheApi cache = cache(10);

        Concurrently.run(16, t -> {
            for (int i = 0; i < 500; i++) {
                String key = "k" + (i % 50);
                String expected = "v:" + key;
                switch ((t + i) % 3) {
                    case 0 -> cache.put(key, expected);
                    case 1 -> cache.get(key).ifPresent(v -> assertThat(v).isEqualTo(expected));
                    default -> assertThat(cache.getOrCompute(key, k -> "v:" + k)).isEqualTo(expected);
                }
                assertThat(cache.size()).isBetween(0, 10);
            }
        });

        assertThat(cache.size()).isBetween(1, 10);
        List<String> present = new ArrayList<>();
        for (int i = 0; i < 50; i++) {
            cache.get("k" + i).ifPresent(present::add);
        }
        assertThat(present).hasSize(cache.size());
    }

    private static void pause(long millis) {
        try {
            Thread.sleep(millis);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
}
