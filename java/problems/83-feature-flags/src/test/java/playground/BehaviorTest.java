package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.FlagService;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static final Map<String, String> NONE = Map.of();

    private FlagService flags() {
        return Candidate.of(Solution::create);
    }

    private static int countOn(FlagService f, String flag, int users) {
        int on = 0;
        for (int u = 0; u < users; u++) {
            if (f.isEnabled(flag, "user-" + u, NONE)) {
                on++;
            }
        }
        return on;
    }

    @Test
    void shouldReturnTheDefaultValue() {
        FlagService f = flags();
        f.define("on", true);
        f.define("off", false);

        assertThat(f.isEnabled("on", "u1", NONE)).isTrue();
        assertThat(f.isEnabled("off", "u1", NONE)).isFalse();
    }

    @Test
    void shouldRejectUnknownFlagsDuplicatesAndBadInput() {
        FlagService f = flags();
        f.define("x", false);

        assertThatThrownBy(() -> f.isEnabled("nope", "u1", NONE)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.define("x", true)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.define(" ", true)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.isEnabled("x", null, NONE)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.isEnabled("x", " ", NONE)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.addRule("nope", "a", "b", true)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.setRollout("x", 101)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.setRollout("x", -1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldApplyTheFirstMatchingRule() {
        FlagService f = flags();
        f.define("beta", false);
        f.addRule("beta", "country", "IN", true);
        f.addRule("beta", "country", "IN", false);
        f.addRule("beta", "plan", "free", false);
        f.addRule("beta", "plan", "pro", true);

        assertThat(f.isEnabled("beta", "u", Map.of("country", "IN"))).isTrue();
        assertThat(f.isEnabled("beta", "u", Map.of("plan", "pro"))).isTrue();
        assertThat(f.isEnabled("beta", "u", Map.of("plan", "free", "country", "US"))).isFalse();
        assertThat(f.isEnabled("beta", "u", Map.of("country", "US"))).isFalse();
        assertThat(f.isEnabled("beta", "u", NONE)).isFalse();
    }

    @Test
    void shouldLetRulesOverrideTheRollout() {
        FlagService f = flags();
        f.define("x", false);
        f.setRollout("x", 0);
        f.addRule("x", "vip", "yes", true);

        assertThat(f.isEnabled("x", "anyone", Map.of("vip", "yes"))).isTrue();
        assertThat(f.isEnabled("x", "anyone", NONE)).isFalse();
    }

    @Test
    void shouldLetTheKillSwitchOverrideEverything() {
        FlagService f = flags();
        f.define("x", true);
        f.addRule("x", "vip", "yes", true);
        f.setRollout("x", 100);

        f.setKilled("x", true);

        assertThat(f.isEnabled("x", "u", Map.of("vip", "yes"))).isFalse();
        assertThat(f.isEnabled("x", "u", NONE)).isFalse();
        f.setKilled("x", false);
        assertThat(f.isEnabled("x", "u", NONE)).isTrue();
    }

    @Test
    void shouldHonourTheRolloutExtremes() {
        FlagService f = flags();
        f.define("none", true);
        f.define("all", false);
        f.setRollout("none", 0);
        f.setRollout("all", 100);

        assertThat(countOn(f, "none", 2_000)).isZero();
        assertThat(countOn(f, "all", 2_000)).isEqualTo(2_000);
    }

    @Test
    void shouldRollOutToRoughlyTheRequestedPercentage() {
        FlagService f = flags();
        f.define("x", false);
        f.setRollout("x", 30);

        int on = countOn(f, "x", 20_000);

        assertThat(on).isBetween(5_400, 6_600);
    }

    @Test
    void shouldGiveTheSameUserTheSameAnswerEveryTime() {
        FlagService f = flags();
        f.define("x", false);
        f.setRollout("x", 50);
        List<Boolean> first = new ArrayList<>();
        for (int u = 0; u < 1_000; u++) {
            first.add(f.isEnabled("x", "user-" + u, NONE));
        }

        for (int round = 0; round < 3; round++) {
            for (int u = 0; u < 1_000; u++) {
                assertThat(f.isEnabled("x", "user-" + u, NONE)).isEqualTo(first.get(u));
            }
        }
    }

    @Test
    void shouldNeverTurnAnybodyOffWhenTheRolloutGrows() {
        FlagService f = flags();
        f.define("x", false);
        f.setRollout("x", 30);
        boolean[] on = new boolean[10_000];
        for (int u = 0; u < on.length; u++) {
            on[u] = f.isEnabled("x", "user-" + u, NONE);
        }

        f.setRollout("x", 60);

        int total = 0;
        for (int u = 0; u < on.length; u++) {
            boolean now = f.isEnabled("x", "user-" + u, NONE);
            if (on[u]) {
                assertThat(now).as("user-%d was on at 30%%", u).isTrue();
            }
            total += now ? 1 : 0;
        }
        assertThat(total).isBetween(5_600, 6_400);
    }

    @Test
    void shouldBucketDifferentFlagsIndependently() {
        FlagService f = flags();
        f.define("a", false);
        f.define("b", false);
        f.setRollout("a", 50);
        f.setRollout("b", 50);
        int both = 0;
        for (int u = 0; u < 20_000; u++) {
            if (f.isEnabled("a", "user-" + u, NONE) && f.isEnabled("b", "user-" + u, NONE)) {
                both++;
            }
        }

        assertThat(both).as("correlated buckets would give ~50%").isBetween(4_400, 5_600);
    }

    @Test
    void shouldLoseNoRuleWhenManyThreadsAddRules() {
        FlagService f = flags();
        f.define("x", false);

        Concurrently.run(16, t -> f.addRule("x", "k" + t, "v", true));

        for (int t = 0; t < 16; t++) {
            assertThat(f.isEnabled("x", "u", Map.of("k" + t, "v"))).as("rule %d", t).isTrue();
        }
    }

    @Test
    void shouldDefineManyFlagsConcurrentlyWhileEvaluating() {
        FlagService f = flags();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 200; i++) {
                String name = "flag-" + t + "-" + i;
                f.define(name, i % 2 == 0);
                f.setRollout(name, 100);
                assertThat(f.isEnabled(name, "u", NONE)).isTrue();
            }
        });

        assertThat(f.isEnabled("flag-3-10", "u", NONE)).isTrue();
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            FlagService f = flags();
            f.define("x", false);

            Concurrently.run(8, t -> {
                f.addRule("x", "k" + t, "v", true);
                f.setRollout("x", 100);
            });

            for (int t = 0; t < 8; t++) {
                assertThat(f.isEnabled("x", "u", Map.of("k" + t, "v"))).as("run %d rule %d", run, t).isTrue();
            }
        }
    }
}
