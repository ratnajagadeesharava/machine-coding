package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.FlagService;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldEvaluateAMillionTimesWhileRulesAreBeingEdited() {
        FlagService f = Candidate.of(Solution::create);
        f.define("x", false);
        f.setRollout("x", 50);

        Concurrently.run(9, 60_000, t -> {
            if (t == 8) {
                for (int i = 0; i < 2_000; i++) {
                    f.addRule("x", "tenant", "t" + i, i % 2 == 0);
                }
                return;
            }
            for (int i = 0; i < 125_000; i++) {
                f.isEnabled("x", "user-" + i, Map.of("tenant", "t" + (i % 2_000)));
            }
        });

        assertThat(f.isEnabled("x", "u", Map.of("tenant", "t0"))).isTrue();
        assertThat(f.isEnabled("x", "u", Map.of("tenant", "t1"))).isFalse();
    }

    @Test
    void shouldKeepAKilledFlagOffUnderConcurrentToggling() {
        FlagService f = Candidate.of(Solution::create);
        f.define("x", true);

        Concurrently.run(8, 60_000, t -> {
            for (int i = 0; i < 10_000; i++) {
                f.setKilled("x", true);
                f.setRollout("x", i % 101);
            }
        });

        assertThat(f.isEnabled("x", "u", Map.of())).isFalse();
    }
}
