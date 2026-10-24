package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.VersionedStore;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldStayCorrectWhenAMillionWritesAndReadsInterleave() {
        VersionedStore s = Candidate.of(Solution::create);

        List<List<Long>> versions = Concurrently.collect(8, 120_000, t -> {
            List<Long> mine = new ArrayList<>();
            for (int i = 0; i < 60_000; i++) {
                mine.add(s.put("key-" + (i % 500), "t" + t + "-" + i));
                s.get("key-" + ((i * 7) % 500));
            }
            return mine;
        });

        Set<Long> all = new HashSet<>();
        versions.forEach(all::addAll);
        assertThat(all).hasSize(480_000);
        assertThat(s.currentVersion()).isEqualTo(480_000);
        assertThat(s.history("key-0")).hasSize(8 * 120);
    }

    @Test
    void shouldAnswerGetAtConsistentlyWithHistoryAfterAConcurrentMixedWorkload() {
        VersionedStore s = Candidate.of(Solution::create);

        Concurrently.run(8, t -> {
            for (int i = 0; i < 2_000; i++) {
                String key = "k" + (i % 20);
                switch (i % 5) {
                    case 0 -> s.delete(key);
                    case 1 -> s.putIfVersion(key, "c" + t, 0);
                    default -> s.put(key, "t" + t + "-" + i);
                }
            }
        });

        long end = s.currentVersion();
        for (int k = 0; k < 20; k++) {
            String key = "k" + k;
            Map<Long, String> entries = new LinkedHashMap<>();
            for (String line : s.history(key)) {
                int colon = line.indexOf(':');
                entries.put(Long.parseLong(line.substring(0, colon)), line.substring(colon + 1));
            }
            Optional<String> expected = Optional.empty();
            for (long v = 1; v <= end; v++) {
                String at = entries.get(v);
                if (at != null) {
                    expected = at.equals("<deleted>") ? Optional.empty() : Optional.of(at);
                }
                assertThat(s.getAt(key, v)).as("%s at %d", key, v).isEqualTo(expected);
            }
        }
    }

    @Test
    void shouldCompactWhileWritersAndSnapshotReadersAreRunning() {
        VersionedStore s = Candidate.of(Solution::create);
        s.putAll(Map.of("left", "0", "right", "0"));
        long pinned = s.snapshot();
        AtomicInteger torn = new AtomicInteger();

        Concurrently.run(8, t -> {
            if (t < 3) {
                for (int i = 1; i <= 500; i++) {
                    s.putAll(Map.of("left", t + "-" + i, "right", t + "-" + i));
                }
            } else if (t < 5) {
                for (int i = 0; i < 100; i++) {
                    long floor = s.currentVersion();
                    s.compact(floor);
                }
            } else {
                for (int i = 0; i < 400; i++) {
                    long snap = s.snapshot();
                    if (!s.snapshotGet(snap, "left").equals(s.snapshotGet(snap, "right"))) {
                        torn.incrementAndGet();
                    }
                    s.releaseSnapshot(snap);
                    assertThat(s.snapshotGet(pinned, "left")).contains("0");
                }
            }
        });

        assertThat(torn.get()).isZero();
        assertThat(s.snapshotGet(pinned, "left")).contains("0");
        assertThat(s.snapshotGet(pinned, "right")).contains("0");
        assertThat(s.get("left")).isEqualTo(s.get("right"));
        assertThat(s.currentVersion()).isEqualTo(1 + 3 * 500);
    }

    @Test
    void shouldReleaseHistoryOnlyWhenTheLastSnapshotReferencingItIsReleased() {
        VersionedStore s = Candidate.of(Solution::create);
        s.put("a", "v1"); // 1
        long first = s.snapshot();
        s.put("a", "v2"); // 2
        long second = s.snapshot();
        s.put("a", "v3"); // 3
        s.put("a", "v4"); // 4

        assertThat(s.compact(4)).isEqualTo(1);
        assertThat(s.history("a")).containsExactly("1:v1", "2:v2", "4:v4");

        s.releaseSnapshot(second);
        assertThat(s.compact(4)).isEqualTo(1);
        assertThat(s.history("a")).containsExactly("1:v1", "4:v4");
        assertThat(s.snapshotGet(first, "a")).contains("v1");

        s.releaseSnapshot(first);
        assertThat(s.compact(4)).isEqualTo(1);
        assertThat(s.history("a")).containsExactly("4:v4");
    }

    @Test
    void shouldKeepATombstoneThatHidesOlderHistoryAnotherSnapshotStillNeeds() {
        VersionedStore s = Candidate.of(Solution::create);
        s.put("a", "v1"); // 1
        long beforeDelete = s.snapshot();
        s.delete("a"); // 2
        long afterDelete = s.snapshot();
        s.put("b", "other"); // 3
        s.put("b", "other2"); // 4

        s.compact(4);

        assertThat(s.snapshotGet(beforeDelete, "a")).contains("v1");
        assertThat(s.snapshotGet(afterDelete, "a")).isEmpty();
        assertThat(s.get("a")).isEmpty();
        s.releaseSnapshot(beforeDelete);
        assertThat(s.snapshotGet(afterDelete, "a")).isEmpty();
    }

    @Test
    void shouldReuseCompareAndSetCorrectlyAcrossDeletesUnderContention() {
        VersionedStore s = Candidate.of(Solution::create);
        AtomicInteger wins = new AtomicInteger();
        AtomicInteger deletes = new AtomicInteger();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 2_000; i++) {
                if (s.putIfVersion("lock", "owner-" + t, 0).isPresent()) {
                    wins.incrementAndGet();
                    assertThat(s.get("lock")).isPresent();
                    if (s.delete("lock") > 0) {
                        deletes.incrementAndGet();
                    }
                }
            }
        });

        assertThat(wins.get()).isEqualTo(deletes.get()).isPositive();
        assertThat(s.get("lock")).isEmpty();
        assertThat(s.currentVersion()).isEqualTo(wins.get() + deletes.get());
        assertThat(s.history("lock")).hasSize(wins.get() + deletes.get());
    }

    @Test
    void shouldKeepBatchesAtomicWhenBatchesOverlapOnDifferentKeys() {
        VersionedStore s = Candidate.of(Solution::create);

        Concurrently.run(6, t -> {
            for (int i = 0; i < 1_000; i++) {
                Map<String, String> m = new HashMap<>();
                m.put("x" + (i % 3), "b" + t + "-" + i);
                m.put("x" + ((i + 1) % 3), "b" + t + "-" + i);
                s.putAll(m);
            }
        });

        assertThat(s.currentVersion()).isEqualTo(6_000);
        for (long v = 1; v <= 6_000; v += 97) {
            Map<String, String> seenAt = new HashMap<>();
            for (int k = 0; k < 3; k++) {
                String key = "x" + k;
                for (String line : s.history(key)) {
                    if (line.startsWith(v + ":")) {
                        seenAt.put(key, line.substring(line.indexOf(':') + 1));
                    }
                }
            }
            assertThat(seenAt).hasSize(2);
            assertThat(new HashSet<>(seenAt.values())).hasSize(1);
        }
        assertThatThrownBy(() -> s.getAt("x0", 6_001)).isInstanceOf(IllegalArgumentException.class);
    }
}
