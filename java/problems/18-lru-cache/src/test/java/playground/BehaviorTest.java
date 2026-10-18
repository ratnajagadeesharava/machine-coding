package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Cache;
import java.time.Duration;
import java.util.LinkedHashMap;
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
    void shouldEvictTheLeastRecentlyUsedEntryOnOverflow() {
        Cache<String, Integer> c = cache(2);

        c.put("a", 1);
        c.put("b", 2);
        c.put("c", 3);

        assertThat(c.get("a")).isEmpty();
        assertThat(c.get("b")).hasValue(2);
        assertThat(c.get("c")).hasValue(3);
        assertThat(c.size()).isEqualTo(2);
    }

    @Test
    void shouldTreatGetAsAUse() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);

        assertThat(c.get("a")).hasValue(1);
        c.put("c", 3);

        assertThat(c.get("b")).isEmpty();
        assertThat(c.get("a")).hasValue(1);
        assertThat(c.get("c")).hasValue(3);
    }

    @Test
    void shouldReplaceValueAndRefreshWhenPuttingAnExistingKey() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);

        c.put("a", 10);
        c.put("c", 3);

        assertThat(c.get("a")).hasValue(10);
        assertThat(c.get("b")).isEmpty();
        assertThat(c.get("c")).hasValue(3);
    }

    @Test
    void shouldNotEvictAnythingWhenUpdatingAKeyAtFullCapacity() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);

        c.put("b", 20);
        c.put("a", 10);

        assertThat(c.size()).isEqualTo(2);
        assertThat(c.get("a")).hasValue(10);
        assertThat(c.get("b")).hasValue(20);
    }

    @Test
    void shouldNotDisturbRecencyWhenLookingUpAbsentKeys() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);

        assertThat(c.get("zzz")).isEmpty();
        c.put("c", 3);

        assertThat(c.get("a")).isEmpty();
        assertThat(c.get("b")).hasValue(2);
    }

    @Test
    void shouldRemoveEntriesAndReportWhetherTheyExisted() {
        Cache<String, Integer> c = cache(3);
        c.put("a", 1);
        c.put("b", 2);

        assertThat(c.remove("a")).isTrue();
        assertThat(c.remove("a")).isFalse();
        assertThat(c.remove("never")).isFalse();

        assertThat(c.get("a")).isEmpty();
        assertThat(c.size()).isEqualTo(1);
    }

    @Test
    void shouldFreeTheSlotOfARemovedEntryImmediately() {
        Cache<String, Integer> c = cache(2);
        c.put("a", 1);
        c.put("b", 2);

        c.remove("a");
        c.put("c", 3);

        assertThat(c.get("b")).hasValue(2);
        assertThat(c.get("c")).hasValue(3);
        assertThat(c.size()).isEqualTo(2);
    }

    @Test
    void shouldWorkWithCapacityOne() {
        Cache<String, Integer> c = cache(1);

        c.put("a", 1);
        c.put("a", 2);
        assertThat(c.get("a")).hasValue(2);
        c.put("b", 3);

        assertThat(c.get("a")).isEmpty();
        assertThat(c.get("b")).hasValue(3);
        assertThat(c.size()).isEqualTo(1);
    }

    @Test
    void shouldEvictInUseOrderAcrossManyInserts() {
        Cache<String, Integer> c = cache(3);
        for (int i = 0; i < 10; i++) {
            c.put("k" + i, i);
        }

        assertThat(c.size()).isEqualTo(3);
        assertThat(c.get("k6")).isEmpty();
        assertThat(c.get("k7")).hasValue(7);
        assertThat(c.get("k8")).hasValue(8);
        assertThat(c.get("k9")).hasValue(9);
    }

    @Test
    void shouldRejectCapacityBelowOne() {
        cache(1); // skipped while not implemented
        assertThatThrownBy(() -> Solution.<String, Integer>create(0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.<String, Integer>create(-5)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldAgreeWithASimpleModelOnRandomOperations() {
        int capacity = 5;
        Cache<String, Integer> c = cache(capacity);
        LinkedHashMap<String, Integer> model = new LinkedHashMap<>(); // first = least recently used
        Random rnd = new Random(7);

        for (int step = 0; step < 5000; step++) {
            String key = "k" + rnd.nextInt(9);
            switch (rnd.nextInt(3)) {
                case 0 -> {
                    Integer v = model.remove(key);
                    if (v != null) {
                        model.put(key, v);
                    }
                    assertThat(c.get(key)).as("step %d get %s", step, key).isEqualTo(Optional.ofNullable(v));
                }
                case 1 -> {
                    if (model.remove(key) == null && model.size() == capacity) {
                        model.remove(model.keySet().iterator().next());
                    }
                    model.put(key, step);
                    c.put(key, step);
                }
                default -> assertThat(c.remove(key)).as("step %d remove %s", step, key).isEqualTo(model.remove(key) != null);
            }
            assertThat(c.size()).as("step %d size", step).isEqualTo(model.size());
        }
        for (Map.Entry<String, Integer> e : model.entrySet()) {
            assertThat(c.get(e.getKey())).hasValue(e.getValue());
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
