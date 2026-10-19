package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Crawler;
import contract.Fetcher;
import java.util.ArrayList;
import java.util.List;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    @Test
    void shouldCrawlAHundredThousandPagesQuickly() {
        Crawler c = Candidate.of(Solution::create);
        int n = 100_000;
        AtomicInteger fetches = new AtomicInteger();
        Fetcher web = url -> {
            fetches.incrementAndGet();
            int i = Integer.parseInt(url.substring(1));
            List<String> out = new ArrayList<>();
            for (int k = 1; k <= 3; k++) {
                int j = (int) (((long) i * 3 + k) % n);
                out.add("p" + j);
            }
            out.add("p" + ((i + 1) % n));
            return out;
        };

        Set<String> reached = c.crawl("p0", 16, web);

        assertThat(reached).hasSize(n);
        assertThat(fetches).hasValue(n);
    }

    @Test
    void shouldKeepCrawlingWhenEveryPageIsSlowAndTheGraphIsDeep() {
        Crawler c = Candidate.of(Solution::create);
        int n = 300;
        ConcurrentHashMap<String, Boolean> seen = new ConcurrentHashMap<>();
        Fetcher web = url -> {
            seen.put(url, true);
            try {
                Thread.sleep(2);
            } catch (InterruptedException e) {
                throw new IllegalStateException(e);
            }
            int i = Integer.parseInt(url.substring(1));
            return i + 1 < n ? List.of("p" + (i + 1)) : List.of();
        };

        Set<String> reached = c.crawl("p0", 32, web);

        assertThat(reached).hasSize(n);
        assertThat(seen).hasSize(n);
    }
}
