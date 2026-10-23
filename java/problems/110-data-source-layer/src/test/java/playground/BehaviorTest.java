package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.DataAccess;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private DataAccess layer() {
        return Candidate.of(Solution::create);
    }

    private static String chars(int n) {
        return "x".repeat(n);
    }

    @Test
    void shouldStartOnDatabaseAndEmpty() {
        DataAccess d = layer();

        assertThat(d.currentSource()).isEqualTo("DATABASE");
        assertThat(d.size()).isZero();
        assertThat(d.get("anything")).isEmpty();
    }

    @Test
    void shouldStoreAndReturnAValue() {
        DataAccess d = layer();
        d.put("user:1", "alice");

        assertThat(d.get("user:1")).isEqualTo(Optional.of("alice"));
        assertThat(d.size()).isEqualTo(1);
    }

    @Test
    void shouldOverwriteWithoutGrowing() {
        DataAccess d = layer();
        d.put("k", "a");
        d.put("k", "b");

        assertThat(d.get("k")).contains("b");
        assertThat(d.size()).isEqualTo(1);
    }

    @Test
    void shouldAcceptTheEmptyStringAsAValue() {
        DataAccess d = layer();
        d.put("k", "");

        assertThat(d.get("k")).contains("");
        assertThat(d.size()).isEqualTo(1);
    }

    @Test
    void shouldReportWhetherRemoveDeletedSomething() {
        DataAccess d = layer();
        d.put("k", "v");

        assertThat(d.remove("k")).isTrue();
        assertThat(d.remove("k")).isFalse();
        assertThat(d.remove("never-there")).isFalse();
        assertThat(d.get("k")).isEmpty();
        assertThat(d.size()).isZero();
    }

    @Test
    void shouldRejectBlankKeysOnEveryOperationInEverySource() {
        DataAccess d = layer();
        for (String source : new String[] {"DATABASE", "API", "FILE"}) {
            d.switchSource(source);
            for (String bad : new String[] {null, "", "   ", "\t"}) {
                assertThatThrownBy(() -> d.get(bad)).isInstanceOf(IllegalArgumentException.class);
                assertThatThrownBy(() -> d.put(bad, "v")).isInstanceOf(IllegalArgumentException.class);
                assertThatThrownBy(() -> d.remove(bad)).isInstanceOf(IllegalArgumentException.class);
            }
            assertThat(d.size()).isZero();
        }
    }

    @Test
    void shouldRejectANullValue() {
        DataAccess d = layer();

        assertThatThrownBy(() -> d.put("k", null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(d.size()).isZero();
    }

    @Test
    void shouldRejectUnknownSourceNamesAndKeepTheCurrentOne() {
        DataAccess d = layer();
        d.switchSource("FILE");

        for (String bad : new String[] {null, "", " ", "SQL", "api", "Database", " FILE", "FILE "}) {
            assertThatThrownBy(() -> d.switchSource(bad)).isInstanceOf(IllegalArgumentException.class);
            assertThat(d.currentSource()).isEqualTo("FILE");
        }
    }

    @Test
    void shouldReportTheCurrentSourceAfterEachSwitch() {
        DataAccess d = layer();

        d.switchSource("API");
        assertThat(d.currentSource()).isEqualTo("API");
        d.switchSource("FILE");
        assertThat(d.currentSource()).isEqualTo("FILE");
        d.switchSource("DATABASE");
        assertThat(d.currentSource()).isEqualTo("DATABASE");
    }

    @Test
    void shouldKeepTheRecordsOfEachSourceIndependent() {
        DataAccess d = layer();
        d.put("k", "db");
        d.put("only-db", "1");

        d.switchSource("API");
        assertThat(d.get("k")).isEmpty();
        assertThat(d.size()).isZero();
        d.put("k", "api");

        d.switchSource("FILE");
        assertThat(d.get("k")).isEmpty();
        assertThat(d.get("only-db")).isEmpty();
        d.put("k", "file");

        assertThat(d.get("k")).contains("file");
        d.switchSource("API");
        assertThat(d.get("k")).contains("api");
        d.switchSource("DATABASE");
        assertThat(d.get("k")).contains("db");
        assertThat(d.get("only-db")).contains("1");
    }

    @Test
    void shouldShowTheOriginalRecordsAgainWhenSwitchingBack() {
        DataAccess d = layer();
        d.put("a", "1");
        d.put("b", "2");

        d.switchSource("API");
        d.put("c", "3");
        d.switchSource("DATABASE");

        assertThat(d.size()).isEqualTo(2);
        assertThat(d.get("a")).contains("1");
        assertThat(d.get("b")).contains("2");
        assertThat(d.get("c")).isEmpty();
    }

    @Test
    void shouldNotTouchRecordsWhenSwitchingToTheCurrentSource() {
        DataAccess d = layer();
        d.put("a", "1");

        d.switchSource("DATABASE");

        assertThat(d.get("a")).contains("1");
        assertThat(d.size()).isEqualTo(1);
    }

    @Test
    void shouldRemoveOnlyFromTheCurrentSource() {
        DataAccess d = layer();
        d.put("k", "db");
        d.switchSource("FILE");
        d.put("k", "file");

        assertThat(d.remove("k")).isTrue();
        assertThat(d.get("k")).isEmpty();

        d.switchSource("DATABASE");
        assertThat(d.get("k")).contains("db");
        assertThat(d.size()).isEqualTo(1);
    }

    @Test
    void shouldAcceptApiValuesUpToThirtyTwoCharactersAndRefuseLonger() {
        DataAccess d = layer();
        d.switchSource("API");

        d.put("ok", chars(32));
        assertThat(d.get("ok")).contains(chars(32));

        assertThatThrownBy(() -> d.put("too-long", chars(33))).isInstanceOf(IllegalArgumentException.class);
        assertThat(d.get("too-long")).isEmpty();
        assertThat(d.size()).isEqualTo(1);
    }

    @Test
    void shouldNotLimitValueLengthOnDatabaseAndFile() {
        DataAccess d = layer();

        d.put("big", chars(10_000));
        assertThat(d.get("big")).contains(chars(10_000));
        d.switchSource("FILE");
        d.put("big", chars(33));
        assertThat(d.get("big")).contains(chars(33));
    }

    @Test
    void shouldTreatApiKeysCaseInsensitivelyButNotTrimThem() {
        DataAccess d = layer();
        d.switchSource("API");

        d.put("Token", "t1");
        assertThat(d.get("token")).contains("t1");
        assertThat(d.get("TOKEN")).contains("t1");
        d.put("tOKEN", "t2");
        assertThat(d.size()).isEqualTo(1);
        assertThat(d.get("Token")).contains("t2");

        d.put(" token", "spaced");
        assertThat(d.size()).isEqualTo(2);
        assertThat(d.get("token")).contains("t2");
        assertThat(d.remove("TokeN")).isTrue();
        assertThat(d.get("token")).isEmpty();
    }

    @Test
    void shouldTrimFileKeysAndStayCaseSensitive() {
        DataAccess d = layer();
        d.switchSource("FILE");

        d.put("  report ", "v1");
        assertThat(d.get("report")).contains("v1");
        assertThat(d.get("\treport\n")).contains("v1");
        d.put("report", "v2");
        assertThat(d.size()).isEqualTo(1);

        d.put("Report", "other");
        assertThat(d.size()).isEqualTo(2);
        assertThat(d.get("report")).contains("v2");
        assertThat(d.get("Report")).contains("other");
        assertThat(d.remove(" report  ")).isTrue();
        assertThat(d.get("report")).isEmpty();
    }

    @Test
    void shouldUseDatabaseKeysExactlyAsGiven() {
        DataAccess d = layer();
        d.put("a", "1");
        d.put("A", "2");
        d.put(" a", "3");
        d.put("a ", "4");

        assertThat(d.size()).isEqualTo(4);
        assertThat(d.get("a")).contains("1");
        assertThat(d.get("A")).contains("2");
        assertThat(d.get(" a")).contains("3");
        assertThat(d.get("a ")).contains("4");
    }

    @Test
    void shouldLeaveStateUntouchedWhenAPutIsRefused() {
        DataAccess d = layer();
        d.switchSource("API");
        d.put("k", "good");

        assertThatThrownBy(() -> d.put("k", chars(33))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.put("k", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.put(" ", "v")).isInstanceOf(IllegalArgumentException.class);

        assertThat(d.get("k")).contains("good");
        assertThat(d.size()).isEqualTo(1);
    }

    @Test
    void shouldCountSizePerSource() {
        DataAccess d = layer();
        d.put("a", "1");
        d.put("b", "2");
        d.put("c", "3");
        d.switchSource("API");
        d.put("a", "1");
        d.switchSource("FILE");

        assertThat(d.size()).isZero();
        d.switchSource("API");
        assertThat(d.size()).isEqualTo(1);
        d.switchSource("DATABASE");
        assertThat(d.size()).isEqualTo(3);
    }

    @Test
    void shouldKeepWritesFromManyThreadsOnASingleSource() {
        DataAccess d = layer();
        d.switchSource("API");

        Concurrently.run(16, t -> {
            for (int i = 0; i < 500; i++) {
                d.put("t" + t + "-k" + i, "v" + i);
            }
        });

        assertThat(d.size()).isEqualTo(16 * 500);
        assertThat(d.get("t7-k123")).contains("v123");
    }

    @Test
    void shouldNeverFailAReadWhileAnotherThreadSwitchesSources() {
        DataAccess d = layer();
        for (String source : new String[] {"DATABASE", "API", "FILE"}) {
            d.switchSource(source);
            d.put("shared", source);
        }
        d.switchSource("DATABASE");
        Set<String> seen = ConcurrentHashMap.newKeySet();
        AtomicInteger missing = new AtomicInteger();

        Concurrently.run(5, t -> {
            if (t == 0) {
                String[] names = {"DATABASE", "API", "FILE"};
                for (int i = 0; i < 3_000; i++) {
                    d.switchSource(names[i % 3]);
                }
                return;
            }
            for (int i = 0; i < 20_000; i++) {
                Optional<String> v = d.get("shared");
                if (v.isEmpty()) {
                    missing.incrementAndGet();
                } else {
                    seen.add(v.get());
                }
            }
        });

        assertThat(missing).hasValue(0);
        assertThat(seen).isSubsetOf("DATABASE", "API", "FILE");
    }
}
