package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.SearchBackend;
import contract.SearchCache;
import java.time.Clock;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.Callable;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.function.Function;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.Eventually;
import testkit.FakeClock;

class BehaviorTest {

    /** Counting fake backend: records every query, version is bumped by the tests, behaviour is swappable. */
    static final class FakeBackend implements SearchBackend {
        final AtomicLong version = new AtomicLong(1);
        final List<String> queries = new CopyOnWriteArrayList<>();
        volatile Function<String, List<String>> handler = q -> new ArrayList<>(List.of(q + "#1", q + "#2"));

        @Override
        public List<String> search(String normalisedQuery) {
            queries.add(normalisedQuery);
            return handler.apply(normalisedQuery);
        }

        @Override
        public long indexVersion() {
            return version.get();
        }

        int calls() {
            return queries.size();
        }
    }

    /** Lets a backend call block until the test releases it. */
    static final class Gate {
        final CountDownLatch entered = new CountDownLatch(1);
        final CountDownLatch release = new CountDownLatch(1);

        void pass() {
            entered.countDown();
            try {
                if (!release.await(20, TimeUnit.SECONDS)) {
                    throw new IllegalStateException("gate never released");
                }
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                throw new IllegalStateException(e);
            }
        }
    }

    private final FakeBackend backend = new FakeBackend();
    private final FakeClock clock = new FakeClock(1_000_000L);

    private SearchCache cache(int capacity, long ttlMillis) {
        return Candidate.of(() -> Solution.create(backend, capacity, ttlMillis, clock));
    }

    @Test
    void shouldFetchFromTheBackendOnAMiss() {
        SearchCache c = cache(10, 1_000);

        assertThat(c.get("red shoes")).containsExactly("red shoes#1", "red shoes#2");

        assertThat(c.hits()).isEqualTo(0);
        assertThat(c.misses()).isEqualTo(1);
        assertThat(c.backendCalls()).isEqualTo(1);
        assertThat(c.size()).isEqualTo(1);
    }

    @Test
    void shouldServeTheSecondIdenticalQueryFromTheCache() {
        SearchCache c = cache(10, 1_000);
        List<String> first = c.get("red shoes");

        assertThat(c.get("red shoes")).isEqualTo(first);

        assertThat(c.hits()).isEqualTo(1);
        assertThat(c.misses()).isEqualTo(1);
        assertThat(backend.calls()).isEqualTo(1);
    }

    @Test
    void shouldTreatCaseAndWhitespaceVariantsAsTheSameQuery() {
        SearchCache c = cache(10, 1_000);
        c.get("red shoes");

        assertThat(c.get("Red  SHOES")).containsExactly("red shoes#1", "red shoes#2");
        assertThat(c.get("  red shoes\t")).containsExactly("red shoes#1", "red shoes#2");
        assertThat(c.get("RED\n \t shoes")).containsExactly("red shoes#1", "red shoes#2");

        assertThat(backend.calls()).isEqualTo(1);
        assertThat(c.hits()).isEqualTo(3);
        assertThat(c.size()).isEqualTo(1);
    }

    @Test
    void shouldGiveTheBackendTheNormalisedQuery() {
        SearchCache c = cache(10, 1_000);

        c.get("  Red   SHOES ");

        assertThat(backend.queries).containsExactly("red shoes");
    }

    @Test
    void shouldTreatWordOrderAsPartOfTheQuery() {
        SearchCache c = cache(10, 1_000);

        c.get("red shoes");
        c.get("shoes red");

        assertThat(backend.queries).containsExactly("red shoes", "shoes red");
        assertThat(c.size()).isEqualTo(2);
    }

    @Test
    void shouldExpireAnEntryExactlyAtItsTtl() {
        SearchCache c = cache(10, 1_000);
        c.get("q");

        clock.advance(999);
        c.get("q");
        assertThat(backend.calls()).isEqualTo(1);

        clock.advance(1);
        c.get("q");
        assertThat(backend.calls()).isEqualTo(2);
        assertThat(c.hits()).isEqualTo(1);
        assertThat(c.misses()).isEqualTo(2);
    }

    @Test
    void shouldNotExtendTheTtlOnHits() {
        SearchCache c = cache(10, 1_000);
        c.get("q");
        clock.advance(600);
        c.get("q");
        clock.advance(400);

        c.get("q");

        assertThat(backend.calls()).isEqualTo(2);
    }

