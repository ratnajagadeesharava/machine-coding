package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Table;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicIntegerArray;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private Table table(int seats) {
        return Candidate.of(() -> Solution.create(seats));
    }

    private static void dine(Table t, int p, Runnable eat) {
        try {
            t.dine(p, eat);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new AssertionError("interrupted", e);
        }
    }

    /** Every philosopher eats {@code meals} times; returns how often two neighbours were eating at the same time. */
    private static int runMeals(Table t, int seats, int meals, Runnable insideMeal, AtomicIntegerArray meat) {
        AtomicIntegerArray eating = new AtomicIntegerArray(seats);
        AtomicInteger violations = new AtomicInteger();
        Concurrently.run(seats, p -> {
            for (int m = 0; m < meals; m++) {
                dine(t, p, () -> {
                    eating.set(p, 1);
                    if (eating.get((p + 1) % seats) == 1 || eating.get((p + seats - 1) % seats) == 1) {
                        violations.incrementAndGet();
                    }
                    insideMeal.run();
                    eating.set(p, 0);
                    meat.incrementAndGet(p);
                });
            }
        });
        return violations.get();
    }

    @Test
    void shouldLetASinglePhilosopherEat() {
        Table t = table(5);
        AtomicInteger meals = new AtomicInteger();

        dine(t, 2, meals::incrementAndGet);

        assertThat(meals).hasValue(1);
    }

    @Test
    void shouldNeverLetNeighboursEatAtTheSameTime() {
        Table t = table(5);
        AtomicIntegerArray meals = new AtomicIntegerArray(5);

        int violations = runMeals(t, 5, 500, Thread::yield, meals);

        assertThat(violations).as("neighbours eating simultaneously").isZero();
    }

    @Test
    void shouldFeedEveryoneEveryTimeWithoutDeadlock() {
        int seats = 5;
        Table t = table(seats);
        AtomicIntegerArray meals = new AtomicIntegerArray(seats);

        runMeals(t, seats, 2_000, () -> {}, meals);

        for (int p = 0; p < seats; p++) {
            assertThat(meals.get(p)).as("meals of philosopher %d", p).isEqualTo(2_000);
        }
    }

    @Test
    void shouldWorkForTwoSeatsWhereBothForksAreShared() {
        Table t = table(2);
        AtomicIntegerArray meals = new AtomicIntegerArray(2);

        int violations = runMeals(t, 2, 2_000, Thread::yield, meals);

        assertThat(violations).isZero();
        assertThat(meals.get(0)).isEqualTo(2_000);
        assertThat(meals.get(1)).isEqualTo(2_000);
    }

    @Test
    void shouldWorkForOtherTableSizes() {
        for (int seats : new int[] {3, 4, 7, 12}) {
            Table t = table(seats);
            AtomicIntegerArray meals = new AtomicIntegerArray(seats);

            int violations = runMeals(t, seats, 300, Thread::yield, meals);

            assertThat(violations).as("%d seats", seats).isZero();
            for (int p = 0; p < seats; p++) {
                assertThat(meals.get(p)).isEqualTo(300);
            }
        }
    }

    @Test
    void shouldLetNonNeighboursEatInParallel() {
        int seats = 5;
        Table t = table(seats);
        AtomicInteger eatingNow = new AtomicInteger();
        AtomicInteger maxEating = new AtomicInteger();

        Concurrently.run(seats, p -> {
            for (int m = 0; m < 20; m++) {
                dine(t, p, () -> {
                    maxEating.accumulateAndGet(eatingNow.incrementAndGet(), Math::max);
                    sleep(10);
                    eatingNow.decrementAndGet();
                });
            }
        });

        assertThat(maxEating.get()).as("a global lock would make this 1").isGreaterThanOrEqualTo(2);
        assertThat(maxEating.get()).as("at most floor(seats / 2) can eat at once").isLessThanOrEqualTo(seats / 2);
    }

    @Test
    void shouldPutTheForksDownWhenEatingThrows() {
        Table t = table(3);

        assertThatThrownBy(() -> dine(t, 0, () -> {
                    throw new IllegalStateException("choked");
                }))
                .isInstanceOf(IllegalStateException.class);

        AtomicInteger meals = new AtomicInteger();
        Concurrently.run(3, p -> dine(t, p, meals::incrementAndGet));
        assertThat(meals).hasValue(3);
    }

    @Test
    void shouldRejectUnknownPhilosophers() {
        Table t = table(5);

        assertThatThrownBy(() -> t.dine(-1, () -> {})).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.dine(5, () -> {})).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            int seats = 2 + run % 5;
            Table t = table(seats);
            AtomicIntegerArray meals = new AtomicIntegerArray(seats);

            int violations = runMeals(t, seats, 200, Thread::yield, meals);

            assertThat(violations).as("run %d (%d seats)", run, seats).isZero();
            for (int p = 0; p < seats; p++) {
                assertThat(meals.get(p)).as("run %d, philosopher %d", run, p).isEqualTo(200);
            }
        }
    }

    private static void sleep(long ms) {
        try {
            Thread.sleep(ms);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
}
