package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.KeyValueStoreApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class HiddenScenariosTest {
    private final FakeClock clock = new FakeClock(5_000L);

    private KeyValueStoreApi store() {
        return Candidate.of(() -> Solution.create(clock));
    }

    @Test
    void deleteInCommittedInnerTransactionShouldBeUndoneByOuterRollback() {
        KeyValueStoreApi kv = store();
        kv.put("k", "v");

        kv.begin();
        kv.begin();
        kv.delete("k");
        kv.commit();
        assertThat(kv.get("k")).isEmpty();
        kv.rollback();

        assertThat(kv.get("k")).hasValue("v");
        assertThat(kv.size()).isEqualTo(1);
    }

    @Test
    void plainPutInsideTransactionShouldRemoveTtlOnceCommitted() {
        KeyValueStoreApi kv = store();
        kv.putWithTtl("k", "v1", 1000);

        kv.begin();
        kv.put("k", "v2");
        kv.commit();
        clock.advance(5000);

        assertThat(kv.get("k")).hasValue("v2");
    }

    @Test
    void ttlStartedInsideTransactionShouldNotBeExtendedByCommit() {
        KeyValueStoreApi kv = store();

        kv.begin();
        kv.begin();
        kv.putWithTtl("k", "v", 100);
        clock.advance(100);
        kv.commit();
        kv.commit();

        assertThat(kv.get("k")).isEmpty();
        assertThat(kv.size()).isZero();
        assertThat(kv.keysWithPrefix("")).isEmpty();
    }

    @Test
    void ttlPutInsideRolledBackTransactionShouldLeaveOldPermanentValueAlone() {
        KeyValueStoreApi kv = store();
        kv.put("k", "v1");

        kv.begin();
        kv.putWithTtl("k", "v2", 10);
        kv.rollback();
        clock.advance(10_000);

        assertThat(kv.get("k")).hasValue("v1");
    }

    @Test
    void prefixListingAndSizeShouldMergeAllOpenLayers() {
        KeyValueStoreApi kv = store();
        kv.put("p:b", "1");
        kv.put("p:d", "1");
        kv.put("q:x", "1");

        kv.begin();
        kv.put("p:a", "2");
        kv.delete("p:b");
        kv.begin();
        kv.put("p:c", "3");
        kv.delete("p:d");
        kv.put("p:b", "back");

        assertThat(kv.keysWithPrefix("p:")).containsExactly("p:a", "p:b", "p:c");
        assertThat(kv.size()).isEqualTo(4);

        kv.rollback();
        assertThat(kv.keysWithPrefix("p:")).containsExactly("p:a", "p:d");
        assertThat(kv.size()).isEqualTo(3);
    }
}
