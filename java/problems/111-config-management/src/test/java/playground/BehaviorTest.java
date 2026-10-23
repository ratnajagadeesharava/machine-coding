package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.ConfigService;
import java.util.List;
import java.util.NoSuchElementException;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {

    private static final long START = 1_000L;

    private final FakeClock clock = new FakeClock(START);

    private ConfigService config() {
        return Candidate.of(() -> Solution.create(clock));
    }

    private static String view(String ns, String key, String value, long version, long updatedAt) {
        return ns + "/" + key + "=" + value + " v" + version + " t" + updatedAt;
    }

    @Test
    void shouldCreateAConfigAtVersionOneStampedWithTheClock() {
        ConfigService c = config();
        c.create("app/web", "db.host", "localhost");

        assertThat(c.get("app/web", "db.host")).contains(view("app/web", "db.host", "localhost", 1, START));
    }

    @Test
    void shouldReturnEmptyForAConfigThatDoesNotExist() {
        ConfigService c = config();
        c.create("app", "a", "1");

        assertThat(c.get("app", "b")).isEqualTo(Optional.empty());
        assertThat(c.get("other", "a")).isEmpty();
    }

    @Test
    void shouldRejectCreatingADuplicateAndKeepTheOriginal() {
        ConfigService c = config();
        c.create("app", "a", "1");
        clock.advance(10);

        assertThatThrownBy(() -> c.create("app", "a", "2")).isInstanceOf(IllegalStateException.class);
        assertThat(c.get("app", "a")).contains(view("app", "a", "1", 1, START));
        assertThat(c.history("app", "a")).hasSize(1);
    }

    @Test
    void shouldUpdateWhenTheExpectedVersionMatches() {
        ConfigService c = config();
        c.create("app", "a", "1");
        clock.advance(500);

        assertThat(c.update("app", "a", "2", 1)).isTrue();
        assertThat(c.get("app", "a")).contains(view("app", "a", "2", 2, START + 500));
        assertThat(c.update("app", "a", "3", 2)).isTrue();
        assertThat(c.get("app", "a")).contains(view("app", "a", "3", 3, START + 500));
    }

    @Test
    void shouldReturnFalseAndChangeNothingWhenTheExpectedVersionIsStale() {
        ConfigService c = config();
        c.create("app", "a", "1");
        assertThat(c.update("app", "a", "2", 1)).isTrue();
        clock.advance(100);

        assertThat(c.update("app", "a", "stale", 1)).isFalse();
        assertThat(c.update("app", "a", "future", 3)).isFalse();
        assertThat(c.update("app", "a", "zero", 0)).isFalse();
        assertThat(c.get("app", "a")).contains(view("app", "a", "2", 2, START));
        assertThat(c.history("app", "a")).hasSize(2);
    }

    @Test
    void shouldBumpTheVersionOnEverySuccessfulUpdateEvenWhenTheValueIsUnchanged() {
        ConfigService c = config();
        c.create("app", "a", "same");

        assertThat(c.update("app", "a", "same", 1)).isTrue();
        assertThat(c.update("app", "a", "same", 2)).isTrue();

        assertThat(c.get("app", "a")).contains(view("app", "a", "same", 3, START));
        assertThat(c.history("app", "a")).hasSize(3);
    }

    @Test
    void shouldFailToUpdateAConfigThatDoesNotExist() {
        ConfigService c = config();

        assertThatThrownBy(() -> c.update("app", "missing", "x", 1)).isInstanceOf(NoSuchElementException.class);
        assertThat(c.get("app", "missing")).isEmpty();
    }

    @Test
    void shouldListANamespaceSortedByKey() {
        ConfigService c = config();
        c.create("app", "zeta", "z");
        c.create("app", "alpha", "a");
        c.create("app", "mid.key", "m");
        c.create("app", "mid-key", "d");
        c.create("app", "1st", "one");
        c.update("app", "alpha", "a2", 1);

        assertThat(c.list("app"))
                .containsExactly(
                        view("app", "1st", "one", 1, START),
                        view("app", "alpha", "a2", 2, START),
                        view("app", "mid-key", "d", 1, START),
                        view("app", "mid.key", "m", 1, START),
                        view("app", "zeta", "z", 1, START));
    }

    @Test
    void shouldKeepNamespacesIsolatedFromEachOther() {
        ConfigService c = config();
        c.create("app/web", "timeout", "30");
        c.create("app/api", "timeout", "60");
        c.create("app/api", "retries", "3");

        assertThat(c.get("app/web", "timeout")).contains(view("app/web", "timeout", "30", 1, START));
        assertThat(c.list("app/web")).hasSize(1);
        assertThat(c.list("app/api")).hasSize(2);
        assertThat(c.list("app")).isEmpty();
        assertThat(c.update("app/api", "timeout", "90", 1)).isTrue();
        assertThat(c.get("app/web", "timeout")).contains(view("app/web", "timeout", "30", 1, START));
    }

    @Test
    void shouldTreatNamespacesAsCaseSensitive() {
        ConfigService c = config();
        c.create("App", "a", "upper");
        c.create("app", "a", "lower");

        assertThat(c.get("App", "a")).contains(view("App", "a", "upper", 1, START));
        assertThat(c.get("app", "a")).contains(view("app", "a", "lower", 1, START));
    }

    @Test
    void shouldReturnAnEmptyListForAnUnknownNamespace() {
        ConfigService c = config();

        assertThat(c.list("nobody")).isEmpty();
        assertThat(c.history("nobody", "a")).isEmpty();
    }

    @Test
    void shouldReturnHistoryOldestFirstIncludingTheCurrentVersion() {
        ConfigService c = config();
        c.create("app", "a", "one");
        clock.advance(10);
        c.update("app", "a", "two", 1);
        clock.advance(10);
        c.update("app", "a", "three", 2);

        assertThat(c.history("app", "a"))
                .containsExactly(
                        view("app", "a", "one", 1, START),
                        view("app", "a", "two", 2, START + 10),
                        view("app", "a", "three", 3, START + 20));
    }

    @Test
    void shouldReturnAHistorySnapshotThatLaterChangesDoNotAlter() {
        ConfigService c = config();
        c.create("app", "a", "one");
        List<String> before = c.history("app", "a");

        c.update("app", "a", "two", 1);

        assertThat(before).containsExactly(view("app", "a", "one", 1, START));
        assertThat(c.history("app", "a")).hasSize(2);
    }

    @Test
    void shouldRollbackByCreatingANewVersionWithTheOldValue() {
        ConfigService c = config();
        c.create("app", "a", "one");
        c.update("app", "a", "two", 1);
        c.update("app", "a", "three", 2);
        clock.advance(250);

        long version = c.rollback("app", "a", 1);

        assertThat(version).isEqualTo(4);
        assertThat(c.get("app", "a")).contains(view("app", "a", "one", 4, START + 250));
        assertThat(c.history("app", "a"))
                .containsExactly(
                        view("app", "a", "one", 1, START),
                        view("app", "a", "two", 2, START),
                        view("app", "a", "three", 3, START),
                        view("app", "a", "one", 4, START + 250));
    }

    @Test
    void shouldAllowRollingBackToTheCurrentVersion() {
        ConfigService c = config();
        c.create("app", "a", "one");

        assertThat(c.rollback("app", "a", 1)).isEqualTo(2);
        assertThat(c.get("app", "a")).contains(view("app", "a", "one", 2, START));
    }

    @Test
    void shouldRejectRollbackToAVersionThatNeverExistedOrOfAMissingConfig() {
        ConfigService c = config();
        c.create("app", "a", "one");
        c.update("app", "a", "two", 1);

        assertThatThrownBy(() -> c.rollback("app", "a", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.rollback("app", "a", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.rollback("app", "a", 3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.rollback("app", "nope", 1)).isInstanceOf(NoSuchElementException.class);
        assertThat(c.get("app", "a")).contains(view("app", "a", "two", 2, START));
        assertThat(c.history("app", "a")).hasSize(2);
    }

    @Test
    void shouldDeleteAConfigAndReportWhetherItExisted() {
        ConfigService c = config();
        c.create("app", "a", "1");
        c.create("app", "b", "2");

        assertThat(c.delete("app", "a")).isTrue();
        assertThat(c.delete("app", "a")).isFalse();
        assertThat(c.delete("app", "never")).isFalse();
        assertThat(c.delete("nobody", "a")).isFalse();
        assertThat(c.get("app", "a")).isEmpty();
        assertThat(c.history("app", "a")).isEmpty();
        assertThat(c.list("app")).containsExactly(view("app", "b", "2", 1, START));
        assertThatThrownBy(() -> c.update("app", "a", "x", 1)).isInstanceOf(NoSuchElementException.class);
    }

    @Test
    void shouldStartOverAtVersionOneWithFreshHistoryWhenRecreatedAfterDelete() {
        ConfigService c = config();
        c.create("app", "a", "old");
        c.update("app", "a", "older", 1);
        c.delete("app", "a");
        clock.advance(70);

        c.create("app", "a", "new");

        assertThat(c.get("app", "a")).contains(view("app", "a", "new", 1, START + 70));
        assertThat(c.history("app", "a")).containsExactly(view("app", "a", "new", 1, START + 70));
        assertThat(c.update("app", "a", "x", 1)).isTrue();
    }

    @Test
    void shouldRejectBlankNamespacesAndBlankKeysOnEveryOperation() {
        ConfigService c = config();
        c.create("app", "a", "1");

        for (String ns : new String[] {null, "", "   ", "\t"}) {
            assertThatThrownBy(() -> c.create(ns, "a", "v")).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> c.update(ns, "a", "v", 1)).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> c.get(ns, "a")).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> c.delete(ns, "a")).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> c.list(ns)).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> c.history(ns, "a")).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> c.rollback(ns, "a", 1)).isInstanceOf(IllegalArgumentException.class);
        }
        for (String key : new String[] {null, "", "  "}) {
            assertThatThrownBy(() -> c.create("app", key, "v")).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> c.update("app", key, "v", 1)).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> c.get("app", key)).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> c.delete("app", key)).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> c.history("app", key)).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> c.rollback("app", key, 1)).isInstanceOf(IllegalArgumentException.class);
        }
        assertThat(c.list("app")).hasSize(1);
    }

    @Test
    void shouldRejectKeysWithCharactersOutsideLowercaseAlphanumericDotUnderscoreDash() {
        ConfigService c = config();

        for (String bad : new String[] {"Upper", "has space", "slash/key", "colon:key", "eq=ual", "ünï", "tab\tkey", "a,b"}) {
            assertThatThrownBy(() -> c.create("app", bad, "v")).isInstanceOf(IllegalArgumentException.class);
        }
        for (String ok : new String[] {"a", "0", "a.b", "a_b", "a-b", "_", ".", "-", "x1.y2_z3-w4"}) {
            c.create("app", ok, "v");
        }
        assertThat(c.list("app")).hasSize(9);
    }

    @Test
    void shouldAcceptValuesUpTo256CharactersAndRejectLongerOnesOrNull() {
        ConfigService c = config();
        String max = "v".repeat(256);

        c.create("app", "max", max);
        c.create("app", "empty", "");
        assertThat(c.get("app", "max")).contains(view("app", "max", max, 1, START));
        assertThat(c.get("app", "empty")).contains(view("app", "empty", "", 1, START));

        assertThatThrownBy(() -> c.create("app", "long", "v".repeat(257))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.create("app", "nul", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.update("app", "max", "v".repeat(257), 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.update("app", "max", null, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThat(c.get("app", "long")).isEmpty();
        assertThat(c.get("app", "max")).contains(view("app", "max", max, 1, START));
    }

    @Test
    void shouldNotChangeAnythingWhenAnUpdateFailsValidation() {
        ConfigService c = config();
        c.create("app", "a", "good");
        clock.advance(5);

        assertThatThrownBy(() -> c.update("app", "a", "x".repeat(300), 1)).isInstanceOf(IllegalArgumentException.class);

        assertThat(c.get("app", "a")).contains(view("app", "a", "good", 1, START));
        assertThat(c.history("app", "a")).hasSize(1);
        assertThat(c.update("app", "a", "next", 1)).isTrue();
    }

    @Test
    void shouldStampEachVersionWithTheClockAtTheMomentItWasWritten() {
        ConfigService c = config();
        c.create("app", "a", "1");
        clock.set(5_000);
        c.update("app", "a", "2", 1);
        clock.set(9_000);

        assertThat(c.get("app", "a")).contains(view("app", "a", "2", 2, 5_000));
        assertThat(c.history("app", "a").get(0)).isEqualTo(view("app", "a", "1", 1, START));
    }

    @Test
    void shouldLetExactlyOneOfManyThreadsCreateTheSameConfig() {
        ConfigService c = config();
        AtomicInteger created = new AtomicInteger();
        AtomicInteger rejected = new AtomicInteger();

        Concurrently.run(16, t -> {
            try {
                c.create("app", "contested", "writer-" + t);
                created.incrementAndGet();
            } catch (IllegalStateException e) {
                rejected.incrementAndGet();
            }
        });

        assertThat(created.get()).isEqualTo(1);
        assertThat(rejected.get()).isEqualTo(15);
        assertThat(c.history("app", "contested")).hasSize(1);
    }

    @Test
    void shouldLetExactlyOneThreadWinWhenAllUseTheSameExpectedVersion() {
        ConfigService c = config();
        c.create("app", "a", "start");

        List<Boolean> results = Concurrently.collect(16, t -> c.update("app", "a", "writer-" + t, 1));

        assertThat(results.stream().filter(r -> r).count()).isEqualTo(1);
        assertThat(c.history("app", "a")).hasSize(2);
        assertThat(c.get("app", "a").orElseThrow()).contains(" v2 ");
    }

    @Test
    void shouldLoseNoUpdateWhenThreadsRetryOnVersionConflicts() {
        ConfigService c = config();
        c.create("app", "counter", "0");
        int threads = 8;
        int perThread = 250;

        Concurrently.run(threads, t -> {
            for (int i = 0; i < perThread; i++) {
                boolean done = false;
                while (!done) {
                    long version = currentVersion(c, "app", "counter");
                    done = c.update("app", "counter", "t" + t + "-" + i, version);
                }
            }
        });

        int expectedVersion = 1 + threads * perThread;
        assertThat(currentVersion(c, "app", "counter")).isEqualTo(expectedVersion);
        List<String> history = c.history("app", "counter");
        assertThat(history).hasSize(expectedVersion);
        for (int i = 0; i < history.size(); i++) {
            assertThat(history.get(i)).contains(" v" + (i + 1) + " ");
        }
    }

    @Test
    void shouldCreateConfigsInDifferentNamespacesConcurrentlyWithoutLosingAny() {
        ConfigService c = config();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 200; i++) {
                c.create("ns-" + (i % 4), "k-" + t + "-" + i, "v");
            }
        });

        int total = 0;
        for (int n = 0; n < 4; n++) {
            total += c.list("ns-" + n).size();
        }
        assertThat(total).isEqualTo(8 * 200);
    }

    private static long currentVersion(ConfigService c, String ns, String key) {
        String v = c.get(ns, key).orElseThrow();
        int at = v.lastIndexOf(" v");
        int end = v.indexOf(' ', at + 2);
        return Long.parseLong(v.substring(at + 2, end));
    }
}
