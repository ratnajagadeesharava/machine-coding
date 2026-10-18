package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.KeyValueStoreApi;
import java.time.Duration;
import java.util.List;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class BehaviorTest {
    private static final long NOW = 1_000_000L;

    private final FakeClock clock = new FakeClock(NOW);

    private KeyValueStoreApi store() {
        return Candidate.of(() -> Solution.create(clock));
    }

    @Test
    void shouldStoreOverwriteAndReadValues() {
        KeyValueStoreApi kv = store();

        kv.put("a", "1");
        assertThat(kv.get("a")).hasValue("1");
        kv.put("a", "2");

        assertThat(kv.get("a")).hasValue("2");
        assertThat(kv.get("missing")).isEmpty();
        assertThat(kv.size()).isEqualTo(1);
    }

    @Test
    void shouldReportWhetherDeleteRemovedAKey() {
        KeyValueStoreApi kv = store();
        kv.put("a", "1");

        assertThat(kv.delete("a")).isTrue();
        assertThat(kv.delete("a")).isFalse();
        assertThat(kv.delete("never")).isFalse();
        assertThat(kv.get("a")).isEmpty();
        assertThat(kv.size()).isZero();
    }

    @Test
    void shouldExpireKeyExactlyWhenTtlElapses() {
        KeyValueStoreApi kv = store();

        assertThat(kv.putWithTtl("session", "abc", 1000)).isTrue();

        clock.advance(999);
        assertThat(kv.get("session")).hasValue("abc");
        clock.advance(1);
        assertThat(kv.get("session")).isEmpty();
    }

    @Test
    void shouldTreatExpiredKeyAsMissingEverywhere() {
        KeyValueStoreApi kv = store();
        kv.putWithTtl("k1", "v", 100);
        kv.put("k2", "v");

        clock.advance(100);

        assertThat(kv.delete("k1")).isFalse();
        assertThat(kv.keysWithPrefix("k")).containsExactly("k2");
        assertThat(kv.size()).isEqualTo(1);
    }

    @Test
    void shouldDropTtlWhenOverwrittenByPlainPut() {
        KeyValueStoreApi kv = store();
        kv.putWithTtl("k", "v1", 100);

        kv.put("k", "v2");
        clock.advance(10_000);

        assertThat(kv.get("k")).hasValue("v2");
    }

    @Test
    void shouldRestartCountdownWhenPutWithTtlAgain() {
        KeyValueStoreApi kv = store();
        kv.putWithTtl("k", "v1", 1000);
        clock.advance(600);

        kv.putWithTtl("k", "v2", 1000);
        clock.advance(600);
        assertThat(kv.get("k")).hasValue("v2");

        clock.advance(399);
        assertThat(kv.get("k")).hasValue("v2");
        clock.advance(1);
        assertThat(kv.get("k")).isEmpty();
    }

    @Test
    void shouldRefuseNonPositiveTtlWithoutChangingAnything() {
        KeyValueStoreApi kv = store();
        kv.put("k", "keep");

        assertThat(kv.putWithTtl("k", "x", 0)).isFalse();
        assertThat(kv.putWithTtl("k", "x", -5)).isFalse();
        assertThat(kv.putWithTtl("new", "x", 0)).isFalse();

        assertThat(kv.get("k")).hasValue("keep");
        assertThat(kv.get("new")).isEmpty();
        assertThat(kv.size()).isEqualTo(1);
    }

    @Test
    void shouldAllowReusingAnExpiredKey() {
        KeyValueStoreApi kv = store();
        kv.putWithTtl("k", "old", 10);
        clock.advance(10);

        kv.put("k", "new");

        assertThat(kv.get("k")).hasValue("new");
        assertThat(kv.size()).isEqualTo(1);
    }

    @Test
    void shouldListKeysByPrefixInAscendingOrder() {
        KeyValueStoreApi kv = store();
        for (String k : List.of("user:2", "user:10", "order:1", "user:1", "usr", "u")) {
            kv.put(k, "v");
        }

        assertThat(kv.keysWithPrefix("user:")).containsExactly("user:1", "user:10", "user:2");
        assertThat(kv.keysWithPrefix("u")).containsExactly("u", "user:1", "user:10", "user:2", "usr");
        assertThat(kv.keysWithPrefix("")).containsExactly("order:1", "u", "user:1", "user:10", "user:2", "usr");
        assertThat(kv.keysWithPrefix("zzz")).isEmpty();
    }

    @Test
    void shouldCountOnlyLiveKeys() {
        KeyValueStoreApi kv = store();
        kv.put("a", "1");
        kv.put("b", "2");
        kv.putWithTtl("c", "3", 50);
        assertThat(kv.size()).isEqualTo(3);

        clock.advance(50);
        kv.delete("a");

        assertThat(kv.size()).isEqualTo(1);
    }

    @Test
    void shouldFailCommitAndRollbackWithoutOpenTransaction() {
        KeyValueStoreApi kv = store();

        assertThat(kv.commit()).isFalse();
        assertThat(kv.rollback()).isFalse();

        kv.begin();
        assertThat(kv.rollback()).isTrue();
        assertThat(kv.rollback()).isFalse();
        kv.begin();
        assertThat(kv.commit()).isTrue();
        assertThat(kv.commit()).isFalse();
    }

    @Test
    void shouldDiscardWritesAndDeletesOnRollback() {
        KeyValueStoreApi kv = store();
        kv.put("a", "1");
        kv.put("b", "2");

        kv.begin();
        kv.put("a", "changed");
        kv.delete("b");
        kv.put("c", "3");
        assertThat(kv.rollback()).isTrue();

        assertThat(kv.get("a")).hasValue("1");
        assertThat(kv.get("b")).hasValue("2");
        assertThat(kv.get("c")).isEmpty();
        assertThat(kv.keysWithPrefix("")).containsExactly("a", "b");
        assertThat(kv.size()).isEqualTo(2);
    }

    @Test
    void shouldShowTransactionWritesAndDeletesToReadsInsideIt() {
        KeyValueStoreApi kv = store();
        kv.put("a", "1");
        kv.put("b", "2");

        kv.begin();
        kv.put("a", "changed");
        kv.delete("b");
        kv.put("c", "3");

        assertThat(kv.get("a")).hasValue("changed");
        assertThat(kv.get("b")).isEmpty();
        assertThat(kv.get("c")).hasValue("3");
        assertThat(kv.keysWithPrefix("")).containsExactly("a", "c");
        assertThat(kv.size()).isEqualTo(2);
        assertThat(kv.delete("b")).isFalse();
    }

    @Test
    void shouldKeepChangesWhenOutermostTransactionCommits() {
        KeyValueStoreApi kv = store();
        kv.put("a", "1");
        kv.put("b", "2");

        kv.begin();
        kv.put("a", "changed");
        kv.delete("b");
        kv.put("c", "3");
        assertThat(kv.commit()).isTrue();

        assertThat(kv.get("a")).hasValue("changed");
        assertThat(kv.get("b")).isEmpty();
        assertThat(kv.get("c")).hasValue("3");
        assertThat(kv.rollback()).isFalse();
        assertThat(kv.size()).isEqualTo(2);
    }

    @Test
    void shouldRollBackOnlyTheInnermostTransaction() {
        KeyValueStoreApi kv = store();

        kv.begin();
        kv.put("outer", "1");
        kv.begin();
        kv.put("inner", "2");
        kv.put("outer", "overwritten");
        assertThat(kv.rollback()).isTrue();

        assertThat(kv.get("inner")).isEmpty();
        assertThat(kv.get("outer")).hasValue("1");
        assertThat(kv.commit()).isTrue();
        assertThat(kv.get("outer")).hasValue("1");
    }

    @Test
    void shouldMergeInnerCommitIntoParentOnly() {
        KeyValueStoreApi kv = store();

        kv.begin();
        kv.put("x", "1");
        kv.begin();
        kv.put("x", "2");
        assertThat(kv.commit()).isTrue();

        assertThat(kv.get("x")).hasValue("2");
        assertThat(kv.rollback()).isTrue();
        assertThat(kv.get("x")).isEmpty();
        assertThat(kv.size()).isZero();
    }

    @Test
    void shouldMakeInnerCommitPermanentOnlyWhenOutermostCommits() {
        KeyValueStoreApi kv = store();

        kv.begin();
        kv.begin();
        kv.put("x", "deep");
        kv.commit();
        assertThat(kv.get("x")).hasValue("deep");
        kv.commit();

        assertThat(kv.get("x")).hasValue("deep");
        assertThat(kv.rollback()).isFalse();
    }

    @Test
    void shouldKeepTtlDeadlineFromThePutCallWhenTransactionCommitsLater() {
        KeyValueStoreApi kv = store();

        kv.begin();
        kv.putWithTtl("k", "v", 1000);
        clock.advance(600);
        kv.commit();

        clock.advance(399);
        assertThat(kv.get("k")).hasValue("v");
        clock.advance(1);
        assertThat(kv.get("k")).isEmpty();
    }

    @Test
    void shouldHideKeyThatExpiresWhileTransactionIsOpen() {
        KeyValueStoreApi kv = store();
        kv.begin();
        kv.putWithTtl("k", "v", 100);
        kv.put("stay", "v");

        clock.advance(100);

        assertThat(kv.get("k")).isEmpty();
        assertThat(kv.keysWithPrefix("")).containsExactly("stay");
        assertThat(kv.size()).isEqualTo(1);
        assertThat(kv.rollback()).isTrue();
        assertThat(kv.size()).isZero();
    }

    @Test
    void shouldRestoreOverwrittenValueWithItsOriginalDeadlineOnRollback() {
        KeyValueStoreApi kv = store();
        kv.putWithTtl("k", "v1", 1000);
        clock.advance(500);

        kv.begin();
        kv.put("k", "v2");
        kv.rollback();

        assertThat(kv.get("k")).hasValue("v1");
        clock.advance(499);
        assertThat(kv.get("k")).hasValue("v1");
        clock.advance(1);
        assertThat(kv.get("k")).isEmpty();
    }

    @Test
    void shouldNotResurrectKeyThatExpiredDuringRolledBackTransaction() {
        KeyValueStoreApi kv = store();
        kv.putWithTtl("k", "v", 100);

        kv.begin();
        kv.delete("k");
        clock.advance(200);
        kv.rollback();

        assertThat(kv.get("k")).isEmpty();
        assertThat(kv.size()).isZero();
    }

    @Test
    void shouldHandleLargeStoresQuickly() {
        KeyValueStoreApi kv = store();

        Assertions.assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            for (int i = 0; i < 50_000; i++) {
                kv.put(String.format("key:%05d", i), "v" + i);
            }
            for (int i = 0; i < 50_000; i++) {
                assertThat(kv.get(String.format("key:%05d", i))).hasValue("v" + i);
            }
            for (int i = 0; i < 100; i++) {
                assertThat(kv.keysWithPrefix(String.format("key:%04d", i * 10))).hasSize(10);
                assertThat(kv.size()).isEqualTo(50_000);
            }
            kv.begin();
            for (int i = 0; i < 50_000; i++) {
                kv.delete(String.format("key:%05d", i));
            }
            assertThat(kv.size()).isZero();
            kv.rollback();
            assertThat(kv.size()).isEqualTo(50_000);
        });
    }
}