    @Test
    void shouldGiveAReloadedEntryAFullNewTtl() {
        SearchCache c = cache(10, 1_000);
        c.get("q");
        clock.advance(1_000);
        c.get("q");
        clock.advance(999);

        c.get("q");

        assertThat(backend.calls()).isEqualTo(2);
    }

    @Test
    void shouldEvictTheLeastRecentlyUsedEntryWhenOverCapacity() {
        SearchCache c = cache(2, 60_000);
        c.get("a");
        c.get("b");
        c.get("a"); // a is now more recent than b
        c.get("c"); // evicts b

        assertThat(c.size()).isEqualTo(2);
        long before = c.backendCalls();
        c.get("a");
        c.get("c");
        assertThat(c.backendCalls()).isEqualTo(before);
        c.get("b");
        assertThat(c.backendCalls()).isEqualTo(before + 1);
    }

    @Test
    void shouldNeverHoldMoreEntriesThanTheCapacity() {
        SearchCache c = cache(5, 60_000);

        for (int i = 0; i < 100; i++) {
            c.get("query " + i);
            assertThat(c.size()).isLessThanOrEqualTo(5);
        }
        assertThat(c.size()).isEqualTo(5);
    }

    @Test
    void shouldDropEntriesWhenTheIndexVersionChanges() {
        SearchCache c = cache(10, 60_000);
        c.get("a");
        c.get("b");

        backend.version.incrementAndGet();
        assertThat(c.size()).isEqualTo(0);
        c.get("a");

        assertThat(backend.calls()).isEqualTo(3);
        assertThat(c.size()).isEqualTo(1);
        c.get("a");
        assertThat(backend.calls()).isEqualTo(3);
        c.get("b");
        assertThat(backend.calls()).isEqualTo(4);
    }

    @Test
    void shouldReturnFreshResultsAfterTheIndexChanged() {
        SearchCache c = cache(10, 60_000);
        backend.handler = q -> List.of("old");
        assertThat(c.get("q")).containsExactly("old");

        backend.handler = q -> List.of("new");
        assertThat(c.get("q")).containsExactly("old");
        backend.version.incrementAndGet();

        assertThat(c.get("q")).containsExactly("new");
    }

    @Test
    void shouldInvalidateOneQueryByItsNormalisedForm() {
        SearchCache c = cache(10, 60_000);
        c.get("red shoes");
        c.get("blue hat");

        c.invalidate("  RED   Shoes ");

        assertThat(c.size()).isEqualTo(1);
        c.get("blue hat");
        assertThat(backend.calls()).isEqualTo(2);
        c.get("red shoes");
        assertThat(backend.calls()).isEqualTo(3);
    }

    @Test
    void shouldInvalidateEverything() {
        SearchCache c = cache(10, 60_000);
        c.get("a");
        c.get("b");

        c.invalidateAll();

        assertThat(c.size()).isEqualTo(0);
        c.get("a");
        c.get("b");
        assertThat(backend.calls()).isEqualTo(4);
    }

    @Test
    void shouldAcceptInvalidateOfAQueryThatIsNotCached() {
        SearchCache c = cache(10, 60_000);

        c.invalidate("never seen");
        c.invalidateAll();

        assertThat(c.size()).isEqualTo(0);
        assertThat(backend.calls()).isEqualTo(0);
    }

    @Test
    void shouldNotCountExpiredEntriesInTheSize() {
        SearchCache c = cache(10, 1_000);
        c.get("a");
        clock.advance(600);
        c.get("b");
        assertThat(c.size()).isEqualTo(2);

        clock.advance(400);

        assertThat(c.size()).isEqualTo(1);
    }

