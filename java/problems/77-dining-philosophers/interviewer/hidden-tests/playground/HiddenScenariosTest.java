package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Table;
import java.util.concurrent.atomic.AtomicIntegerArray;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldNotStarveAnyoneWhenNeighboursAreGreedy() {
        int seats = 5;
        Table t = Candidate.of(() -> Solution.create(seats));
        AtomicIntegerArray meals = new AtomicIntegerArray(seats);

        Concurrently.run(seats, 60_000, p -> {
            // even philosophers eat back to back, odd ones are slower; everyone must still finish
            int count = p % 2 == 0 ? 20_000 : 2_000;
            for (int m = 0; m < count; m++) {
                try {
                    t.dine(p, () -> {
                        meals.incrementAndGet(p);
                        if (p % 2 == 1) {
                            Thread.yield();
                        }
                    });
                } catch (InterruptedException e) {
                    throw new AssertionError(e);
                }
            }
        });

        for (int p = 0; p < seats; p++) {
            assertThat(meals.get(p)).isEqualTo(p % 2 == 0 ? 20_000 : 2_000);
        }
    }

    @Test
    void shouldSurviveOneHundredPhilosophers() {
        int seats = 100;
        Table t = Candidate.of(() -> Solution.create(seats));
        AtomicIntegerArray meals = new AtomicIntegerArray(seats);

        Concurrently.run(seats, 60_000, p -> {
            for (int m = 0; m < 300; m++) {
                try {
                    t.dine(p, () -> meals.incrementAndGet(p));
                } catch (InterruptedException e) {
                    throw new AssertionError(e);
                }
            }
        });

        for (int p = 0; p < seats; p++) {
            assertThat(meals.get(p)).isEqualTo(300);
        }
    }
}
