package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.JobScheduler;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static final String FEWEST = JobScheduler.FEWEST_UNFINISHED;
    private static final String MOST = JobScheduler.MOST_FINISHED;

    private static Set<String> caps(String... c) {
        return Set.of(c);
    }

    @Test
    void shouldInterleaveBothCriteriaOnTheSameMachines() {
        JobScheduler s = Candidate.of(Solution::create);
        s.addMachine("a", caps("x"));
        s.addMachine("b", caps("x"));
        s.addMachine("c", caps("x"));

        assertThat(s.submitJob("j1", caps("x"), MOST)).contains("a");   // all tie at 0 finished
        assertThat(s.submitJob("j2", caps("x"), FEWEST)).contains("b"); // a:1 b:0 c:0 -> b
        s.finishJob("j2");                                               // b finished 1
        assertThat(s.submitJob("j3", caps("x"), MOST)).contains("b");   // b has the most finished
        assertThat(s.submitJob("j4", caps("x"), FEWEST)).contains("c"); // a:1 b:1 c:0
        s.finishJob("j4");                                               // c finished 1
        s.finishJob("j3");                                               // b finished 2
        assertThat(s.submitJob("j5", caps("x"), MOST)).contains("b");
        assertThat(s.submitJob("j6", caps("x"), FEWEST)).contains("c"); // a:1 b:1 c:0
        assertThat(s.unfinishedCount("a")).isEqualTo(1);
        assertThat(s.unfinishedCount("b")).isEqualTo(1);
        assertThat(s.unfinishedCount("c")).isEqualTo(1);
        assertThat(s.finishedCount("b")).isEqualTo(2);
        assertThat(s.finishedCount("c")).isEqualTo(1);
    }

    @Test
    void shouldNeverPickAMachineMissingEvenOneCapabilityAcrossManyMachines() {
        JobScheduler s = Candidate.of(Solution::create);
        String[] all = {"c0", "c1", "c2", "c3", "c4"};
        Map<String, Set<String>> offered = new HashMap<>();
        for (int m = 0; m < 200; m++) {
            Set<String> c = new java.util.HashSet<>();
            for (int k = 0; k < all.length; k++) {
                if (((m >> k) & 1) == 1) {
                    c.add(all[k]);
                }
            }
            offered.put("m" + String.format("%03d", m), c);
            s.addMachine("m" + String.format("%03d", m), c);
        }
        int submitted = 0;
        for (int j = 0; j < 3000; j++) {
            Set<String> need = new java.util.HashSet<>();
            for (int k = 0; k < all.length; k++) {
                if (((j * 7 + j / 5) >> k & 1) == 1 && (j % 3 != 0 || k < 2)) {
                    need.add(all[k]);
                }
            }
            Optional<String> m = s.submitJob("job" + j, need, j % 2 == 0 ? FEWEST : MOST);
            if (m.isPresent()) {
                submitted++;
                assertThat(offered.get(m.get())).containsAll(need);
                if (j % 4 == 0) {
                    s.finishJob("job" + j);
                }
            }
        }
        int total = 0;
        for (String m : offered.keySet()) {
            total += s.unfinishedCount(m) + s.finishedCount(m);
        }
        assertThat(total).isEqualTo(submitted);
    }

    @Test
    void shouldSpreadJobsEvenlyAcrossEligibleMachinesOnlyAndIgnoreOthers() {
        JobScheduler s = Candidate.of(Solution::create);
        for (int i = 0; i < 10; i++) {
            s.addMachine("gpu-" + i, caps("gpu"));
            s.addMachine("cpu-" + i, caps("cpu"));
        }
        for (int j = 0; j < 100; j++) {
            assertThat(s.submitJob("g" + j, caps("gpu"), FEWEST)).isPresent();
        }
        for (int i = 0; i < 10; i++) {
            assertThat(s.unfinishedCount("gpu-" + i)).isEqualTo(10);
            assertThat(s.unfinishedCount("cpu-" + i)).isEqualTo(0);
        }
    }

    @Test
    void shouldAccumulateMostFinishedOnOneMachineWhileLoadGrowsThere() {
        JobScheduler s = Candidate.of(Solution::create);
        s.addMachine("a", caps("x"));
        s.addMachine("b", caps("x"));
        s.submitJob("seed", caps("x"), FEWEST); // a
        s.submitJob("seed2", caps("x"), FEWEST); // b
        s.finishJob("seed2");                    // b leads on finished

        for (int i = 0; i < 50; i++) {
            assertThat(s.submitJob("m" + i, caps("x"), MOST)).contains("b"); // load on b is ignored
        }
        assertThat(s.unfinishedCount("b")).isEqualTo(50);
        assertThat(s.unfinishedCount("a")).isEqualTo(1);
        assertThat(s.submitJob("f", caps("x"), FEWEST)).contains("a");
    }

    @Test
    void shouldLeaveNoTraceOfRejectedOperations() {
        JobScheduler s = Candidate.of(Solution::create);
        s.addMachine("a", caps("x"));
        s.submitJob("j1", caps("x"), FEWEST);
        s.finishJob("j1");

        assertThatThrownBy(() -> s.finishJob("j1")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> s.submitJob("j1", caps("x"), FEWEST)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.submitJob("j2", caps("x"), "BOGUS")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addMachine("a", caps("y"))).isInstanceOf(IllegalArgumentException.class);

        assertThat(s.unfinishedCount("a")).isEqualTo(0);
        assertThat(s.finishedCount("a")).isEqualTo(1);
        assertThat(s.submitJob("j2", caps("x"), FEWEST)).contains("a"); // j2 id still free after BOGUS
        assertThat(s.submitJob("j3", caps("y"), FEWEST)).isEmpty();      // original capabilities kept
    }

    @Test
    void shouldRegisterExactlyOneMachineWhenManyThreadsAddTheSameId() {
        JobScheduler s = Candidate.of(Solution::create);
        AtomicInteger ok = new AtomicInteger();

        Concurrently.run(16, t -> {
            try {
                s.addMachine("only", caps("c" + t));
                ok.incrementAndGet();
            } catch (IllegalArgumentException expected) {
                // lost the race
            }
        });

        assertThat(ok.get()).isEqualTo(1);
    }

    @Test
    void shouldAcceptEachJobIdOnceWhenManyThreadsSubmitTheSameIds() {
        JobScheduler s = Candidate.of(Solution::create);
        s.addMachine("a", caps("x"));
        s.addMachine("b", caps("x"));
        AtomicInteger accepted = new AtomicInteger();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 500; i++) {
                try {
                    s.submitJob("job-" + i, caps("x"), FEWEST);
                    accepted.incrementAndGet();
                } catch (IllegalArgumentException duplicate) {
                    // another thread got there first
                }
            }
        });

        assertThat(accepted.get()).isEqualTo(500);
        assertThat(s.unfinishedCount("a") + s.unfinishedCount("b")).isEqualTo(500);
        assertThat(Math.abs(s.unfinishedCount("a") - s.unfinishedCount("b"))).isLessThanOrEqualTo(1);
    }

    @Test
    void shouldKeepInvariantsWhileMachinesJoinAndJobsFlowConcurrently() {
        JobScheduler s = Candidate.of(Solution::create);
        s.addMachine("seed", caps("x"));
        List<String> placed = java.util.Collections.synchronizedList(new ArrayList<>());

        Concurrently.run(9, 60_000, t -> {
            if (t == 8) {
                for (int i = 0; i < 200; i++) {
                    s.addMachine("late-" + i, caps("x"));
                }
                return;
            }
            for (int i = 0; i < 1_000; i++) {
                String id = "j-" + t + "-" + i;
                Optional<String> m = s.submitJob(id, caps("x"), i % 2 == 0 ? FEWEST : MOST);
                assertThat(m).isPresent();
                placed.add(id);
                if (i % 5 == 0) {
                    s.finishJob(id);
                }
            }
        });

        int unfinished = s.unfinishedCount("seed");
        int finished = s.finishedCount("seed");
        for (int i = 0; i < 200; i++) {
            unfinished += s.unfinishedCount("late-" + i);
            finished += s.finishedCount("late-" + i);
        }
        assertThat(unfinished + finished).isEqualTo(8_000);
        assertThat(finished).isEqualTo(8 * 200);
        assertThat(placed).hasSize(8_000);
    }
}
