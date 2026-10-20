package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SagaLog;
import contract.SagaOrchestrator;
import contract.SagaStatus;
import contract.SagaStep;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.BiPredicate;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

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

    /** Scripted step: records every call into the shared journal ("exec:name", "comp:name"). */
    private static SagaStep step(String name, List<String> journal, BiPredicate<String, Integer> execOk, BiPredicate<String, Integer> compOk) {
        AtomicInteger execCalls = new AtomicInteger();
        AtomicInteger compCalls = new AtomicInteger();
        return new SagaStep() {
            @Override
            public String name() {
                return name;
            }

            @Override
            public boolean execute(String sagaId) {
                journal.add("exec:" + name);
                return execOk.test(sagaId, execCalls.incrementAndGet());
            }

            @Override
            public boolean compensate(String sagaId) {
                journal.add("comp:" + name);
                return compOk.test(sagaId, compCalls.incrementAndGet());
            }
        };
    }

    private static SagaStep good(String name, List<String> journal) {
        return step(name, journal, (id, n) -> true, (id, n) -> true);
    }

    private static SagaStep failing(String name, List<String> journal) {
        return step(name, journal, (id, n) -> false, (id, n) -> true);
    }

    private static SagaOrchestrator orchestrator(SagaLog log, List<Long> sleeps) {
        return Candidate.of(() -> Solution.orchestrator(log, sleeps::add));
    }

    private static List<SagaStep> steps(SagaStep... s) {
        return List.of(s);
    }

    @Test
    void shouldRunAllStepsInOrderAndComplete() {
        List<String> journal = new ArrayList<>();
        SagaOrchestrator o = orchestrator(new MemoryLog(), new ArrayList<>());

        SagaStatus result = o.run("s1", steps(good("reserve", journal), good("charge", journal), good("ship", journal)));

        assertThat(result).isEqualTo(SagaStatus.COMPLETED);
        assertThat(journal).containsExactly("exec:reserve", "exec:charge", "exec:ship");
        assertThat(o.status("s1")).isEqualTo(SagaStatus.COMPLETED);
    }

    @Test
    void shouldCompleteAnEmptySaga() {
        SagaOrchestrator o = orchestrator(new MemoryLog(), new ArrayList<>());

        assertThat(o.run("empty", List.of())).isEqualTo(SagaStatus.COMPLETED);
    }

    @Test
    void shouldReportNotFoundForAnUnknownSaga() {
        SagaOrchestrator o = orchestrator(new MemoryLog(), new ArrayList<>());

        assertThat(o.status("nope")).isEqualTo(SagaStatus.NOT_FOUND);
    }

    @Test
    void shouldCompensateCompletedStepsInReverseOrderWhenAStepFails() {
        List<String> journal = new ArrayList<>();
        SagaOrchestrator o = orchestrator(new MemoryLog(), new ArrayList<>());

        SagaStatus result = o.run("s1", steps(good("reserve", journal), good("charge", journal), failing("ship", journal), good("notify", journal)));

        assertThat(result).isEqualTo(SagaStatus.COMPENSATED);
        assertThat(journal).containsExactly(
                "exec:reserve", "exec:charge", "exec:ship", "exec:ship", "exec:ship", "comp:charge", "comp:reserve");
        assertThat(o.status("s1")).isEqualTo(SagaStatus.COMPENSATED);
    }

    @Test
    void shouldCompensateNothingWhenTheFirstStepFails() {
        List<String> journal = new ArrayList<>();
        SagaOrchestrator o = orchestrator(new MemoryLog(), new ArrayList<>());

        SagaStatus result = o.run("s1", steps(failing("reserve", journal), good("charge", journal)));

        assertThat(result).isEqualTo(SagaStatus.COMPENSATED);
        assertThat(journal).containsOnly("exec:reserve");
    }

    @Test
    void shouldRetryAFailingStepWithBackoffAndSucceedOnTheThirdAttempt() {
        List<String> journal = new ArrayList<>();
        List<Long> sleeps = new ArrayList<>();
        SagaOrchestrator o = orchestrator(new MemoryLog(), sleeps);
        SagaStep flaky = step("flaky", journal, (id, n) -> n >= 3, (id, n) -> true);

        SagaStatus result = o.run("s1", steps(good("a", journal), flaky));

        assertThat(result).isEqualTo(SagaStatus.COMPLETED);
        assertThat(journal).containsExactly("exec:a", "exec:flaky", "exec:flaky", "exec:flaky");
        assertThat(sleeps).containsExactly(100L, 200L);
    }

    @Test
    void shouldTreatAnExceptionLikeAFailure() {
        List<String> journal = new ArrayList<>();
        SagaOrchestrator o = orchestrator(new MemoryLog(), new ArrayList<>());
        SagaStep throwing = step("boom", journal, (id, n) -> {
            throw new IllegalStateException("payment gateway down");
        }, (id, n) -> true);

        SagaStatus result = o.run("s1", steps(good("a", journal), throwing));

        assertThat(result).isEqualTo(SagaStatus.COMPENSATED);
        assertThat(journal).containsExactly("exec:a", "exec:boom", "exec:boom", "exec:boom", "comp:a");
    }

    @Test
    void shouldNotSleepAfterTheLastFailedAttempt() {
        List<String> journal = new ArrayList<>();
        List<Long> sleeps = new ArrayList<>();
        SagaOrchestrator o = orchestrator(new MemoryLog(), sleeps);

        o.run("s1", steps(failing("a", journal)));

        assertThat(sleeps).containsExactly(100L, 200L);
    }

    @Test
    void shouldRetryCompensationsUpToFiveTimes() {
        List<String> journal = new ArrayList<>();
        List<Long> sleeps = new ArrayList<>();
        SagaOrchestrator o = orchestrator(new MemoryLog(), sleeps);
        SagaStep stubborn = step("charge", journal, (id, n) -> true, (id, n) -> n >= 4);

        SagaStatus result = o.run("s1", steps(good("reserve", journal), stubborn, failing("ship", journal)));

        assertThat(result).isEqualTo(SagaStatus.COMPENSATED);
        assertThat(journal.stream().filter("comp:charge"::equals).count()).isEqualTo(4);
        assertThat(sleeps).containsExactly(100L, 200L, 100L, 200L, 400L);
        assertThat(journal).endsWith("comp:charge", "comp:reserve");
    }

    @Test
    void shouldStopWithCompensationFailedAndNotCompensateTheRest() {
        List<String> journal = new ArrayList<>();
        List<Long> sleeps = new ArrayList<>();
        SagaOrchestrator o = orchestrator(new MemoryLog(), sleeps);
        SagaStep broken = step("charge", journal, (id, n) -> true, (id, n) -> false);

        SagaStatus result = o.run("s1", steps(good("reserve", journal), broken, failing("ship", journal)));

        assertThat(result).isEqualTo(SagaStatus.COMPENSATION_FAILED);
        assertThat(journal.stream().filter("comp:charge"::equals).count()).isEqualTo(5);
        assertThat(journal).doesNotContain("comp:reserve");
        assertThat(sleeps).containsSequence(100L, 200L, 100L, 200L, 400L, 800L);
        assertThat(o.status("s1")).isEqualTo(SagaStatus.COMPENSATION_FAILED);
    }

    @Test
    void shouldNotRunAFinishedSagaAgain() {
        List<String> journal = new ArrayList<>();
        SagaOrchestrator o = orchestrator(new MemoryLog(), new ArrayList<>());
        List<SagaStep> s = steps(good("a", journal), good("b", journal));
        o.run("s1", s);
        journal.clear();

        assertThat(o.run("s1", s)).isEqualTo(SagaStatus.COMPLETED);
        assertThat(journal).isEmpty();
    }

    @Test
    void shouldKeepSagasIndependent() {
        List<String> journal = new ArrayList<>();
        SagaOrchestrator o = orchestrator(new MemoryLog(), new ArrayList<>());

        assertThat(o.run("ok", steps(good("a", journal)))).isEqualTo(SagaStatus.COMPLETED);
        assertThat(o.run("bad", steps(failing("a", journal)))).isEqualTo(SagaStatus.COMPENSATED);
        assertThat(o.status("ok")).isEqualTo(SagaStatus.COMPLETED);
        assertThat(o.status("bad")).isEqualTo(SagaStatus.COMPENSATED);
    }

    @Test
    void shouldRecoverTheStatusFromTheLogAfterARestart() {
        MemoryLog log = new MemoryLog();
        List<String> journal = new ArrayList<>();
        SagaOrchestrator first = orchestrator(log, new ArrayList<>());
        first.run("done", steps(good("a", journal)));
        first.run("undone", steps(good("a", journal), failing("b", journal)));

        SagaOrchestrator second = orchestrator(log, new ArrayList<>());

        assertThat(second.status("done")).isEqualTo(SagaStatus.COMPLETED);
        assertThat(second.status("undone")).isEqualTo(SagaStatus.COMPENSATED);
        assertThat(second.status("other")).isEqualTo(SagaStatus.NOT_FOUND);
    }

    @Test
    void shouldContinueACrashedSagaWithoutRepeatingFinishedSteps() {
        // full run on one log, then pretend the process died after every possible number of log writes
        List<String> j0 = new ArrayList<>();
        MemoryLog full = new MemoryLog();
        orchestrator(full, new ArrayList<>()).run("s1", steps(good("a", j0), good("b", j0), good("c", j0)));
        List<String> all = full.records();

        for (int k = 0; k <= all.size(); k++) {
            List<String> journal = new ArrayList<>();
            SagaOrchestrator o = orchestrator(new MemoryLog(all.subList(0, k)), new ArrayList<>());

            SagaStatus result = o.run("s1", steps(good("a", journal), good("b", journal), good("c", journal)));

            assertThat(result).as("crash after %d writes", k).isEqualTo(SagaStatus.COMPLETED);
            assertThat(journal).as("crash after %d writes", k).doesNotHaveDuplicates();
            assertThat(journal).as("crash after %d writes", k).doesNotContain("comp:a", "comp:b", "comp:c");
        }
    }

    @Test
    void shouldContinueACrashedCompensationInReverseOrder() {
        List<String> j0 = new ArrayList<>();
        MemoryLog full = new MemoryLog();
        orchestrator(full, new ArrayList<>()).run("s1", steps(good("a", j0), good("b", j0), good("c", j0), failing("d", j0)));
        List<String> all = full.records();

        for (int k = 0; k <= all.size(); k++) {
            List<String> journal = new ArrayList<>();
            SagaOrchestrator o = orchestrator(new MemoryLog(all.subList(0, k)), new ArrayList<>());

            SagaStatus result = o.run("s1", steps(good("a", journal), good("b", journal), good("c", journal), failing("d", journal)));

            assertThat(result).as("crash after %d writes", k).isEqualTo(SagaStatus.COMPENSATED);
            assertThat(journal).as("crash after %d writes", k).doesNotContain("comp:d");
            List<String> comps = journal.stream().filter(e -> e.startsWith("comp:")).toList();
            assertThat(comps).as("crash after %d writes", k).isSubsetOf("comp:c", "comp:b", "comp:a");
            assertThat(comps).as("crash after %d writes", k).isSortedAccordingTo((x, y) -> y.compareTo(x));
        }
    }

    @Test
    void shouldRunManySagasConcurrentlyWithoutMixingThemUp() {
        MemoryLog log = new MemoryLog();
        SagaOrchestrator o = Candidate.of(() -> Solution.orchestrator(log, ms -> {}));
        ConcurrentHashMap<String, List<String>> journals = new ConcurrentHashMap<>();
        ConcurrentHashMap<String, SagaStatus> results = new ConcurrentHashMap<>();

        Concurrently.run(8, 30_000, t -> {
            for (int i = 0; i < 50; i++) {
                String id = "t" + t + "-" + i;
                List<String> journal = Collections.synchronizedList(new ArrayList<>());
                journals.put(id, journal);
                boolean doomed = i % 3 == 0;
                SagaStep last = doomed ? failing("last", journal) : good("last", journal);
                results.put(id, o.run(id, steps(good("a", journal), good("b", journal), last)));
            }
        });

        assertThat(results).hasSize(400);
        results.forEach((id, status) -> {
            boolean doomed = Integer.parseInt(id.substring(id.indexOf('-') + 1)) % 3 == 0;
            assertThat(status).as(id).isEqualTo(doomed ? SagaStatus.COMPENSATED : SagaStatus.COMPLETED);
            assertThat(o.status(id)).as(id).isEqualTo(status);
            if (doomed) {
                assertThat(journals.get(id)).as(id).containsExactly("exec:a", "exec:b", "exec:last", "exec:last", "exec:last", "comp:b", "comp:a");
            } else {
                assertThat(journals.get(id)).as(id).containsExactly("exec:a", "exec:b", "exec:last");
            }
        });
    }

    @Test
    void shouldReturnRunningForASagaThatAnotherThreadIsRunning() throws Exception {
        SagaOrchestrator o = orchestrator(new MemoryLog(), new ArrayList<>());
        java.util.concurrent.CountDownLatch inside = new java.util.concurrent.CountDownLatch(1);
        java.util.concurrent.CountDownLatch release = new java.util.concurrent.CountDownLatch(1);
        List<String> journal = new ArrayList<>();
        SagaStep slow = step("slow", journal, (id, n) -> {
            inside.countDown();
            try {
                release.await();
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            return true;
        }, (id, n) -> true);
        Thread worker = new Thread(() -> o.run("s1", steps(slow)));
        worker.start();
        assertThat(inside.await(5, java.util.concurrent.TimeUnit.SECONDS)).isTrue();

        assertThat(o.status("s1")).isEqualTo(SagaStatus.RUNNING);
        assertThat(o.run("s1", steps(slow))).isEqualTo(SagaStatus.RUNNING);
        release.countDown();
        worker.join(5_000);

        assertThat(o.status("s1")).isEqualTo(SagaStatus.COMPLETED);
        assertThat(journal).containsExactly("exec:slow");
    }
}
