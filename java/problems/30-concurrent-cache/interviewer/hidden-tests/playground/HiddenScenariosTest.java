package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.CacheApi;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.Eventually;
import testkit.FakeClock;

class HiddenScenariosTest {
    private static final long TTL = 1_000;

    private final FakeClock clock = new FakeClock(0);

    private CacheApi cache(int capacity) {
        return Candidate.of(() -> Solution.create(capacity, TTL, clock));
    }

    @Test
    void ttlOfAComputedValueStartsWhenTheLoaderFinishes() {
        CacheApi cache = cache(3);

        cache.getOrCompute("k", key -> {
            clock.advance(600);
            return "v";
        });

        clock.advance(999);
        assertThat(cache.get("k")).hasValue("v");
        clock.advance(1);
        assertThat(cache.get("k")).isEmpty();
    }

    @Test
    void aFailedReloadOfAnExpiredKeyLeavesItAbsent() {
        CacheApi cache = cache(3);
        cache.put("k", "old");
        clock.advance(TTL);

        assertThatThrownBy(() -> cache.getOrCompute("k", key -> {
            throw new IllegalArgumentException("nope");
        })).isInstanceOf(IllegalArgumentException.class).hasMessage("nope");

        assertThat(cache.get("k")).isEmpty();
        assertThat(cache.size()).isZero();
    }

    @Test
    void aComputedValueInsertedIntoAFullCacheEvictsTheLeastRecentlyUsedEntry() {
        CacheApi cache = cache(2);
        cache.put("a", "1");
        cache.put("b", "2");

        cache.getOrCompute("c", key -> "3");

        assertThat(cache.get("a")).isEmpty();
        assertThat(cache.get("b")).hasValue("2");
        assertThat(cache.get("c")).hasValue("3");
    }

    @Test
    void concurrentCallersAfterExpiryShareOneReload() {
        CacheApi cache = cache(5);
        cache.put("k", "old");
        clock.advance(TTL);
        int threads = 8;
        AtomicInteger entered = new AtomicInteger();
        AtomicInteger calls = new AtomicInteger();

        List<String> results = Concurrently.collect(threads, i -> {
            entered.incrementAndGet();
            return cache.getOrCompute("k", key -> {
                calls.incrementAndGet();
                Eventually.await(() -> entered.get() == threads, 5_000);
                return "new";
            });
        });

        assertThat(calls).hasValue(1);
        assertThat(results).containsOnly("new");
        assertThat(cache.get("k")).hasValue("new");
    }

    @Test
    void shouldWorkWithCapacityOne() {
        CacheApi cache = cache(1);

        cache.put("a", "1");
        cache.put("b", "2");

        assertThat(cache.get("a")).isEmpty();
        assertThat(cache.get("b")).hasValue("2");
        assertThat(cache.size()).isEqualTo(1);
        assertThat(cache.getOrCompute("c", key -> "3")).isEqualTo("3");
        assertThat(cache.get("b")).isEmpty();
        assertThat(cache.size()).isEqualTo(1);
    }
}
