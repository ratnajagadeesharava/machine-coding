package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SagaLog;
import contract.SagaOrchestrator;
import contract.SagaStatus;
import contract.SagaStep;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Random;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static final class MemoryLog implements SagaLog {
        private final List<String> records = Collections.synchronizedList(new ArrayList<>());

        MemoryLog() {}

        MemoryLog(List<String> initial) {
            records.addAll(initial);
        }

        @Override
        public void append(String record) {
            records.add(record);
        }

        @Override
        public List<String> records() {
            synchronized (records) {
                return new ArrayList<>(records);
            }
        }
    }

    /** Fails its first {@code execFails} executes and first {@code compFails} compensations. */
    private static final class Scripted implements SagaStep {
        final String name;
        final int execFails;
        final int compFails;
        final List<String> journal;
        final AtomicInteger execs = new AtomicInteger();
        final AtomicInteger comps = new AtomicInteger();

        Scripted(String name, int execFails, int compFails, List<String> journal) {
            this.name = name;
            this.execFails = execFails;
            this.compFails = compFails;
            this.journal = journal;
        }

        @Override
        public String name() {
            return name;
        }

        @Override
        public boolean execute(String sagaId) {
            journal.add("exec:" + name);
            return execs.incrementAndGet() > execFails;
        }

        @Override
        public boolean compensate(String sagaId) {
            journal.add("comp:" + name);
            return comps.incrementAndGet() > compFails;
        }
    }

    @Test
    void shouldSurviveACrashAtEveryPointOfRandomSagas() {
        for (long seed = 0; seed < 60; seed++) {
            Random rnd = new Random(seed);
            int n = 2 + rnd.nextInt(4);
            int[] execFails = new int[n];
            int[] compFails = new int[n];
            for (int i = 0; i < n; i++) {
                execFails[i] = rnd.nextInt(5) == 0 ? 3 : rnd.nextInt(3);
                compFails[i] = rnd.nextInt(6) == 0 ? 5 : rnd.nextInt(3);
            }
            MemoryLog full = new MemoryLog();
            List<String> j0 = new ArrayList<>();
            SagaOrchestrator o0 = Candidate.of(() -> Solution.orchestrator(full, ms -> {}));
            SagaStatus expected = o0.run("s", build(n, execFails, compFails, j0));
            List<String> all = full.records();

            for (int k = 0; k <= all.size(); k++) {
                List<String> journal = new ArrayList<>();
                SagaOrchestrator o = Solution.orchestrator(new MemoryLog(all.subList(0, k)), ms -> {});
                SagaStatus got = o.run("s", build(n, execFails, compFails, journal));
                assertThat(got).as("seed %d crash after %d", seed, k).isEqualTo(expected);
                assertThat(o.status("s")).isEqualTo(expected);
                if (k == all.size()) assertThat(journal).as("a finished saga must not run again").isEmpty();
                List<String> comps = journal.stream().filter(e -> e.startsWith("comp:")).toList();
                assertThat(comps).as("reverse order").isSortedAccordingTo((x, y) -> y.compareTo(x));
            }
        }
    }

    private static List<SagaStep> build(int n, int[] execFails, int[] compFails, List<String> journal) {
        List<SagaStep> steps = new ArrayList<>();
        for (int i = 0; i < n; i++) steps.add(new Scripted("s" + i, execFails[i], compFails[i], journal));
        return steps;
    }

    @Test
    void shouldRunTheSameSagaOnlyOnceWhenManyThreadsRaceForIt() {
        MemoryLog log = new MemoryLog();
        SagaOrchestrator o = Candidate.of(() -> Solution.orchestrator(log, ms -> {}));
        for (int round = 0; round < 100; round++) {
            String id = "race" + round;
            List<String> journal = Collections.synchronizedList(new ArrayList<>());
            List<SagaStep> steps = List.of(new Scripted("a", 0, 0, journal), new Scripted("b", 0, 0, journal));
            ConcurrentHashMap<SagaStatus, AtomicInteger> seen = new ConcurrentHashMap<>();

            Concurrently.run(8, 10_000, t -> seen.computeIfAbsent(o.run(id, steps), s -> new AtomicInteger()).incrementAndGet());

            assertThat(journal).as(id).containsExactly("exec:a", "exec:b");
            assertThat(o.status(id)).isEqualTo(SagaStatus.COMPLETED);
            assertThat(seen.getOrDefault(SagaStatus.COMPLETED, new AtomicInteger()).get()).isGreaterThanOrEqualTo(1);
        }
    }
}
