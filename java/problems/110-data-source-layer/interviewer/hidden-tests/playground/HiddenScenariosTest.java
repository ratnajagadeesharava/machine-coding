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

class HiddenScenariosTest {

    private static final String[] SOURCES = {"DATABASE", "API", "FILE"};

    private DataAccess layer() {
        return Candidate.of(Solution::create);
    }

    private static int total(DataAccess d) {
        int sum = 0;
        for (String s : SOURCES) {
            d.switchSource(s);
            sum += d.size();
        }
        return sum;
    }

    @Test
    void shouldKeepTheOldValueWhenAnApiOverwriteIsRefused() {
        DataAccess d = layer();
        d.switchSource("API");
        d.put("Key", "old");

        assertThatThrownBy(() -> d.put("KEY", "y".repeat(33))).isInstanceOf(IllegalArgumentException.class);

        assertThat(d.get("key")).contains("old");
        assertThat(d.size()).isEqualTo(1);
    }

    @Test
    void shouldTreatAWhitespaceOnlyKeyAsBlankEvenOnFile() {
        DataAccess d = layer();
        d.switchSource("FILE");

        assertThatThrownBy(() -> d.put("   ", "v")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.get(" \t ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.remove("\n")).isInstanceOf(IllegalArgumentException.class);
        assertThat(d.size()).isZero();
    }

    @Test
    void shouldNeverApplyOneSourceNormalizationToAnother() {
        DataAccess d = layer();
        d.put(" Mixed ", "db");
        d.switchSource("FILE");
        d.put(" Mixed ", "file");
        d.switchSource("API");
        d.put(" Mixed ", "api");

        d.switchSource("DATABASE");
        assertThat(d.get(" Mixed ")).contains("db");
        assertThat(d.get("Mixed")).isEmpty();
        assertThat(d.get(" mixed ")).isEmpty();
        d.switchSource("FILE");
        assertThat(d.get("Mixed")).contains("file");
        assertThat(d.get("mixed")).isEmpty();
        d.switchSource("API");
        assertThat(d.get(" mixed ")).contains("api");
        assertThat(d.get("Mixed")).isEmpty();
    }

    @Test
    void shouldKeepLongValuesStoredElsewhereAfterATripThroughApi() {
        DataAccess d = layer();
        d.put("long", "z".repeat(40));
        d.switchSource("API");
        assertThat(d.get("long")).isEmpty();
        assertThatThrownBy(() -> d.put("long", "z".repeat(40))).isInstanceOf(IllegalArgumentException.class);
        d.switchSource("DATABASE");

        assertThat(d.get("long")).contains("z".repeat(40));
        assertThat(d.size()).isEqualTo(1);
    }

    @Test
    void shouldSendWritesToThePreviousSourceAfterAFailedSwitch() {
        DataAccess d = layer();
        d.switchSource("API");

        assertThatThrownBy(() -> d.switchSource("api")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.switchSource(null)).isInstanceOf(IllegalArgumentException.class);
        d.put("k", "v");

        assertThat(d.currentSource()).isEqualTo("API");
        assertThat(total(d)).isEqualTo(1);
        d.switchSource("API");
        assertThat(d.get("k")).contains("v");
    }

    @Test
    void shouldLoseNoRecordAcrossThousandsOfSwitches() {
        DataAccess d = layer();
        for (int i = 0; i < 300; i++) {
            d.switchSource(SOURCES[i % 3]);
            d.put("k" + i, "v" + i);
        }
        for (int i = 0; i < 6_000; i++) {
            d.switchSource(SOURCES[(i * 7) % 3]);
        }

        assertThat(total(d)).isEqualTo(300);
        d.switchSource("FILE");
        assertThat(d.get("k2")).contains("v2");
        d.switchSource("API");
        assertThat(d.get("k1")).contains("v1");
        d.switchSource("DATABASE");
        assertThat(d.get("k0")).contains("v0");
    }

    @Test
    void shouldLandEveryConcurrentWriteInExactlyOneSource() {
        DataAccess d = layer();
        AtomicInteger written = new AtomicInteger();

        Concurrently.run(9, 60_000, t -> {
            if (t == 8) {
                for (int i = 0; i < 6_000; i++) {
                    d.switchSource(SOURCES[i % 3]);
                }
                return;
            }
            for (int i = 0; i < 2_000; i++) {
                d.put("w" + t + "-" + i, "v");
                written.incrementAndGet();
            }
        });

        assertThat(total(d)).isEqualTo(written.get()).isEqualTo(8 * 2_000);
    }

    @Test
    void shouldNotLoseOrDuplicateRemovesWhileSwitching() {
        DataAccess d = layer();
        for (String s : SOURCES) {
            d.switchSource(s);
            for (int i = 0; i < 1_000; i++) {
                d.put("k" + i, "v");
            }
        }
        AtomicInteger removed = new AtomicInteger();

        Concurrently.run(5, 60_000, t -> {
            if (t == 0) {
                for (int i = 0; i < 4_000; i++) {
                    d.switchSource(SOURCES[i % 3]);
                }
                return;
            }
            for (int round = 0; round < 3; round++) {
                for (int i = 0; i < 1_000; i++) {
                    if (d.remove("k" + i)) {
                        removed.incrementAndGet();
                    }
                }
            }
        });

        assertThat(removed.get() + total(d)).isEqualTo(3_000);
    }

    @Test
    void shouldServeHeavyReadsWhileTheSourceIsSwitchedThousandsOfTimes() {
        DataAccess d = layer();
        for (String s : SOURCES) {
            d.switchSource(s);
            for (int i = 0; i < 100; i++) {
                d.put("k" + i, s + i);
            }
        }
        d.switchSource("DATABASE");
        Set<String> bad = ConcurrentHashMap.newKeySet();

        Concurrently.run(9, 60_000, t -> {
            if (t == 8) {
                for (int i = 0; i < 10_000; i++) {
                    d.switchSource(SOURCES[i % 3]);
                }
                return;
            }
            for (int i = 0; i < 100_000; i++) {
                int k = i % 100;
                Optional<String> v = d.get("k" + k);
                if (v.isEmpty() || !Set.of("DATABASE" + k, "API" + k, "FILE" + k).contains(v.get())) {
                    bad.add(String.valueOf(v));
                }
            }
        });

        assertThat(bad).isEmpty();
    }
}
