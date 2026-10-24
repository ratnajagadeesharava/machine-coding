package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.VersionedStore;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.OptionalLong;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private VersionedStore store() {
        return Candidate.of(Solution::create);
    }

    private static Map<String, String> batch(String... kv) {
        Map<String, String> m = new LinkedHashMap<>();
        for (int i = 0; i < kv.length; i += 2) {
            m.put(kv[i], kv[i + 1]);
        }
        return m;
    }

    @Test
    void shouldStartAtVersionZeroWhenNothingWasWritten() {
        VersionedStore s = store();

        assertThat(s.currentVersion()).isZero();
        assertThat(s.get("a")).isEmpty();
        assertThat(s.history("a")).isEmpty();
    }

    @Test
    void shouldNumberWritesOfAllKeysWithOneGapFreeCounter() {
        VersionedStore s = store();

        assertThat(s.put("a", "x")).isEqualTo(1);
        assertThat(s.put("b", "y")).isEqualTo(2);
        assertThat(s.put("a", "z")).isEqualTo(3);
        assertThat(s.currentVersion()).isEqualTo(3);
    }

    @Test
    void shouldReturnTheLatestValue() {
        VersionedStore s = store();
        s.put("a", "x");
        s.put("a", "z");

        assertThat(s.get("a")).contains("z");
        assertThat(s.get("other")).isEmpty();
    }

    @Test
    void shouldAcceptAnEmptyStringAsAValue() {
        VersionedStore s = store();
        long v = s.put("a", "");

        assertThat(s.get("a")).contains("");
        assertThat(s.getAt("a", v)).contains("");
        assertThat(s.history("a")).containsExactly("1:");
    }

    @Test
    void shouldRecordEveryWriteEvenWhenTheValueDoesNotChange() {
        VersionedStore s = store();

        assertThat(s.put("a", "same")).isEqualTo(1);
        assertThat(s.put("a", "same")).isEqualTo(2);
        assertThat(s.history("a")).containsExactly("1:same", "2:same");
    }

    @Test
    void shouldDeleteWithATombstoneThatConsumesAVersion() {
        VersionedStore s = store();
        s.put("a", "x");

        assertThat(s.delete("a")).isEqualTo(2);
        assertThat(s.get("a")).isEmpty();
        assertThat(s.currentVersion()).isEqualTo(2);
    }

    @Test
    void shouldReturnMinusOneAndConsumeNoVersionWhenDeletingAKeyWithoutLiveValue() {
        VersionedStore s = store();

        assertThat(s.delete("never")).isEqualTo(-1);
        s.put("a", "x");
        assertThat(s.delete("a")).isEqualTo(2);
        assertThat(s.delete("a")).isEqualTo(-1);

        assertThat(s.currentVersion()).isEqualTo(2);
        assertThat(s.history("a")).containsExactly("1:x", "2:<deleted>");
        assertThat(s.history("never")).isEmpty();
    }

    @Test
    void shouldAllowWritingAKeyAgainAfterItWasDeleted() {
        VersionedStore s = store();
        s.put("a", "x");
        s.delete("a");

        assertThat(s.put("a", "y")).isEqualTo(3);
        assertThat(s.get("a")).contains("y");
        assertThat(s.history("a")).containsExactly("1:x", "2:<deleted>", "3:y");
    }

    @Test
    void shouldAnswerGetAtWithTheNewestWriteAtOrBeforeTheVersion() {
        VersionedStore s = store();
        s.put("a", "x"); // 1
        s.put("b", "y"); // 2
        s.put("a", "z"); // 3
        s.put("c", "w"); // 4

        assertThat(s.getAt("a", 1)).contains("x");
        assertThat(s.getAt("a", 2)).contains("x");
        assertThat(s.getAt("a", 3)).contains("z");
        assertThat(s.getAt("a", 4)).contains("z");
        assertThat(s.getAt("b", 4)).contains("y");
    }

    @Test
    void shouldReturnEmptyFromGetAtBeforeTheKeyWasFirstWritten() {
        VersionedStore s = store();
        s.put("a", "x"); // 1
        s.put("b", "y"); // 2

        assertThat(s.getAt("b", 1)).isEmpty();
        assertThat(s.getAt("never", 2)).isEmpty();
    }

    @Test
    void shouldReturnEmptyFromGetAtAtAndAfterATombstoneButNotBeforeIt() {
        VersionedStore s = store();
        s.put("a", "x"); // 1
        s.put("b", "y"); // 2
        s.delete("b"); // 3
        s.put("c", "w"); // 4

        assertThat(s.getAt("b", 2)).contains("y");
        assertThat(s.getAt("b", 3)).isEmpty();
        assertThat(s.getAt("b", 4)).isEmpty();
    }

    @Test
    void shouldRejectGetAtOutsideTheVersionRange() {
        VersionedStore s = store();
        s.put("a", "x");
        s.put("a", "y");

        assertThatThrownBy(() -> s.getAt("a", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.getAt("a", -5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.getAt("a", 3)).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.getAt("a", 2)).contains("y");
    }

    @Test
    void shouldListHistoryOldestFirstWithTombstones() {
        VersionedStore s = store();
        s.put("a", "x"); // 1
        s.put("b", "y"); // 2
        s.put("a", "z"); // 3
        s.delete("b"); // 4

        assertThat(s.history("a")).containsExactly("1:x", "3:z");
        assertThat(s.history("b")).containsExactly("2:y", "4:<deleted>");
    }

    @Test
    void shouldWriteAllEntriesOfAPutAllAsOneVersion() {
        VersionedStore s = store();
        s.put("a", "old"); // 1

        long v = s.putAll(batch("a", "new", "b", "new", "c", "new"));

        assertThat(v).isEqualTo(2);
        assertThat(s.currentVersion()).isEqualTo(2);
        assertThat(s.history("b")).containsExactly("2:new");
        assertThat(s.getAt("a", 1)).contains("old");
        assertThat(s.getAt("a", 2)).contains("new");
        assertThat(s.getAt("b", 1)).isEmpty();
        assertThat(s.getAt("c", 2)).contains("new");
        assertThat(s.put("d", "x")).isEqualTo(3);
    }

    @Test
    void shouldRejectAnInvalidPutAllWithoutApplyingAnythingOrConsumingAVersion() {
        VersionedStore s = store();
        s.put("a", "x");
        Map<String, String> nullValue = batch("good", "1");
        nullValue.put("bad", null);

        assertThatThrownBy(() -> s.putAll(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.putAll(Map.of())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.putAll(batch("good", "1", " ", "2"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.putAll(nullValue)).isInstanceOf(IllegalArgumentException.class);

        assertThat(s.currentVersion()).isEqualTo(1);
        assertThat(s.get("good")).isEmpty();
        assertThat(s.put("b", "y")).isEqualTo(2);
    }

    @Test
    void shouldRejectBlankKeysAndNullValues() {
        VersionedStore s = store();
        s.put("a", "x");

        assertThatThrownBy(() -> s.put(null, "v")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.put("  ", "v")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.put("k", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.delete(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.get("")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.getAt(" ", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.history(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.putIfVersion(" ", "v", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.putIfVersion("k", null, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.putIfVersion("k", "v", -1)).isInstanceOf(IllegalArgumentException.class);

        assertThat(s.currentVersion()).isEqualTo(1);
    }

    @Test
    void shouldPutIfVersionZeroOnlyWhenTheKeyHasNoLiveValue() {
        VersionedStore s = store();

        assertThat(s.putIfVersion("k", "a", 0)).hasValue(1);
        assertThat(s.putIfVersion("k", "b", 0)).isEmpty();
        assertThat(s.get("k")).contains("a");
        assertThat(s.currentVersion()).isEqualTo(1);
    }

    @Test
    void shouldCompareAgainstTheVersionOfTheKeysLastWriteNotTheStoreVersion() {
        VersionedStore s = store();
        s.put("k", "a"); // 1
        s.put("other", "x"); // 2
        s.put("other", "y"); // 3

        assertThat(s.putIfVersion("k", "b", 3)).isEmpty();
        assertThat(s.putIfVersion("k", "b", 1)).hasValue(4);
        assertThat(s.putIfVersion("k", "c", 1)).isEmpty();
        assertThat(s.putIfVersion("k", "c", 4)).hasValue(5);
        assertThat(s.get("k")).contains("c");
        assertThat(s.currentVersion()).isEqualTo(5);
    }

    @Test
    void shouldNotConsumeAVersionOnACompareAndSetMismatch() {
        VersionedStore s = store();
        s.put("k", "a"); // 1

        assertThat(s.putIfVersion("k", "b", 7)).isEmpty();
        assertThat(s.putIfVersion("k", "b", 0)).isEmpty();

        assertThat(s.currentVersion()).isEqualTo(1);
        assertThat(s.history("k")).containsExactly("1:a");
        assertThat(s.put("x", "y")).isEqualTo(2);
    }

    @Test
    void shouldTreatADeletedKeyAsVersionZeroForCompareAndSet() {
        VersionedStore s = store();
        s.put("k", "a"); // 1
        s.delete("k"); // 2

        assertThat(s.putIfVersion("k", "b", 1)).isEmpty();
        assertThat(s.putIfVersion("k", "b", 2)).isEmpty();
        assertThat(s.putIfVersion("k", "b", 0)).hasValue(3);
        assertThat(s.get("k")).contains("b");
    }

    @Test
    void shouldFreezeASnapshotAgainstLaterWrites() {
        VersionedStore s = store();
        s.put("a", "1"); // 1
        s.put("b", "1"); // 2
        long snap = s.snapshot();

        s.put("a", "2");
        s.delete("b");
        s.put("c", "3");

        assertThat(s.snapshotGet(snap, "a")).contains("1");
        assertThat(s.snapshotGet(snap, "b")).contains("1");
        assertThat(s.snapshotGet(snap, "c")).isEmpty();
        assertThat(s.get("a")).contains("2");
        assertThat(s.get("b")).isEmpty();
    }

    @Test
    void shouldGiveEachSnapshotItsOwnViewAndDistinctIds() {
        VersionedStore s = store();
        long empty = s.snapshot();
        s.put("a", "1");
        long one = s.snapshot();
        s.put("a", "2");
        long two = s.snapshot();

        assertThat(Set.of(empty, one, two)).hasSize(3);
        assertThat(s.snapshotGet(empty, "a")).isEmpty();
        assertThat(s.snapshotGet(one, "a")).contains("1");
        assertThat(s.snapshotGet(two, "a")).contains("2");
        assertThat(s.currentVersion()).isEqualTo(2);
    }

    @Test
    void shouldRejectUnknownAndReleasedSnapshots() {
        VersionedStore s = store();
        s.put("a", "1");
        long snap = s.snapshot();
        long other = s.snapshot();

        assertThatThrownBy(() -> s.snapshotGet(9_999, "a")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.releaseSnapshot(9_999)).isInstanceOf(IllegalArgumentException.class);
        s.releaseSnapshot(snap);
        assertThatThrownBy(() -> s.snapshotGet(snap, "a")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.releaseSnapshot(snap)).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.snapshotGet(other, "a")).contains("1");
        assertThatThrownBy(() -> s.snapshotGet(other, " ")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldCompactOldEntriesAndReportHowManyWereRemoved() {
        VersionedStore s = store();
        s.put("a", "v1"); // 1
        s.put("a", "v2"); // 2
        s.put("b", "x"); // 3
        s.put("a", "v3"); // 4
        s.delete("b"); // 5
        s.put("c", "z"); // 6

        assertThat(s.compact(4)).isEqualTo(2);

        assertThat(s.history("a")).containsExactly("4:v3");
        assertThat(s.history("b")).containsExactly("3:x", "5:<deleted>");
        assertThat(s.getAt("a", 4)).contains("v3");
        assertThat(s.getAt("b", 4)).contains("x");
        assertThat(s.getAt("b", 5)).isEmpty();
        assertThat(s.getAt("c", 6)).contains("z");
        assertThat(s.get("a")).contains("v3");
        assertThat(s.compact(4)).isZero();
    }

    @Test
    void shouldRejectGetAtBelowTheCompactionFloorButKeepAnsweringAtAndAboveIt() {
        VersionedStore s = store();
        s.put("a", "v1"); // 1
        s.put("a", "v2"); // 2
        s.put("a", "v3"); // 3
        s.put("a", "v4"); // 4

        s.compact(3);

        assertThatThrownBy(() -> s.getAt("a", 1)).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> s.getAt("a", 2)).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> s.getAt("never", 2)).isInstanceOf(IllegalStateException.class);
        assertThat(s.getAt("a", 3)).contains("v3");
        assertThat(s.getAt("a", 4)).contains("v4");
        assertThatThrownBy(() -> s.getAt("a", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.getAt("a", 5)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldKeepTheEntryThatIsVisibleAtTheFloorEvenWhenItIsOlderThanTheFloor() {
        VersionedStore s = store();
        s.put("old", "kept"); // 1
        s.put("x", "1"); // 2
        s.put("x", "2"); // 3
        s.put("x", "3"); // 4

        assertThat(s.compact(4)).isEqualTo(2);

        assertThat(s.getAt("old", 4)).contains("kept");
        assertThat(s.history("old")).containsExactly("1:kept");
        assertThat(s.history("x")).containsExactly("4:3");
    }

    @Test
    void shouldForgetKeysWhoseTombstoneIsTheOnlyThingLeft() {
        VersionedStore s = store();
        s.put("a", "x"); // 1
        s.delete("a"); // 2
        s.put("b", "y"); // 3

        assertThat(s.compact(3)).isEqualTo(2);

        assertThat(s.history("a")).isEmpty();
        assertThat(s.get("a")).isEmpty();
        assertThat(s.getAt("a", 3)).isEmpty();
        assertThat(s.put("a", "again")).isEqualTo(4);
    }

    @Test
    void shouldKeepEntriesAnOpenSnapshotStillNeedsUntilItIsReleased() {
        VersionedStore s = store();
        s.put("a", "v1"); // 1
        s.put("a", "v2"); // 2
        long snap = s.snapshot();
        s.put("a", "v3"); // 3
        s.put("a", "v4"); // 4

        assertThat(s.compact(4)).isEqualTo(2);
        assertThat(s.snapshotGet(snap, "a")).contains("v2");
        assertThat(s.history("a")).containsExactly("2:v2", "4:v4");

        s.releaseSnapshot(snap);
        assertThat(s.compact(4)).isEqualTo(1);
        assertThat(s.history("a")).containsExactly("4:v4");
    }

    @Test
    void shouldKeepSnapshotsReadableThatWereTakenBeforeTheFloorMovedPastThem() {
        VersionedStore s = store();
        s.put("a", "v1"); // 1
        s.put("b", "w1"); // 2
        long snap = s.snapshot();
        s.delete("a"); // 3
        s.put("b", "w2"); // 4
        s.put("c", "z"); // 5

        s.compact(5);

        assertThat(s.snapshotGet(snap, "a")).contains("v1");
        assertThat(s.snapshotGet(snap, "b")).contains("w1");
        assertThat(s.snapshotGet(snap, "c")).isEmpty();
        assertThat(s.get("a")).isEmpty();
        assertThat(s.get("b")).contains("w2");
        assertThatThrownBy(() -> s.getAt("a", 2)).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldRejectCompactionOutsideTheVersionRange() {
        VersionedStore s = store();
        assertThatThrownBy(() -> s.compact(1)).isInstanceOf(IllegalArgumentException.class);
        s.put("a", "x");
        s.put("a", "y");

        assertThatThrownBy(() -> s.compact(0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.compact(-3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.compact(3)).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.history("a")).containsExactly("1:x", "2:y");
    }

    @Test
    void shouldNeverLowerTheFloorAndKeepNumberingVersionsAfterCompaction() {
        VersionedStore s = store();
        for (int i = 1; i <= 6; i++) {
            s.put("a", "v" + i);
        }
        assertThat(s.compact(5)).isEqualTo(4);

        assertThat(s.compact(2)).isZero();
        assertThatThrownBy(() -> s.getAt("a", 3)).isInstanceOf(IllegalStateException.class);
        assertThat(s.put("a", "v7")).isEqualTo(7);
        assertThat(s.currentVersion()).isEqualTo(7);
        assertThat(s.getAt("a", 6)).contains("v6");
        assertThat(s.history("a")).containsExactly("5:v5", "6:v6", "7:v7");
    }

    @Test
    void shouldHandOutEachVersionExactlyOnceWhenThreadsWriteDistinctKeys() {
        VersionedStore s = store();
        int threads = 8;
        int each = 500;

        List<List<Long>> perThread = Concurrently.collect(threads, t -> {
            List<Long> got = new ArrayList<>();
            for (int i = 0; i < each; i++) {
                got.add(s.put("key-" + t + "-" + i, "v"));
            }
            return got;
        });

        Set<Long> all = new HashSet<>();
        perThread.forEach(all::addAll);
        assertThat(all).hasSize(threads * each);
        assertThat(all).allMatch(v -> v >= 1 && v <= threads * each);
        assertThat(s.currentVersion()).isEqualTo(threads * each);
    }

    @Test
    void shouldKeepAKeysHistoryStrictlyIncreasingWhenThreadsWriteTheSameKey() {
        VersionedStore s = store();
        int threads = 8;
        int each = 300;

        List<List<Long>> perThread = Concurrently.collect(threads, t -> {
            List<Long> got = new ArrayList<>();
            for (int i = 0; i < each; i++) {
                got.add(s.put("hot", "t" + t + "-" + i));
            }
            return got;
        });

        perThread.forEach(list -> assertThat(list).isSorted());
        List<String> history = s.history("hot");
        assertThat(history).hasSize(threads * each);
        long previous = 0;
        for (String entry : history) {
            long version = Long.parseLong(entry.substring(0, entry.indexOf(':')));
            assertThat(version).isGreaterThan(previous);
            previous = version;
        }
        assertThat(s.currentVersion()).isEqualTo(threads * each);
    }

    @Test
    void shouldLetExactlyOneThreadWinACompareAndSetRace() {
        VersionedStore s = store();

        List<OptionalLong> results = Concurrently.collect(16, t -> s.putIfVersion("k", "from-" + t, 0));

        assertThat(results.stream().filter(OptionalLong::isPresent).count()).isEqualTo(1);
        assertThat(s.currentVersion()).isEqualTo(1);
        assertThat(s.history("k")).hasSize(1);

        long expected = 1;
        List<OptionalLong> second = Concurrently.collect(16, t -> s.putIfVersion("k", "second-" + t, expected));
        assertThat(second.stream().filter(OptionalLong::isPresent).count()).isEqualTo(1);
        assertThat(s.currentVersion()).isEqualTo(2);
    }

    @Test
    void shouldLetExactlyOneThreadDeleteALiveKey() {
        VersionedStore s = store();
        s.put("k", "x");

        List<Long> results = Concurrently.collect(16, t -> s.delete("k"));

        assertThat(results.stream().filter(v -> v > 0).count()).isEqualTo(1);
        assertThat(results.stream().filter(v -> v == -1).count()).isEqualTo(15);
        assertThat(s.currentVersion()).isEqualTo(2);
    }

    @Test
    void shouldNotLoseIncrementsWhenThreadsRetryCompareAndSet() {
        VersionedStore s = store();
        s.put("counter", "0");
        int threads = 8;
        int each = 200;

        Concurrently.run(threads, t -> {
            for (int i = 0; i < each; i++) {
                while (true) {
                    String last = s.history("counter").getLast();
                    int colon = last.indexOf(':');
                    long keyVersion = Long.parseLong(last.substring(0, colon));
                    int seen = Integer.parseInt(last.substring(colon + 1));
                    if (s.putIfVersion("counter", String.valueOf(seen + 1), keyVersion).isPresent()) {
                        break;
                    }
                }
            }
        });

        assertThat(s.get("counter")).contains(String.valueOf(threads * each));
        assertThat(s.currentVersion()).isEqualTo(1 + threads * each);
    }

    @Test
    void shouldShowBothKeysOfABatchOrNeitherInEverySnapshotTakenDuringWrites() {
        VersionedStore s = store();
        s.putAll(batch("left", "0", "right", "0"));
        AtomicInteger torn = new AtomicInteger();

        Concurrently.run(8, t -> {
            if (t < 3) {
                for (int i = 1; i <= 400; i++) {
                    String v = t + "-" + i;
                    s.putAll(batch("left", v, "right", v));
                }
                return;
            }
            for (int i = 0; i < 400; i++) {
                long snap = s.snapshot();
                Optional<String> left = s.snapshotGet(snap, "left");
                Optional<String> right = s.snapshotGet(snap, "right");
                if (!left.equals(right)) {
                    torn.incrementAndGet();
                }
                s.releaseSnapshot(snap);
            }
        });

        assertThat(torn.get()).isZero();
        assertThat(s.currentVersion()).isEqualTo(1 + 3 * 400);
        assertThat(s.get("left")).isEqualTo(s.get("right"));
    }

    @Test
    void shouldKeepASnapshotStableWhileOtherThreadsKeepWriting() {
        VersionedStore s = store();
        for (int i = 0; i < 50; i++) {
            s.put("k" + i, "base");
        }
        long snap = s.snapshot();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 300; i++) {
                s.put("k" + (i % 50), "t" + t);
                if (i % 7 == 0) {
                    s.delete("k" + (i % 50));
                }
                assertThat(s.snapshotGet(snap, "k" + (i % 50))).contains("base");
                assertThat(s.snapshotGet(snap, "new-" + t)).isEmpty();
            }
        });

        for (int i = 0; i < 50; i++) {
            assertThat(s.snapshotGet(snap, "k" + i)).contains("base");
        }
    }
}