    @Test
    void shouldRejectNullAndBlankQueriesWithoutCountingThem() {
        SearchCache c = cache(10, 1_000);

        assertThatThrownBy(() -> c.get(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.get("")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.get(" \t\n ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.invalidate(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.invalidate("  ")).isInstanceOf(IllegalArgumentException.class);

        assertThat(c.hits() + c.misses() + c.backendCalls()).isEqualTo(0);
        assertThat(backend.calls()).isEqualTo(0);
    }

    @Test
    void shouldRejectInvalidConfiguration() {
        cache(10, 1_000); // proves the factory is wired

        assertThatThrownBy(() -> Solution.create(null, 10, 1_000, clock)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(backend, 10, 1_000, (Clock) null))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(backend, 0, 1_000, clock)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(backend, -1, 1_000, clock))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(backend, 10, 0, clock)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(backend, 10, -5, clock)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldReturnUnmodifiableLists() {
        SearchCache c = cache(10, 1_000);
        List<String> miss = c.get("q");
        List<String> hit = c.get("q");

        assertThatThrownBy(() -> miss.add("x")).isInstanceOf(UnsupportedOperationException.class);
        assertThatThrownBy(() -> hit.set(0, "x")).isInstanceOf(UnsupportedOperationException.class);
        assertThatThrownBy(hit::clear).isInstanceOf(UnsupportedOperationException.class);
        assertThat(c.get("q")).containsExactly("q#1", "q#2");
    }

    @Test
    void shouldNotBeAffectedWhenTheBackendMutatesItsListLater() {
        List<String> backendList = new ArrayList<>(List.of("d1", "d2"));
        backend.handler = q -> backendList;
        SearchCache c = cache(10, 1_000);
        List<String> first = c.get("q");

        backendList.add("d3");
        backendList.set(0, "changed");

        assertThat(first).containsExactly("d1", "d2");
        assertThat(c.get("q")).containsExactly("d1", "d2");
    }

    @Test
    void shouldCacheAnEmptyResult() {
        backend.handler = q -> new ArrayList<>();
        SearchCache c = cache(10, 1_000);

        assertThat(c.get("nothing matches")).isEmpty();
        assertThat(c.get("nothing matches")).isEmpty();

        assertThat(backend.calls()).isEqualTo(1);
        assertThat(c.hits()).isEqualTo(1);
    }

    @Test
    void shouldPropagateABackendFailureAndNotCacheIt() {
        AtomicBoolean fail = new AtomicBoolean(true);
        backend.handler = q -> {
            if (fail.get()) {
                throw new IllegalStateException("backend down");
            }
            return List.of("ok");
        };
        SearchCache c = cache(10, 1_000);

        assertThatThrownBy(() -> c.get("q")).isInstanceOf(IllegalStateException.class).hasMessage("backend down");
        assertThat(c.size()).isEqualTo(0);
        assertThat(c.misses()).isEqualTo(1);
        assertThat(c.backendCalls()).isEqualTo(1);

        assertThatThrownBy(() -> c.get("q")).isInstanceOf(IllegalStateException.class);
        assertThat(c.backendCalls()).isEqualTo(2);

        fail.set(false);
        assertThat(c.get("q")).containsExactly("ok");
        assertThat(c.get("q")).containsExactly("ok");
        assertThat(c.backendCalls()).isEqualTo(3);
        assertThat(c.hits()).isEqualTo(1);
    }

    @Test
    void shouldCallTheBackendOnceForConcurrentIdenticalQueries() throws Exception {
        Gate gate = new Gate();
        backend.handler = q -> {
            gate.pass();
            return List.of("d1", "d2");
        };
        SearchCache c = cache(10, 60_000);
        String[] spellings = {"red shoes", "Red Shoes", "  red   SHOES "};
        ExecutorService pool = Executors.newFixedThreadPool(8);
        try {
            List<Future<List<String>>> futures = new ArrayList<>();
            for (int i = 0; i < 8; i++) {
                final String spelling = spellings[i % spellings.length];
                futures.add(pool.submit(() -> c.get(spelling)));
            }
            assertThat(gate.entered.await(10, TimeUnit.SECONDS)).isTrue();
            Eventually.assertTrue(() -> c.misses() == 8, 10_000, "all 8 callers reached the cache");
            assertThat(backend.calls()).isEqualTo(1);

            gate.release.countDown();
            for (Future<List<String>> f : futures) {
                assertThat(f.get(10, TimeUnit.SECONDS)).containsExactly("d1", "d2");
            }
        } finally {
            gate.release.countDown();
            pool.shutdownNow();
        }

        assertThat(c.backendCalls()).isEqualTo(1);
        assertThat(c.misses()).isEqualTo(8);
        assertThat(c.hits()).isEqualTo(0);
        assertThat(c.size()).isEqualTo(1);
        c.get("red shoes");
        assertThat(c.hits()).isEqualTo(1);
    }

    @Test
    void shouldPropagateOneFailureToEveryWaiterAndRetryOnTheNextCall() throws Exception {
        Gate gate = new Gate();
        AtomicBoolean fail = new AtomicBoolean(true);
        backend.handler = q -> {
            gate.pass();
            if (fail.get()) {
                throw new IllegalStateException("boom");
            }
            return List.of("ok");
        };
        SearchCache c = cache(10, 60_000);
        ExecutorService pool = Executors.newFixedThreadPool(6);
        try {
            List<Future<List<String>>> futures = new ArrayList<>();
            for (int i = 0; i < 6; i++) {
                futures.add(pool.submit(() -> c.get("q")));
            }
            assertThat(gate.entered.await(10, TimeUnit.SECONDS)).isTrue();
            Eventually.assertTrue(() -> c.misses() == 6, 10_000, "all 6 callers reached the cache");
            gate.release.countDown();

            for (Future<List<String>> f : futures) {
                assertThatThrownBy(() -> f.get(10, TimeUnit.SECONDS))
                        .isInstanceOf(ExecutionException.class)
                        .hasCauseInstanceOf(IllegalStateException.class)
                        .hasRootCauseMessage("boom");
            }
        } finally {
            gate.release.countDown();
            pool.shutdownNow();
        }
        assertThat(backend.calls()).isEqualTo(1);
        assertThat(c.size()).isEqualTo(0);

        fail.set(false);
        assertThat(c.get("q")).containsExactly("ok");
        assertThat(backend.calls()).isEqualTo(2);
    }

    @Test
    void shouldNotBlockOtherQueriesWhileOneBackendCallIsSlow() throws Exception {
        Gate gate = new Gate();
        backend.handler = q -> {
            if (q.equals("slow")) {
                gate.pass();
            }
            return List.of(q);
        };
        SearchCache c = cache(10, 60_000);
        c.get("cached");
        ExecutorService pool = Executors.newFixedThreadPool(3);
        try {
            Future<List<String>> slow = pool.submit(() -> c.get("slow"));
            assertThat(gate.entered.await(10, TimeUnit.SECONDS)).isTrue();

            Future<List<String>> fast = pool.submit(() -> c.get("fast"));
            Future<List<String>> cached = pool.submit(() -> c.get("cached"));
            assertThat(fast.get(10, TimeUnit.SECONDS)).containsExactly("fast");
            assertThat(cached.get(10, TimeUnit.SECONDS)).containsExactly("cached");
            assertThat(slow.isDone()).isFalse();

            gate.release.countDown();
            assertThat(slow.get(10, TimeUnit.SECONDS)).containsExactly("slow");
        } finally {
            gate.release.countDown();
            pool.shutdownNow();
        }
    }

    @Test
    void shouldStayConsistentUnderMixedConcurrentLoad() {
        backend.handler = q -> List.of(q + "#doc");
        SearchCache c = cache(10, 50);
        int threads = 8;
        int perThread = 4_000;
        AtomicInteger gets = new AtomicInteger();

        Concurrently.run(threads, 60_000, t -> {
            for (int i = 0; i < perThread; i++) {
                if (t == 0 && i % 100 == 0) {
                    backend.version.incrementAndGet();
                }
                if (t == 1 && i % 50 == 0) {
                    clock.advance(10);
                }
                if (t == 2 && i % 700 == 0) {
                    c.invalidateAll();
                }
                String key = "q" + ((i * 7 + t) % 30);
                String spelling = (i % 2 == 0) ? key : "  " + key.toUpperCase() + " ";
                List<String> r = c.get(spelling);
                gets.incrementAndGet();
                assertThat(r).containsExactly(key + "#doc");
                assertThat(c.size()).isLessThanOrEqualTo(10);
            }
        });

        assertThat(c.hits() + c.misses()).isEqualTo(gets.get());
        assertThat(c.backendCalls()).isLessThanOrEqualTo(c.misses());
        assertThat(c.backendCalls()).isEqualTo(backend.calls());
        assertThat(c.size()).isLessThanOrEqualTo(10);
    }
}
