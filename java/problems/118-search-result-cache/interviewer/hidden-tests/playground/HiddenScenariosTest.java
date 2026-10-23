package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SearchBackend;
import contract.SearchCache;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import java.util.function.Function;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.Eventually;
import testkit.FakeClock;

class HiddenScenariosTest {

    static final class FakeBackend implements SearchBackend {
        final AtomicLong version = new AtomicLong(1);
        final List<String> queries = new CopyOnWriteArrayList<>();
        volatile Function<String, List<String>> handler = q -> List.of(q + "#1");

        @Override
        public List<String> search(String normalisedQuery) {
            queries.add(normalisedQuery);
            return handler.apply(normalisedQuery);
        }

        @Override
        public long indexVersion() {
            return version.get();
        }
    }

    private final FakeBackend backend = new FakeBackend();
    private final FakeClock clock = new FakeClock(5_000L);

    private SearchCache cache(int capacity, long ttl) {
        return Candidate.of(() -> Solution.create(backend, capacity, ttl, clock));
    }

    @Test
    void shouldTagAnEntryWithTheVersionReadBeforeTheBackendCall() {
        backend.handler = q -> {
            backend.version.incrementAndGet(); // the index changes while the search runs
            return List.of("maybe-stale");
        };
        SearchCache c = cache(10, 60_000);

        assertThat(c.get("q")).containsExactly("maybe-stale");

        assertThat(c.size()).isEqualTo(0);
        backend.handler = q -> List.of("fresh");
        assertThat(c.get("q")).containsExactly("fresh");
        assertThat(c.backendCalls()).isEqualTo(2);
        assertThat(c.get("q")).containsExactly("fresh");
        assertThat(c.backendCalls()).isEqualTo(2);
    }

    @Test
    void shouldNotStoreOrShareACallThatWasInvalidatedWhileInFlight() throws Exception {
        CountDownLatch entered = new CountDownLatch(1);
        CountDownLatch release = new CountDownLatch(1);
        AtomicInteger n = new AtomicInteger();
        backend.handler = q -> {
            if (n.getAndIncrement() == 0) {
                entered.countDown();
                try {
                    release.await(20, TimeUnit.SECONDS);
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                }
                return List.of("stale");
            }
            return List.of("fresh");
        };
        SearchCache c = cache(10, 60_000);
        ExecutorService pool = Executors.newFixedThreadPool(2);
        try {
            Future<List<String>> old = pool.submit(() -> c.get("q"));
            assertThat(entered.await(10, TimeUnit.SECONDS)).isTrue();

            c.invalidate("Q");
            Future<List<String>> after = pool.submit(() -> c.get("q"));
            assertThat(after.get(10, TimeUnit.SECONDS)).containsExactly("fresh");

            release.countDown();
            assertThat(old.get(10, TimeUnit.SECONDS)).containsExactly("stale");
        } finally {
            release.countDown();
            pool.shutdownNow();
        }

        assertThat(c.get("q")).containsExactly("fresh");
        assertThat(c.backendCalls()).isEqualTo(2);
    }

