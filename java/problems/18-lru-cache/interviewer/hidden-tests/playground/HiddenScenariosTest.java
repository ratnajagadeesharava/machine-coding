package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Cache;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private Cache<String, Integer> cache(int capacity) {
        return Candidate.of(() -> Solution.<String, Integer>create(capacity));
    }

    @Test
    void capacityOneShouldKeepOnlyTheNewestKey() {
        Cache<String, Integer> c = cache(1);
        c.put("a", 1);
        c.put("b", 2);
        c.put("c", 3);
        assertThat(c.size()).isEqualTo(1);
        assertThat(c.get("a")).isEmpty();
        assertThat(c.get("b")).isEmpty();
        assertThat(c.get("c")).hasValue(3);
    }

    @Test
    void aReinsertedRemovedKeyShouldBeTheMostRecentlyUsed() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);
        c.remove("a");
        c.put("a", 5);   // order now: b, a
        c.put("c", 3);   // evicts b
        assertThat(c.get("b")).isEmpty();
        assertThat(c.get("a")).hasValue(5);
        assertThat(c.get("c")).hasValue(3);
    }

    @Test
    void repeatedGetsOfTheSameKeyShouldKeepItAliveAgainstAStreamOfInserts() {
        Cache<String, Integer> c = cache(3);
        c.put("hot", 1);
        for (int i = 0; i < 50; i++) {
            c.put("cold" + i, i);
            assertThat(c.get("hot")).hasValue(1);
        }
        assertThat(c.size()).isEqualTo(3);
        assertThat(c.get("cold48")).hasValue(48);
        assertThat(c.get("cold49")).hasValue(49);
        assertThat(c.get("cold47")).isEmpty();
    }

    @Test
    void getOrderShouldDetermineWhoIsEvictedNext() {
        Cache<String, Integer> c = cache(3);
        c.put("a", 1);
        c.put("b", 2);
        c.put("c", 3);
        c.get("a");
        c.get("c");
        c.get("b");      // use order oldest..newest: a, c, b
        c.put("d", 4);   // evicts a
        c.put("e", 5);   // evicts c
        assertThat(c.get("a")).isEmpty();
        assertThat(c.get("c")).isEmpty();
        assertThat(c.get("b")).hasValue(2);
        assertThat(c.get("d")).hasValue(4);
        assertThat(c.get("e")).hasValue(5);
    }

    @Test
    void removingEverythingShouldLeaveAnEmptyReusableCache() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);
        assertThat(c.remove("a")).isTrue();
        assertThat(c.remove("b")).isTrue();
        assertThat(c.size()).isZero();
        c.put("x", 1);
        c.put("y", 2);
        assertThat(c.size()).isEqualTo(2);
        assertThat(c.get("x")).hasValue(1);
    }
}
