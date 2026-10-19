package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.OrderedPrinter;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldHandOverTheTurnTwentyThousandTimesQuickly() {
        int workers = 4;
        OrderedPrinter p = Candidate.of(() -> Solution.create(workers));
        List<Integer> log = new ArrayList<>();

        Concurrently.run(workers, 20_000, id -> {
            for (int r = 0; r < 5_000; r++) {
                try {
                    p.takeTurn(id, () -> log.add(id));
                } catch (InterruptedException e) {
                    throw new AssertionError(e);
                }
            }
        });

        assertThat(log).hasSize(20_000);
        for (int i = 0; i < log.size(); i++) {
            assertThat(log.get(i)).isEqualTo(i % workers);
        }
    }

    @Test
    void shouldSupportARingOfOne() {
        OrderedPrinter p = Candidate.of(() -> Solution.create(1));
        List<Integer> log = new ArrayList<>();

        for (int i = 0; i < 1_000; i++) {
            try {
                p.takeTurn(0, () -> log.add(0));
            } catch (InterruptedException e) {
                throw new AssertionError(e);
            }
        }

        assertThat(log).hasSize(1_000);
    }
}