    @Test
    void shouldCallTheBackendExactlyOncePerQueryUnderHeavyOverlap() {
        backend.handler = q -> {
            try {
                Thread.sleep(1);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            return List.of(q + "#1");
        };
        SearchCache c = cache(1_000, 600_000);
        int threads = 32;
        int keys = 500;

        Concurrently.run(threads, 60_000, t -> {
            for (int k = 0; k < keys; k++) {
                String q = "Query  " + k;
                assertThat(c.get(q)).containsExactly("query " + k + "#1");
            }
        });

        assertThat(c.backendCalls()).isEqualTo(keys);
        assertThat(backend.queries).hasSize(keys);
        assertThat(c.hits() + c.misses()).isEqualTo((long) threads * keys);
        assertThat(c.size()).isEqualTo(keys);
    }

    @Test
    void shouldGiveEveryWaiterTheVeryExceptionTheBackendThrew() throws Exception {
        CountDownLatch release = new CountDownLatch(1);
        CountDownLatch entered = new CountDownLatch(1);
        IllegalStateException boom = new IllegalStateException("boom");
        backend.handler = q -> {
            entered.countDown();
            try {
                release.await(20, TimeUnit.SECONDS);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            throw boom;
        };
        SearchCache c = cache(10, 60_000);
        ExecutorService pool = Executors.newFixedThreadPool(4);
        try {
            List<Future<List<String>>> fs = new ArrayList<>();
            for (int i = 0; i < 4; i++) {
                fs.add(pool.submit(() -> c.get("q")));
            }
            assertThat(entered.await(10, TimeUnit.SECONDS)).isTrue();
            Eventually.assertTrue(() -> c.misses() == 4, 10_000, "all callers arrived");
            release.countDown();
            for (Future<List<String>> f : fs) {
                try {
                    f.get(10, TimeUnit.SECONDS);
                    throw new AssertionError("expected a failure");
                } catch (ExecutionException e) {
                    assertThat(e.getCause()).isSameAs(boom);
                }
            }
        } finally {
            release.countDown();
            pool.shutdownNow();
        }
    }

    @Test
    void shouldKeepAFailingQueryFromAffectingOthers() {
        backend.handler = q -> {
            if (q.equals("bad")) {
                throw new IllegalStateException("bad query");
            }
            return List.of(q + "#1");
        };
        SearchCache c = cache(10, 60_000);

        Concurrently.run(8, t -> {
            for (int i = 0; i < 200; i++) {
                try {
                    c.get("bad");
                    throw new AssertionError("should fail");
                } catch (IllegalStateException expected) {
                    // fine
                }
                assertThat(c.get("good " + (i % 5))).containsExactly("good " + (i % 5) + "#1");
            }
        });

        assertThat(c.size()).isEqualTo(5);
        backend.handler = q -> List.of("recovered");
        assertThat(c.get("bad")).containsExactly("recovered");
    }

    @Test
    void shouldOrderEvictionByUseAndNotByAgeOrTtl() {
        SearchCache c = cache(3, 10_000);
        c.get("a");
        c.get("b");
        c.get("c");
        c.get("a");
        c.get("b"); // recency now: c (oldest), a, b
        c.get("d"); // evicts c
        long calls = c.backendCalls();

        c.get("a");
        c.get("b");
        c.get("d");
        assertThat(c.backendCalls()).isEqualTo(calls);

        c.get("c"); // reload of c evicts a (the least recently used)
        assertThat(c.backendCalls()).isEqualTo(calls + 1);
        c.get("b");
        c.get("d");
        c.get("c");
        assertThat(c.backendCalls()).isEqualTo(calls + 1);
        c.get("a");
        assertThat(c.backendCalls()).isEqualTo(calls + 2);
    }

    @Test
    void shouldNeverServeAResultOlderThanTheIndexVersionSeenBeforeTheGet() {
        backend.handler = q -> List.of("v" + backend.version.get());
        SearchCache c = cache(5, 600_000);

        for (int i = 0; i < 1_000; i++) {
            backend.version.incrementAndGet();
            long expected = backend.version.get();
            assertThat(c.get("q" + (i % 3))).containsExactly("v" + expected);
            assertThat(c.get("q" + (i % 3))).containsExactly("v" + expected);
        }
    }

    @Test
    void shouldSurviveATinyCapacityUnderManyDistinctQueries() {
        backend.handler = q -> List.of(q + "#1");
        SearchCache c = cache(1, 1_000);

        Concurrently.run(16, 60_000, t -> {
            for (int i = 0; i < 2_000; i++) {
                String q = "k" + ((i + t) % 40);
                assertThat(c.get(q)).containsExactly(q + "#1");
                assertThat(c.size()).isLessThanOrEqualTo(1);
                if (i % 500 == 0) {
                    clock.advance(300);
                    backend.version.incrementAndGet();
                }
            }
        });

        assertThat(c.hits() + c.misses()).isEqualTo(16L * 2_000);
    }

    @Test
    void shouldLowerCaseNonAsciiQueriesWithoutDependingOnTheDefaultLocale() {
        SearchCache c = cache(10, 60_000);

        c.get("  ÉCOLE   Primaire ");
        c.get("école primaire");

        assertThat(backend.queries).containsExactly("école primaire");
        assertThat(c.hits()).isEqualTo(1);
    }
}
