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
    void anEvictedKeyShouldComeBackWithCountOne() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);
        c.get("a");
        c.get("b");          // a: 2, b: 2 (a older)
        c.put("c", 3);       // evicts a
        c.put("a", 1);       // evicts c (count 1); a is back at count 1
        c.put("d", 4);       // a (count 1) must go, not b (count 2)
        assertThat(c.get("b")).hasValue(2);
        assertThat(c.get("d")).hasValue(4);
        assertThat(c.get("a")).isEmpty();
    }

    @Test
    void frequencyShouldBeatRecency() {
        Cache<String, Integer> c = cache(2);
        c.put("old", 1);
        c.get("old");
        c.put("fresh", 2);   // newer but count 1
        c.put("newer", 3);   // evicts fresh
        assertThat(c.get("fresh")).isEmpty();
        assertThat(c.get("old")).hasValue(1);
        assertThat(c.get("newer")).hasValue(3);
    }

    @Test
    void tieBreakShouldFollowTheOrderOfMostRecentGetsAndPuts() {
        Cache<String, Integer> c = cache(3);
        c.put("a", 1);
        c.put("b", 2);
        c.put("c", 3);
        c.get("a");
        c.get("b");
        c.get("c");          // all at count 2; recency a < b < c
        c.get("a");          // a: 3
        c.put("d", 4);       // lowest count 2 is shared by b and c; b is older
        assertThat(c.get("b")).isEmpty();
        assertThat(c.get("a")).hasValue(1);
        assertThat(c.get("c")).hasValue(3);
        assertThat(c.get("d")).hasValue(4);
    }

    @Test
    void repeatedPutsOfTheSameKeyShouldMakeItPopular() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("a", 2);
        c.put("a", 3);
        c.put("b", 10);
        c.put("c", 11);      // evicts b
        assertThat(c.get("a")).hasValue(3);
        assertThat(c.get("b")).isEmpty();
        assertThat(c.get("c")).hasValue(11);
    }

    @Test
    void removingTheOnlyLowCountEntryShouldMakeTheNextLowestTheVictim() {
        Cache<String, Integer> c = cache(3);
        c.put("a", 1);
        c.put("b", 2);
        c.put("c", 3);
        c.get("a");
        c.get("a");          // a: 3
        c.get("b");          // b: 2, c: 1
        assertThat(c.remove("c")).isTrue();
        c.put("d", 4);       // room available, nothing evicted
        assertThat(c.size()).isEqualTo(3);
        c.put("e", 5);       // d (count 1) goes
        assertThat(c.get("d")).isEmpty();
        assertThat(c.get("a")).hasValue(1);
        assertThat(c.get("b")).hasValue(2);
        assertThat(c.get("e")).hasValue(5);
    }
}
