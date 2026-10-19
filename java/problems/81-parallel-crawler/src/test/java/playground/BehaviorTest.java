package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Crawler;
import contract.Fetcher;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private Crawler crawler() {
        return Candidate.of(Solution::create);
    }

    /** A fake web: a link graph with optional latency and broken pages. Records how it is being used. */
    static final class Web implements Fetcher {
        final Map<String, List<String>> graph = new HashMap<>();
        final Set<String> broken = new HashSet<>();
        final ConcurrentHashMap<String, AtomicInteger> fetches = new ConcurrentHashMap<>();
        final AtomicInteger totalFetches = new AtomicInteger();
        final AtomicInteger running = new AtomicInteger();
        final AtomicInteger maxRunning = new AtomicInteger();
        long latencyMs;

        Web link(String from, String... to) {
            graph.computeIfAbsent(from, k -> new ArrayList<>()).addAll(List.of(to));
            for (String t : to) {
                graph.computeIfAbsent(t, k -> new ArrayList<>());
            }
            return this;
        }

        @Override
        public List<String> links(String url) {
            fetches.computeIfAbsent(url, k -> new AtomicInteger()).incrementAndGet();
            totalFetches.incrementAndGet();
            maxRunning.accumulateAndGet(running.incrementAndGet(), Math::max);
            try {
                if (latencyMs > 0) {
                    Thread.sleep(latencyMs);
                }
                if (broken.contains(url)) {
                    throw new IllegalStateException("404 " + url);
                }
                List<String> out = graph.get(url);
                return out == null ? List.of() : List.copyOf(out);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                throw new IllegalStateException(e);
            } finally {
                running.decrementAndGet();
            }
        }

        Set<String> reachableFrom(String start) {
            Set<String> seen = new HashSet<>(Set.of(start));
            ArrayDeque<String> todo = new ArrayDeque<>(List.of(start));
            while (!todo.isEmpty()) {
                String u = todo.poll();
                if (broken.contains(u)) {
                    continue;
                }
                for (String v : graph.getOrDefault(u, List.of())) {
                    if (seen.add(v)) {
                        todo.add(v);
                    }
                }
            }
            return seen;
        }

        void assertEachFetchedOnce(Set<String> expected) {
            assertThat(fetches.keySet()).as("fetched pages").isEqualTo(expected);
            fetches.forEach((url, n) -> assertThat(n.get()).as("fetches of %s", url).isEqualTo(1));
        }
    }

    @Test
    void shouldReturnJustTheStartPageWhenItHasNoLinks() {
        Web web = new Web().link("a");

        Set<String> reached = crawler().crawl("a", 4, web);

        assertThat(reached).containsExactly("a");
        web.assertEachFetchedOnce(Set.of("a"));
    }

    @Test
    void shouldFollowCyclesDiamondsSelfLinksAndDuplicateLinks() {
        Web web = new Web()
                .link("a", "b", "c", "b", "a")
                .link("b", "d")
                .link("c", "d", "a")
                .link("d", "a", "e")
                .link("lonely", "a");

        Set<String> reached = crawler().crawl("a", 3, web);

        assertThat(reached).containsExactlyInAnyOrder("a", "b", "c", "d", "e");
        web.assertEachFetchedOnce(Set.of("a", "b", "c", "d", "e"));
    }

    @Test
    void shouldFinishALongChainWhereOnlyOneWorkerIsBusyAtATime() {
        Web web = new Web();
        web.latencyMs = 1;
        int n = 200;
        for (int i = 0; i < n - 1; i++) {
            web.link("p" + i, "p" + (i + 1));
        }

        Set<String> reached = crawler().crawl("p0", 8, web);

        assertThat(reached).hasSize(n);
        web.assertEachFetchedOnce(reached);
    }

    @Test
    void shouldCrawlABinaryTree() {
        Web web = new Web();
        int depth = 10;
        for (int i = 1; i < (1 << depth); i++) {
            web.link("n" + i, "n" + (2 * i), "n" + (2 * i + 1));
        }

        Set<String> reached = crawler().crawl("n1", 8, web);

        assertThat(reached).hasSize((1 << (depth + 1)) - 1);
        web.assertEachFetchedOnce(reached);
    }

    @Test
    void shouldFetchEachPageOnceInAHeavilyLinkedGrid() {
        Web web = new Web();
        int size = 30;
        for (int x = 0; x < size; x++) {
            for (int y = 0; y < size; y++) {
                String me = x + ":" + y;
                web.link(me, ((x + 1) % size) + ":" + y, ((x + size - 1) % size) + ":" + y, x + ":" + ((y + 1) % size), x + ":" + ((y + size - 1) % size));
            }
        }

        Set<String> reached = crawler().crawl("0:0", 16, web);

        assertThat(reached).hasSize(size * size);
        web.assertEachFetchedOnce(reached);
    }

    @Test
    void shouldCountBrokenPagesAsReachedAndKeepGoing() {
        Web web = new Web().link("a", "b", "c").link("b", "d").link("c", "d").link("d", "e").link("x", "y");
        web.broken.add("b");
        web.broken.add("d");

        Set<String> reached = crawler().crawl("a", 4, web);

        // d is reached through b and c; it is broken, so e stays unknown
        assertThat(reached).containsExactlyInAnyOrder("a", "b", "c", "d");
        web.assertEachFetchedOnce(Set.of("a", "b", "c", "d"));
    }

    @Test
    void shouldNeverRunMoreFetchesThanWorkersButUseThem() {
        Web web = new Web();
        web.latencyMs = 10;
        String[] leaves = new String[100];
        for (int i = 0; i < leaves.length; i++) {
            leaves[i] = "leaf" + i;
        }
        web.link("root", leaves);

        Set<String> reached = crawler().crawl("root", 4, web);

        assertThat(reached).hasSize(101);
        assertThat(web.maxRunning.get()).as("more concurrent fetches than workers").isLessThanOrEqualTo(4);
        assertThat(web.maxRunning.get()).as("workers were not used in parallel").isGreaterThanOrEqualTo(2);
    }

    @Test
    void shouldFetchSequentiallyWithOneWorker() {
        Web web = new Web();
        web.latencyMs = 2;
        web.link("root", "a", "b", "c", "d").link("a", "b").link("c", "d");

        Set<String> reached = crawler().crawl("root", 1, web);

        assertThat(reached).hasSize(5);
        assertThat(web.maxRunning).hasValue(1);
    }

    @Test
    void shouldNotFetchAnythingAfterCrawlReturns() throws Exception {
        Web web = new Web();
        web.latencyMs = 3;
        for (int i = 0; i < 60; i++) {
            web.link("r", "c" + i);
            web.link("c" + i, "c" + ((i + 1) % 60));
        }

        crawler().crawl("r", 8, web);
        int fetchesAtReturn = web.totalFetches.get();
        Thread.sleep(150);

        assertThat(web.totalFetches).as("a worker kept fetching after crawl() returned").hasValue(fetchesAtReturn);
    }

    @Test
    void shouldRejectNonPositiveWorkers() {
        Crawler c = crawler();

        assertThatThrownBy(() -> c.crawl("a", 0, new Web().link("a"))).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldAllowSeveralCrawlsAtOnce() {
        Crawler c = crawler();

        Concurrently.run(4, t -> {
            Web web = new Web();
            for (int i = 0; i < 100; i++) {
                web.link("s" + t + "-" + i, "s" + t + "-" + ((i * 7 + 1) % 100), "s" + t + "-" + ((i + 1) % 100));
            }
            Set<String> reached = c.crawl("s" + t + "-0", 4, web);
            assertThat(reached).isEqualTo(web.reachableFrom("s" + t + "-0"));
            web.assertEachFetchedOnce(reached);
        });
    }

    @Test
    void shouldCrawlRandomGraphsCorrectlyOver100Runs() {
        Crawler c = crawler();
        for (int run = 0; run < 100; run++) {
            Random r = new Random(run);
            int nodes = 20 + r.nextInt(200);
            Web web = new Web();
            for (int i = 0; i < nodes; i++) {
                int out = r.nextInt(4);
                for (int k = 0; k < out; k++) {
                    web.link("u" + i, "u" + r.nextInt(nodes));
                }
                web.graph.computeIfAbsent("u" + i, k -> new ArrayList<>());
                if (r.nextInt(15) == 0) {
                    web.broken.add("u" + i);
                }
            }
            Set<String> expected = web.reachableFrom("u0");

            Set<String> reached = c.crawl("u0", 1 + r.nextInt(12), web);

            assertThat(reached).as("run %d", run).isEqualTo(expected);
            web.assertEachFetchedOnce(expected);
        }
    }
}
