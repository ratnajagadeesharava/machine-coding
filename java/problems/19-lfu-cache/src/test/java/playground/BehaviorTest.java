package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Cache;
import java.time.Duration;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.Random;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    private Cache<String, Integer> cache(int capacity) {
        return Candidate.of(() -> Solution.<String, Integer>create(capacity));
    }

    @Test
    void shouldFollowTheClassicEvictionSequence() {
        Cache<String, Integer> c = cache(2);

        c.put("1", 1);
        c.put("2", 2);
        assertThat(c.get("1")).hasValue(1);
        c.put("3", 3);          // evicts 2 (count 1)
        assertThat(c.get("2")).isEmpty();
        assertThat(c.get("3")).hasValue(3);
        c.put("4", 4);          // 1 and 3 tie at count 2; 1 is least recently used
        assertThat(c.get("1")).isEmpty();
        assertThat(c.get("3")).hasValue(3);
        assertThat(c.get("4")).hasValue(4);
    }

    @Test
    void shouldReturnStoredValuesAndEmptyForAbsentKeys() {
        Cache<String, Integer> c = cache(3);

        c.put("a", 1);
        c.put("b", 2);

        assertThat(c.get("a")).hasValue(1);
        assertThat(c.get("b")).hasValue(2);
        assertThat(c.get("z")).isEmpty();
        assertThat(c.size()).isEqualTo(2);
    }

    @Test
    void shouldEvictTheLeastFrequentlyUsedEntry() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);
        c.get("a");

        c.put("c", 3);

        assertThat(c.get("b")).isEmpty();
        assertThat(c.get("a")).hasValue(1);
        assertThat(c.get("c")).hasValue(3);
    }

    @Test
    void shouldBreakTiesByLeastRecentUse() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);

        c.put("c", 3);

        assertThat(c.get("a")).isEmpty();
        assertThat(c.get("b")).hasValue(2);
        assertThat(c.get("c")).hasValue(3);
    }

    @Test
    void shouldCountPutOfAnExistingKeyAsAUseAndReplaceTheValue() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);

        c.put("a", 10);
        c.put("c", 3);

        assertThat(c.get("b")).isEmpty();
        assertThat(c.get("a")).hasValue(10);
        assertThat(c.get("c")).hasValue(3);
        assertThat(c.size()).isEqualTo(2);
    }

    @Test
    void shouldNotEvictWhenUpdatingAnExistingKeyAtFullCapacity() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);

        c.put("a", 5);
        c.put("b", 6);

        assertThat(c.size()).isEqualTo(2);
        assertThat(c.get("a")).hasValue(5);
        assertThat(c.get("b")).hasValue(6);
    }

    @Test
    void shouldStartNewKeysAtCountOneSoAFreshKeyIsTheNextVictim() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.get("a");
        c.get("a");          // a: count 3
        c.put("b", 2);       // b: count 1

        c.put("c", 3);       // evicts b, not a

        assertThat(c.get("b")).isEmpty();
        assertThat(c.get("a")).hasValue(1);
        assertThat(c.get("c")).hasValue(3);
    }

    @Test
    void shouldForgetTheCountOfARemovedKey() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.get("a");
        c.get("a");          // a: count 3
        assertThat(c.remove("a")).isTrue();
        c.put("a", 1);       // a: count 1 again
        c.put("b", 2);       // b: count 1, a is less recently used

        c.put("c", 3);       // tie at 1 -> evicts a

        assertThat(c.get("a")).isEmpty();
        assertThat(c.get("b")).hasValue(2);
        assertThat(c.get("c")).hasValue(3);
    }

    @Test
    void shouldRemoveEntriesAndFreeTheirSlots() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);

        assertThat(c.remove("a")).isTrue();
        assertThat(c.remove("a")).isFalse();
        assertThat(c.remove("never")).isFalse();
        c.put("c", 3);

        assertThat(c.size()).isEqualTo(2);
        assertThat(c.get("b")).hasValue(2);
        assertThat(c.get("c")).hasValue(3);
    }

    @Test
    void shouldNotDisturbCountsWhenLookingUpAbsentKeys() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);

        assertThat(c.get("zzz")).isEmpty();
        c.put("c", 3);

        assertThat(c.get("a")).isEmpty();
        assertThat(c.get("b")).hasValue(2);
    }

    @Test
    void shouldWorkWithCapacityOne() {
        Cache<String, Integer> c = cache(1);

        c.put("a", 1);
        c.get("a");
        c.put("b", 2);

        assertThat(c.get("a")).isEmpty();
        assertThat(c.get("b")).hasValue(2);
        assertThat(c.size()).isEqualTo(1);
    }

    @Test
    void shouldResetTheLowestCountToOneAfterAnInsert() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);
        for (int i = 0; i < 4; i++) {
            c.get("a");
        }
        for (int i = 0; i < 4; i++) {
            c.get("b");
        }                    // both at count 5, a less recently used

        c.put("c", 3);       // evicts a
        c.put("d", 4);       // c (count 1) must be the victim, not b

        assertThat(c.get("c")).isEmpty();
        assertThat(c.get("b")).hasValue(2);
        assertThat(c.get("d")).hasValue(4);
        assertThat(c.get("a")).isEmpty();
    }

    @Test
    void shouldRejectCapacityBelowOne() {
        cache(1); // skipped while not implemented
        assertThatThrownBy(() -> Solution.<String, Integer>create(0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.<String, Integer>create(-1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldAgreeWithASimpleModelOnRandomOperations() {
        int capacity = 3;
        Cache<String, Integer> c = cache(capacity);
        Map<String, int[]> model = new HashMap<>(); // key -> {value, count, lastUseTick}
        Random rnd = new Random(11);
        int tick = 0;

        for (int step = 0; step < 6000; step++) {
            String key = "k" + rnd.nextInt(7);
            switch (rnd.nextInt(3)) {
                case 0 -> {
                    int[] e = model.get(key);
                    Optional<Integer> expected = Optional.empty();
                    if (e != null) {
                        e[1]++;
                        e[2] = ++tick;
                        expected = Optional.of(e[0]);
                    }
                    assertThat(c.get(key)).as("step %d get %s", step, key).isEqualTo(expected);
                }
                case 1 -> {
                    int[] e = model.get(key);
                    if (e != null) {
                        e[0] = step;
                        e[1]++;
                        e[2] = ++tick;
                    } else {
                        if (model.size() == capacity) {
                            String victim = null;
                            for (Map.Entry<String, int[]> m : model.entrySet()) {
                                int[] v = m.getValue();
                                if (victim == null
                                        || v[1] < model.get(victim)[1]
                                        || (v[1] == model.get(victim)[1] && v[2] < model.get(victim)[2])) {
                                    victim = m.getKey();
                                }
                            }
                            model.remove(victim);
                        }
                        model.put(key, new int[] {step, 1, ++tick});
                    }
                    c.put(key, step);
                }
                default -> assertThat(c.remove(key)).as("step %d remove %s", step, key).isEqualTo(model.remove(key) != null);
            }
            assertThat(c.size()).as("step %d size", step).isEqualTo(model.size());
        }
    }

    @Test
    void shouldStayFastWithHundredsOfThousandsOfEntries() {
        Cache<String, Integer> c = cache(100_000);
        Assertions.assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            for (int i = 0; i < 200_000; i++) {
                c.put("k" + i, i);
            }
            assertThat(c.size()).isEqualTo(100_000);
            assertThat(c.get("k0")).isEmpty();
            assertThat(c.get("k99999")).isEmpty();
            assertThat(c.get("k199999")).hasValue(199_999);

            for (int i = 0; i < 1_000_000; i++) {
                String key = "k" + ((long) i * 7919 % 150_000);
                if (i % 4 == 0) {
                    c.put(key, i);
                } else {
                    c.get(key);
                }
            }
            assertThat(c.size()).isEqualTo(100_000);
        });
    }
}
