package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.JobScheduler;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static final String FEWEST = JobScheduler.FEWEST_UNFINISHED;
    private static final String MOST = JobScheduler.MOST_FINISHED;

    private JobScheduler scheduler() {
        return Candidate.of(Solution::create);
    }

    private static Set<String> caps(String... c) {
        return Set.of(c);
    }

    @Test
    void shouldAssignAJobToTheOnlyEligibleMachine() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("gpu", "ssd"));

        assertThat(s.submitJob("j1", caps("gpu"), FEWEST)).contains("m1");
        assertThat(s.unfinishedCount("m1")).isEqualTo(1);
        assertThat(s.finishedCount("m1")).isEqualTo(0);
        assertThat(s.machineOf("j1")).isEqualTo("m1");
    }

    @Test
    void shouldReturnEmptyWhenNoMachineHasAllRequiredCapabilities() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("gpu"));
        s.addMachine("m2", caps("ssd"));

        assertThat(s.submitJob("j1", caps("gpu", "ssd"), FEWEST)).isEmpty();
        assertThat(s.unfinishedCount("m1")).isEqualTo(0);
        assertThat(s.unfinishedCount("m2")).isEqualTo(0);
    }

    @Test
    void shouldReturnEmptyWhenThereAreNoMachines() {
        JobScheduler s = scheduler();

        assertThat(s.submitJob("j1", caps(), FEWEST)).isEmpty();
        assertThat(s.submitJob("j2", caps("gpu"), MOST)).isEmpty();
    }

    @Test
    void shouldFitAJobWithNoRequirementsOnAnyMachine() {
        JobScheduler s = scheduler();
        s.addMachine("b", caps());
        s.addMachine("a", caps("gpu"));

        assertThat(s.submitJob("j1", caps(), FEWEST)).contains("a");
        assertThat(s.submitJob("j2", caps(), FEWEST)).contains("b");
    }

    @Test
    void shouldMatchCapabilitiesExactly() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("GPU", "gpu-a100"));

        assertThat(s.submitJob("j1", caps("gpu"), FEWEST)).isEmpty();
        assertThat(s.submitJob("j2", caps("gpu-a"), FEWEST)).isEmpty();
        assertThat(s.submitJob("j3", caps("GPU"), FEWEST)).contains("m1");
    }

    @Test
    void shouldPickTheMachineWithFewestUnfinishedJobs() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("x"));
        s.addMachine("m2", caps("x"));
        s.addMachine("m3", caps("x"));

        assertThat(s.submitJob("j1", caps("x"), FEWEST)).contains("m1");
        assertThat(s.submitJob("j2", caps("x"), FEWEST)).contains("m2");
        assertThat(s.submitJob("j3", caps("x"), FEWEST)).contains("m3");
        assertThat(s.submitJob("j4", caps("x"), FEWEST)).contains("m1");
        s.finishJob("j2");
        assertThat(s.submitJob("j5", caps("x"), FEWEST)).contains("m2");
    }

    @Test
    void shouldBreakFewestUnfinishedTiesBySmallestMachineIdNotInsertionOrder() {
        JobScheduler s = scheduler();
        s.addMachine("m9", caps("x"));
        s.addMachine("m10", caps("x"));
        s.addMachine("m2", caps("x"));

        // lexicographic: "m10" < "m2" < "m9"
        assertThat(s.submitJob("j1", caps("x"), FEWEST)).contains("m10");
        assertThat(s.submitJob("j2", caps("x"), FEWEST)).contains("m2");
        assertThat(s.submitJob("j3", caps("x"), FEWEST)).contains("m9");
    }

    @Test
    void shouldPickTheMachineWithMostFinishedJobs() {
        JobScheduler s = scheduler();
        s.addMachine("a", caps("x"));
        s.addMachine("b", caps("x"));
        s.submitJob("j1", caps("x"), FEWEST); // a
        s.submitJob("j2", caps("x"), FEWEST); // b
        s.submitJob("j3", caps("x"), FEWEST); // a
        s.submitJob("j4", caps("x"), FEWEST); // b
        s.finishJob("j2");
        s.finishJob("j4");
        s.finishJob("j1");

        assertThat(s.finishedCount("a")).isEqualTo(1);
        assertThat(s.finishedCount("b")).isEqualTo(2);
        assertThat(s.submitJob("j5", caps("x"), MOST)).contains("b");
    }

    @Test
    void shouldBreakMostFinishedTiesBySmallestMachineId() {
        JobScheduler s = scheduler();
        s.addMachine("zeta", caps("x"));
        s.addMachine("alpha", caps("x"));
        s.addMachine("mid", caps("x"));

        assertThat(s.submitJob("j1", caps("x"), MOST)).contains("alpha");
        // MOST_FINISHED ignores load: all three still tie at 0 finished
        assertThat(s.submitJob("j2", caps("x"), MOST)).contains("alpha");
        assertThat(s.unfinishedCount("alpha")).isEqualTo(2);
    }

    @Test
    void shouldMoveAJobFromUnfinishedToFinishedWhenItCompletes() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("x"));
        s.submitJob("j1", caps("x"), FEWEST);
        s.submitJob("j2", caps("x"), FEWEST);

        s.finishJob("j1");

        assertThat(s.unfinishedCount("m1")).isEqualTo(1);
        assertThat(s.finishedCount("m1")).isEqualTo(1);
        s.finishJob("j2");
        assertThat(s.unfinishedCount("m1")).isEqualTo(0);
        assertThat(s.finishedCount("m1")).isEqualTo(2);
    }

    @Test
    void shouldRankOnlyAmongEligibleMachines() {
        JobScheduler s = scheduler();
        s.addMachine("idle", caps("cpu"));
        s.addMachine("busy", caps("cpu", "gpu"));
        s.submitJob("j1", caps("gpu"), FEWEST); // busy (only eligible)
        s.submitJob("j2", caps("gpu"), FEWEST); // busy

        assertThat(s.unfinishedCount("idle")).isEqualTo(0);
        assertThat(s.unfinishedCount("busy")).isEqualTo(2);
        assertThat(s.submitJob("j3", caps("gpu"), FEWEST)).contains("busy");
        assertThat(s.submitJob("j4", caps("cpu"), FEWEST)).contains("idle");
    }

    @Test
    void shouldRankMostFinishedOnlyAmongEligibleMachines() {
        JobScheduler s = scheduler();
        s.addMachine("veteran", caps("cpu"));
        s.addMachine("novice", caps("cpu", "gpu"));
        s.submitJob("j1", caps("cpu"), FEWEST); // novice? tie at 0 -> "novice" < "veteran"
        assertThat(s.machineOf("j1")).isEqualTo("novice");
        s.submitJob("j2", caps("cpu"), FEWEST); // veteran
        s.finishJob("j2");

        assertThat(s.submitJob("j3", caps("cpu"), MOST)).contains("veteran");
        assertThat(s.submitJob("j4", caps("gpu"), MOST)).contains("novice");
    }

    @Test
    void shouldUseFewestUnfinishedAsTheInitialDefaultCriterion() {
        JobScheduler s = scheduler();
        s.addMachine("a", caps("x"));
        s.addMachine("b", caps("x"));

        assertThat(s.submitJob("j1", caps("x"))).contains("a");
        assertThat(s.submitJob("j2", caps("x"))).contains("b");
        assertThat(s.submitJob("j3", caps("x"))).contains("a");
    }

    @Test
    void shouldHonourAChangedDefaultCriterionAndLetACallOverrideIt() {
        JobScheduler s = scheduler();
        s.addMachine("a", caps("x"));
        s.addMachine("b", caps("x"));
        s.submitJob("j1", caps("x"), FEWEST); // a
        s.submitJob("j2", caps("x"), FEWEST); // b
        s.finishJob("j2");                    // b has 1 finished, 0 unfinished; a has 1 unfinished

        s.setDefaultCriterion(MOST);
        assertThat(s.submitJob("j3", caps("x"))).contains("b");
        assertThat(s.submitJob("j4", caps("x"), FEWEST)).contains("a"); // b:1 unfinished, a:1 -> tie -> a
        s.setDefaultCriterion(FEWEST);
        assertThat(s.submitJob("j5", caps("x"))).contains("b"); // a:2 unfinished, b:1
    }

    @Test
    void shouldAssignMachinesAddedAfterJobsWereSubmitted() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("x"));
        assertThat(s.submitJob("j1", caps("gpu"), FEWEST)).isEmpty();

        s.addMachine("m2", caps("x", "gpu"));

        assertThat(s.submitJob("j2", caps("gpu"), FEWEST)).contains("m2");
        assertThat(s.submitJob("j3", caps("x"), FEWEST)).contains("m1");
    }

    @Test
    void shouldNotRecordAJobThatFoundNoMachine() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("x"));

        assertThat(s.submitJob("j1", caps("gpu"), FEWEST)).isEmpty();
        assertThatThrownBy(() -> s.finishJob("j1")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.machineOf("j1")).isInstanceOf(IllegalArgumentException.class);

        s.addMachine("m2", caps("gpu"));
        assertThat(s.submitJob("j1", caps("gpu"), FEWEST)).contains("m2"); // id is reusable after an empty result
    }

    @Test
    void shouldNotBeAffectedByLaterChangesToTheCallersCapabilitySets() {
        JobScheduler s = scheduler();
        Set<String> offered = new HashSet<>(Set.of("x"));
        s.addMachine("m1", offered);
        offered.add("gpu");
        Set<String> required = new HashSet<>(Set.of("x"));

        assertThat(s.submitJob("j1", required, FEWEST)).contains("m1");
        required.add("gpu");
        assertThat(s.submitJob("j2", Set.of("gpu"), FEWEST)).isEmpty();
    }

    @Test
    void shouldRejectDuplicateAndInvalidMachines() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("x"));

        assertThatThrownBy(() -> s.addMachine("m1", caps("y"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addMachine(" ", caps("y"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addMachine(null, caps("y"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addMachine("m2", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addMachine("m3", Set.of(" "))).isInstanceOf(IllegalArgumentException.class);
        // the failed duplicate must not have replaced the original capabilities
        assertThat(s.submitJob("j1", caps("x"), FEWEST)).contains("m1");
        assertThat(s.submitJob("j2", caps("y"), FEWEST)).isEmpty();
        // failed adds registered nothing
        assertThatThrownBy(() -> s.unfinishedCount("m2")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.unfinishedCount("m3")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectInvalidSubmissionsAndChangeNothing() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("x"));

        assertThatThrownBy(() -> s.submitJob(null, caps("x"), FEWEST)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.submitJob(" ", caps("x"), FEWEST)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.submitJob("j1", null, FEWEST)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.submitJob("j1", Set.of(" "), FEWEST)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.submitJob("j1", caps("x"), "ROUND_ROBIN")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.submitJob("j1", caps("x"), null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.submitJob("j1", caps("x"), "fewest_unfinished"))
                .isInstanceOf(IllegalArgumentException.class);

        assertThat(s.unfinishedCount("m1")).isEqualTo(0);
        assertThatThrownBy(() -> s.finishJob("j1")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectAJobIdThatIsAlreadyAssigned() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("x"));
        s.addMachine("m2", caps("x"));
        s.submitJob("j1", caps("x"), FEWEST); // m1

        assertThatThrownBy(() -> s.submitJob("j1", caps("x"), FEWEST)).isInstanceOf(IllegalArgumentException.class);
        s.finishJob("j1");
        assertThatThrownBy(() -> s.submitJob("j1", caps("x"), FEWEST)).isInstanceOf(IllegalArgumentException.class);

        assertThat(s.unfinishedCount("m1")).isEqualTo(0);
        assertThat(s.unfinishedCount("m2")).isEqualTo(0);
        assertThat(s.finishedCount("m1")).isEqualTo(1);
    }

    @Test
    void shouldRejectAnUnknownDefaultCriterionAndKeepThePreviousOne() {
        JobScheduler s = scheduler();
        s.addMachine("a", caps("x"));
        s.addMachine("b", caps("x"));
        s.submitJob("j1", caps("x"), FEWEST); // a

        assertThatThrownBy(() -> s.setDefaultCriterion("bogus")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.setDefaultCriterion(null)).isInstanceOf(IllegalArgumentException.class);

        assertThat(s.submitJob("j2", caps("x"))).contains("b"); // still FEWEST_UNFINISHED
    }

    @Test
    void shouldRejectFinishingAnUnknownJobOrFinishingTwice() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("x"));
        s.submitJob("j1", caps("x"), FEWEST);
        s.submitJob("j2", caps("x"), FEWEST);

        assertThatThrownBy(() -> s.finishJob("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.finishJob(null)).isInstanceOf(IllegalArgumentException.class);
        s.finishJob("j1");
        assertThatThrownBy(() -> s.finishJob("j1")).isInstanceOf(IllegalStateException.class);

        assertThat(s.unfinishedCount("m1")).isEqualTo(1);
        assertThat(s.finishedCount("m1")).isEqualTo(1);
    }

    @Test
    void shouldRejectCountsForUnknownMachines() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("x"));

        assertThatThrownBy(() -> s.unfinishedCount("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.finishedCount("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.unfinishedCount(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.machineOf("nope")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldBalanceExactlyWhenManyThreadsSubmitAtOnce() {
        JobScheduler s = scheduler();
        for (String m : List.of("m1", "m2", "m3", "m4")) {
            s.addMachine(m, caps("x"));
        }

        Concurrently.run(16, t -> {
            for (int i = 0; i < 250; i++) {
                assertThat(s.submitJob("j-" + t + "-" + i, caps("x"), FEWEST)).isPresent();
            }
        });

        // 4000 jobs over 4 machines picked atomically by "fewest unfinished": no machine can drift ahead.
        for (String m : List.of("m1", "m2", "m3", "m4")) {
            assertThat(s.unfinishedCount(m)).isEqualTo(1000);
        }
    }

    @Test
    void shouldLetExactlyOneThreadFinishAJob() {
        JobScheduler s = scheduler();
        s.addMachine("m1", caps("x"));
        for (int i = 0; i < 200; i++) {
            s.submitJob("j" + i, caps("x"), FEWEST);
        }
        AtomicInteger ok = new AtomicInteger();
        AtomicInteger rejected = new AtomicInteger();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 200; i++) {
                try {
                    s.finishJob("j" + i);
                    ok.incrementAndGet();
                } catch (IllegalStateException e) {
                    rejected.incrementAndGet();
                }
            }
        });

        assertThat(ok.get()).isEqualTo(200);
        assertThat(rejected.get()).isEqualTo(1400);
        assertThat(s.unfinishedCount("m1")).isEqualTo(0);
        assertThat(s.finishedCount("m1")).isEqualTo(200);
    }

    @Test
    void shouldKeepCountsConsistentWhenSubmittingAndFinishingConcurrently() {
        JobScheduler s = scheduler();
        s.addMachine("a", caps("x"));
        s.addMachine("b", caps("x"));

        Concurrently.run(8, t -> {
            for (int i = 0; i < 300; i++) {
                String id = "j-" + t + "-" + i;
                Optional<String> m = s.submitJob(id, caps("x"), i % 2 == 0 ? FEWEST : MOST);
                assertThat(m).isPresent();
                if (i % 3 == 0) {
                    s.finishJob(id);
                }
            }
        });

        int total = s.unfinishedCount("a") + s.unfinishedCount("b") + s.finishedCount("a") + s.finishedCount("b");
        assertThat(total).isEqualTo(8 * 300);
        assertThat(s.finishedCount("a") + s.finishedCount("b")).isEqualTo(8 * 100);
    }
}
