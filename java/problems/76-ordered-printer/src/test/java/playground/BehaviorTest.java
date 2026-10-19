package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.OrderedPrinter;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.Eventually;

class BehaviorTest {

    private OrderedPrinter printer(int workers) {
        return Candidate.of(() -> Solution.create(workers));
    }

    private static void turn(OrderedPrinter p, int worker, Runnable action) {
        try {
            p.takeTurn(worker, action);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new AssertionError("interrupted", e);
        }
    }

    /** Every worker takes {@code rounds} turns; the log is a plain ArrayList on purpose (the printer must publish writes). */
    private static List<Integer> runRounds(OrderedPrinter p, int workers, int rounds) {
        List<Integer> log = new ArrayList<>();
        Concurrently.run(workers, id -> {
            for (int r = 0; r < rounds; r++) {
                turn(p, id, () -> log.add(id));
            }
        });
        return log;
    }

    private static List<Integer> expected(int workers, int rounds) {
        List<Integer> out = new ArrayList<>();
        for (int r = 0; r < rounds; r++) {
            for (int w = 0; w < workers; w++) {
                out.add(w);
            }
        }
        return out;
    }

    @Test
    void shouldRunWorkersInStrictRoundRobin() {
        OrderedPrinter p = printer(3);

        List<Integer> log = runRounds(p, 3, 50);

        assertThat(log).isEqualTo(expected(3, 50));
    }

    @Test
    void shouldWorkWithASingleWorker() {
        OrderedPrinter p = printer(1);

        List<Integer> log = runRounds(p, 1, 100);

        assertThat(log).hasSize(100).containsOnly(0);
    }

    @Test
    void shouldWorkWithManyWorkers() {
        OrderedPrinter p = printer(16);

        List<Integer> log = runRounds(p, 16, 30);

        assertThat(log).isEqualTo(expected(16, 30));
    }

    @Test
    void shouldKeepOrderWhenWorkersArriveInReverse() throws Exception {
        OrderedPrinter p = printer(4);
        List<Integer> log = new ArrayList<>();
        List<Thread> threads = new ArrayList<>();
        for (int id = 3; id >= 0; id--) {
            final int worker = id;
            Thread t = new Thread(() -> turn(p, worker, () -> log.add(worker)));
            t.setDaemon(true);
            t.start();
            threads.add(t);
            Thread.sleep(30);
        }

        for (Thread t : threads) {
            t.join(5_000);
        }

        assertThat(log).containsExactly(0, 1, 2, 3);
    }

    @Test
    void shouldBlockAnOutOfTurnWorkerUntilItsTurn() {
        OrderedPrinter p = printer(2);
        AtomicBoolean ran = new AtomicBoolean();
        Thread second = new Thread(() -> turn(p, 1, () -> ran.set(true)));
        second.setDaemon(true);
        second.start();

        Eventually.assertStays(() -> !ran.get(), 150, "worker 1 must wait for worker 0");
        turn(p, 0, () -> {});

        Eventually.assertTrue(ran::get, 5_000, "worker 1 runs once worker 0 is done");
    }

    @Test
    void shouldNeverRunTwoActionsAtTheSameTime() {
        int workers = 6;
        OrderedPrinter p = printer(workers);
        AtomicInteger inside = new AtomicInteger();
        AtomicInteger maxInside = new AtomicInteger();

        Concurrently.run(workers, id -> {
            for (int r = 0; r < 100; r++) {
                turn(p, id, () -> {
                    maxInside.accumulateAndGet(inside.incrementAndGet(), Math::max);
                    Thread.yield();
                    inside.decrementAndGet();
                });
            }
        });

        assertThat(maxInside.get()).isEqualTo(1);
    }

    @Test
    void shouldHandTheTurnOverEvenWhenTheActionThrows() {
        OrderedPrinter p = printer(2);
        AtomicBoolean secondRan = new AtomicBoolean();

        assertThatThrownBy(() -> turn(p, 0, () -> {
                    throw new IllegalStateException("boom");
                }))
                .isInstanceOf(IllegalStateException.class)
                .hasMessage("boom");

        Concurrently.run(1, i -> turn(p, 1, () -> secondRan.set(true)));
        assertThat(secondRan).isTrue();
    }

    @Test
    void shouldRejectUnknownWorkers() {
        OrderedPrinter p = printer(3);

        assertThatThrownBy(() -> p.takeTurn(-1, () -> {})).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.takeTurn(3, () -> {})).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            int workers = 2 + run % 6;
            OrderedPrinter p = printer(workers);

            List<Integer> log = runRounds(p, workers, 20);

            assertThat(log).as("run %d with %d workers", run, workers).isEqualTo(expected(workers, 20));
        }
    }
}
