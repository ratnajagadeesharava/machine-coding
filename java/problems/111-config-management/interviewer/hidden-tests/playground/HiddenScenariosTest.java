package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.ConfigService;
import java.util.ArrayList;
import java.util.List;
import java.util.NoSuchElementException;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class HiddenScenariosTest {

    private final FakeClock clock = new FakeClock(10_000L);

    private ConfigService config() {
        return Candidate.of(() -> Solution.create(clock));
    }

    private static String view(String ns, String key, String value, long version, long updatedAt) {
        return ns + "/" + key + "=" + value + " v" + version + " t" + updatedAt;
    }

    private static long versionOf(String view) {
        int at = view.lastIndexOf(" v");
        return Long.parseLong(view.substring(at + 2, view.indexOf(' ', at + 2)));
    }

    @Test
    void shouldReportInvalidInputBeforeAVersionMismatchOrAMissingConfig() {
        ConfigService c = config();
        c.create("app", "a", "1");

        assertThatThrownBy(() -> c.update("app", "a", "x".repeat(257), 99)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.update("app", "Bad Key", "x", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.update("app", "missing", "x".repeat(257), 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.update(" ", "missing", "x", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThat(c.history("app", "a")).hasSize(1);
    }

    @Test
    void shouldKeepTheTimestampsOfOldVersionsWhenRollingBackLater() {
        ConfigService c = config();
        c.create("app", "a", "one");
        clock.advance(100);
        c.update("app", "a", "two", 1);
        clock.advance(100);

        assertThat(c.rollback("app", "a", 1)).isEqualTo(3);
        clock.advance(100);
        assertThat(c.rollback("app", "a", 2)).isEqualTo(4);
        assertThat(c.rollback("app", "a", 3)).isEqualTo(5);

        assertThat(c.history("app", "a"))
                .containsExactly(
                        view("app", "a", "one", 1, 10_000),
                        view("app", "a", "two", 2, 10_100),
                        view("app", "a", "one", 3, 10_200),
                        view("app", "a", "two", 4, 10_300),
                        view("app", "a", "one", 5, 10_300));
    }

    @Test
    void shouldRoundTripValuesThatLookLikeTheViewFormat() {
        ConfigService c = config();
        String[] values = {"a=b", "x v9 t9", "line1\nline2", "päth/with space ✓", " leading and trailing ", "/", "a/b=c v1 t2"};
        for (int i = 0; i < values.length; i++) {
            c.create("weird ns/with=chars", "k" + i, values[i]);
        }
        for (int i = 0; i < values.length; i++) {
            assertThat(c.get("weird ns/with=chars", "k" + i))
                    .contains(view("weird ns/with=chars", "k" + i, values[i], 1, 10_000));
        }
        assertThat(c.list("weird ns/with=chars")).hasSize(values.length);
    }

    @Test
    void shouldKeepVersionsContiguousWhenUpdatesAndRollbacksRaceOnOneKey() {
        ConfigService c = config();
        c.create("app", "hot", "seed");
        AtomicInteger updates = new AtomicInteger();
        AtomicInteger rollbacks = new AtomicInteger();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 300; i++) {
                if (t % 2 == 0) {
                    List<String> h = c.history("app", "hot");
                    long v = versionOf(h.get(h.size() - 1));
                    if (c.update("app", "hot", "u" + t + "-" + i, v)) {
                        updates.incrementAndGet();
                    }
                } else {
                    List<String> h = c.history("app", "hot");
                    long to = 1 + (i % h.size());
                    c.rollback("app", "hot", to);
                    rollbacks.incrementAndGet();
                }
            }
        });

        List<String> history = c.history("app", "hot");
        assertThat(history).hasSize(1 + updates.get() + rollbacks.get());
        for (int i = 0; i < history.size(); i++) {
            assertThat(versionOf(history.get(i))).isEqualTo(i + 1);
        }
        assertThat(c.get("app", "hot")).contains(history.get(history.size() - 1));
    }

    @Test
    void shouldAlwaysListASortedConsistentSnapshotWhileConfigsAreBeingAdded() {
        ConfigService c = config();
        int writers = 4;
        int perWriter = 250;

        Concurrently.run(writers + 2, 60_000, t -> {
            if (t < writers) {
                for (int i = 0; i < perWriter; i++) {
                    c.create("big", String.format("k%d-%04d", t, i), "v");
                }
                return;
            }
            for (int r = 0; r < 200; r++) {
                List<String> seen = c.list("big");
                List<String> sorted = new ArrayList<>(seen);
                sorted.sort(null);
                assertThat(seen).isEqualTo(sorted);
            }
        });

        assertThat(c.list("big")).hasSize(writers * perWriter);
    }

    @Test
    void shouldHandleDeleteAndRecreateRacingWithoutBreakingVersionRules() {
        ConfigService c = config();
        c.create("app", "flip", "0");

        Concurrently.run(6, t -> {
            for (int i = 0; i < 400; i++) {
                switch (t % 3) {
                    case 0 -> c.delete("app", "flip");
                    case 1 -> {
                        try {
                            c.create("app", "flip", "c");
                        } catch (IllegalStateException expected) {
                            // someone else created it first
                        }
                    }
                    default -> {
                        try {
                            c.update("app", "flip", "u", 1);
                        } catch (NoSuchElementException expected) {
                            // deleted in between
                        }
                    }
                }
            }
        });

        List<String> history = c.history("app", "flip");
        for (int i = 0; i < history.size(); i++) {
            assertThat(versionOf(history.get(i))).isEqualTo(i + 1);
        }
        assertThat(c.get("app", "flip").isPresent()).isEqualTo(!history.isEmpty());
    }

    @Test
    void shouldKeepGetCheapAfterLongHistories() {
        ConfigService c = config();
        c.create("app", "long", "0");
        for (int i = 1; i <= 5_000; i++) {
            assertThat(c.update("app", "long", "v" + i, i)).isTrue();
        }

        Concurrently.run(1, 20_000, t -> {
            for (int i = 0; i < 20_000; i++) {
                c.get("app", "long");
            }
        });

        assertThat(c.get("app", "long")).contains(view("app", "long", "v5000", 5_001, 10_000));
        assertThat(c.history("app", "long")).hasSize(5_001);
        assertThat(c.rollback("app", "long", 2_500)).isEqualTo(5_002);
        assertThat(c.get("app", "long")).contains(view("app", "long", "v2499", 5_002, 10_000));
    }
}
