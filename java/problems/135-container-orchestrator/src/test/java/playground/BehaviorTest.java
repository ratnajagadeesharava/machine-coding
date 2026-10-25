package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Orchestrator;
import contract.Strategy;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static Orchestrator orch(Strategy strategy) {
        return Candidate.of(() -> Solution.create(strategy));
    }

    private static Orchestrator firstFit() {
        return orch(Strategy.FIRST_FIT);
    }

    // ---------------------------------------------------------------- strategies

    @Test
    void shouldPlaceOnTheNodeWithTheSmallestIdThatFitsWithFirstFit() {
        Orchestrator o = firstFit();
        o.addNode("b", 1000, 1000);
        o.addNode("a", 1000, 1000);

        assertThat(o.submit("c1", 100, 100, 1)).contains("a");
        assertThat(o.submit("c2", 100, 100, 1)).contains("a");
        assertThat(o.placement("c2")).isEqualTo("a");
    }

    @Test
    void shouldSkipANodeWithEnoughCpuButNotEnoughMemory() {
        Orchestrator o = firstFit();
        o.addNode("a", 1000, 100);
        o.addNode("b", 500, 500);

        assertThat(o.submit("c", 200, 200, 1)).contains("b");
    }

    @Test
    void shouldPickTheTightestNodeWithBestFitAndTheEmptiestWithWorstFit() {
        Orchestrator best = orch(Strategy.BEST_FIT);
        Orchestrator worst = orch(Strategy.WORST_FIT);
        for (Orchestrator o : List.of(best, worst)) {
            o.addNode("n1", 1000, 1000);
            o.addNode("n2", 400, 400);
            o.addNode("n3", 2000, 2000);
        }

        assertThat(best.submit("c", 300, 300, 1)).contains("n2");
        assertThat(worst.submit("c", 300, 300, 1)).contains("n3");
    }

    @Test
    void shouldMeasureLeftoverAsTheSumOfFreeCpuAndFreeMemoryAfterPlacement() {
        Orchestrator best = orch(Strategy.BEST_FIT);
        Orchestrator worst = orch(Strategy.WORST_FIT);
        for (Orchestrator o : List.of(best, worst)) {
            o.addNode("n1", 1000, 100); // leftover for (100,50): 900 + 50 = 950
            o.addNode("n2", 500, 500); //  leftover for (100,50): 400 + 450 = 850
        }

        assertThat(best.submit("c", 100, 50, 1)).contains("n2");
        assertThat(worst.submit("c", 100, 50, 1)).contains("n1");
    }

    @Test
    void shouldBreakStrategyTiesByNodeIdAscending() {
        for (Strategy s : Strategy.values()) {
            Orchestrator o = orch(s);
            o.addNode("n2", 500, 500);
            o.addNode("n1", 500, 500);
            o.addNode("n3", 500, 500);

            assertThat(o.submit("c", 100, 100, 1)).as(s.name()).contains("n1");
        }
    }

    @Test
    void shouldApplyAStrategyChangeToLaterSubmissionsOnly() {
        Orchestrator o = firstFit();
        o.addNode("n1", 1000, 1000);
        o.addNode("n2", 400, 400);
        assertThat(o.submit("a", 300, 300, 1)).contains("n1");

        o.setStrategy(Strategy.BEST_FIT);

        assertThat(o.placement("a")).isEqualTo("n1"); // nothing moves
        assertThat(o.submit("b", 50, 50, 1)).contains("n2"); // leftover 700 on n2 vs 1300 on n1
        o.setStrategy(Strategy.WORST_FIT);
        assertThat(o.submit("c", 50, 50, 1)).contains("n1");
    }

    @Test
    void shouldAcceptAContainerThatFitsExactly() {
        Orchestrator o = firstFit();
        o.addNode("n", 500, 500);

        assertThat(o.submit("exact", 500, 500, 1)).contains("n");
        assertThat(o.nodeUsage("n")).isEqualTo("500/500,500/500");
        assertThat(o.submit("one", 1, 1, 1)).isEmpty();
    }

    // ---------------------------------------------------------------- pending and retry

    @Test
    void shouldKeepAContainerThatFitsNowhereAsPending() {
        Orchestrator o = firstFit();
        assertThat(o.submit("early", 10, 10, 1)).isEmpty(); // no nodes at all
        o.addNode("n", 100, 100);
        assertThat(o.placement("early")).isEqualTo("n"); // retried when the node arrived

        assertThat(o.submit("a", 60, 60, 5)).contains("n");
        assertThat(o.submit("b", 60, 60, 5)).isEmpty();
        assertThat(o.placement("b")).isEqualTo("PENDING");
        assertThat(o.pending()).containsExactly("b");
    }

    @Test
    void shouldRetryPendingContainersWhenCapacityIsFreed() {
        Orchestrator o = firstFit();
        o.addNode("n", 100, 100);
        o.submit("a", 60, 60, 5);
        o.submit("b", 60, 60, 5);

        o.remove("a");

        assertThat(o.placement("b")).isEqualTo("n");
        assertThat(o.pending()).isEmpty();
        assertThat(o.nodeUsage("n")).isEqualTo("60/100,60/100");
    }

    @Test
    void shouldRetryPendingContainersWhenANodeIsAdded() {
        Orchestrator o = firstFit();
        o.addNode("n1", 100, 100);
        o.submit("a", 100, 100, 1);
        o.submit("b", 80, 80, 1);

        o.addNode("n2", 100, 100);

        assertThat(o.placement("b")).isEqualTo("n2");
    }

    @Test
    void shouldRetryHighestPriorityFirstThenOldestSubmission() {
        Orchestrator o = firstFit();
        o.addNode("n", 100, 100);
        o.submit("blocker", 100, 100, 1);
        o.submit("p1", 100, 100, 5);
        o.submit("p2", 100, 100, 9);
        o.submit("p3", 100, 100, 5);
        o.submit("p4", 100, 100, 9);

        assertThat(o.pending()).containsExactly("p2", "p4", "p1", "p3");

        o.remove("blocker");
        assertThat(o.placement("p2")).isEqualTo("n");
        assertThat(o.pending()).containsExactly("p4", "p1", "p3");

        o.remove("p2");
        assertThat(o.placement("p4")).isEqualTo("n");
        o.remove("p4");
        assertThat(o.placement("p1")).isEqualTo("n");
    }

    @Test
    void shouldNotLetAPendingContainerThatStillDoesNotFitBlockSmallerOnes() {
        Orchestrator o = firstFit();
        o.addNode("n1", 100, 100);
        o.addNode("n2", 50, 50);
        o.submit("x1", 100, 100, 1); // n1
        o.submit("x2", 50, 50, 1); //  n2
        o.submit("big", 100, 100, 50);
        o.submit("small", 50, 50, 1);
        assertThat(o.pending()).containsExactly("big", "small");

        o.remove("x2");

        assertThat(o.placement("small")).isEqualTo("n2");
        assertThat(o.pending()).containsExactly("big");
    }

    @Test
    void shouldPlaceANewContainerImmediatelyEvenWhenOthersWaitWithHigherPriority() {
        Orchestrator o = firstFit();
        o.addNode("n", 100, 100);
        o.submit("a", 70, 70, 1);
        assertThat(o.submit("waiting", 50, 50, 90)).isEmpty();

        assertThat(o.submit("small", 30, 30, 1)).contains("n");
        assertThat(o.pending()).containsExactly("waiting");
    }

    @Test
    void shouldRemoveAPendingContainerWithoutTouchingOthers() {
        Orchestrator o = firstFit();
        o.addNode("n", 100, 100);
        o.submit("a", 100, 100, 1);
        o.submit("b", 10, 10, 1);
        o.submit("c", 10, 10, 1);

        o.remove("b");

        assertThat(o.pending()).containsExactly("c");
        assertThatThrownBy(() -> o.placement("b")).isInstanceOf(IllegalArgumentException.class);
        assertThat(o.containersOn("n")).containsExactly("a");
    }

    // ---------------------------------------------------------------- queries

    @Test
    void shouldReportUsageAndSortedContainers() {
        Orchestrator o = firstFit();
        o.addNode("n", 1000, 2048);
        assertThat(o.nodeUsage("n")).isEqualTo("0/1000,0/2048");
        assertThat(o.containersOn("n")).isEmpty();

        o.submit("b", 300, 100, 1);
        o.submit("a", 200, 156, 1);

        assertThat(o.nodeUsage("n")).isEqualTo("500/1000,256/2048");
        assertThat(o.containersOn("n")).containsExactly("a", "b");
        o.remove("a");
        assertThat(o.nodeUsage("n")).isEqualTo("300/1000,100/2048");
    }

    @Test
    void shouldAllowAnIdToBeReusedAfterRemoval() {
        Orchestrator o = firstFit();
        o.addNode("n", 100, 100);
        o.submit("a", 100, 100, 1);
        o.remove("a");

        assertThat(o.submit("a", 20, 20, 1)).contains("n");
        assertThat(o.nodeUsage("n")).isEqualTo("20/100,20/100");
    }

    // ---------------------------------------------------------------- node removal

    @Test
    void shouldRescheduleTheContainersOfARemovedNodeAndReportThemInIdOrder() {
        Orchestrator o = firstFit();
        o.addNode("n1", 100, 100);
        o.addNode("n2", 100, 100);
        o.submit("b", 40, 40, 1); // n1
        o.submit("a", 60, 60, 1); // n1
        o.submit("c", 50, 50, 1); // n2
        o.submit("d", 10, 10, 1); // n2

        List<String> moved = o.removeNode("n1");

        // retry order is b (older) then a; b fits n2's last 40, a no longer fits anywhere
        assertThat(moved).containsExactly("a->PENDING", "b->n2");
        assertThat(o.pending()).containsExactly("a");
        assertThat(o.nodeUsage("n2")).isEqualTo("100/100,100/100");
        assertThatThrownBy(() -> o.nodeUsage("n1")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldGiveScarceSpaceToTheHigherPriorityContainerOfARemovedNode() {
        Orchestrator o = firstFit();
        o.addNode("n1", 100, 100);
        o.addNode("n2", 60, 60);
        o.submit("lo", 60, 60, 1);
        o.submit("hi", 40, 40, 50);

        assertThat(o.removeNode("n1")).containsExactly("hi->n2", "lo->PENDING");
        assertThat(o.pending()).containsExactly("lo");
    }

    @Test
    void shouldReturnAnEmptyListWhenARemovedNodeWasEmptyAndKeepEverythingPendingWithoutNodes() {
        Orchestrator o = firstFit();
        o.addNode("n1", 100, 100);
        o.addNode("n2", 100, 100);
        assertThat(o.removeNode("n2")).isEmpty();

        o.submit("a", 10, 10, 1);
        assertThat(o.removeNode("n1")).containsExactly("a->PENDING");
        assertThat(o.placement("a")).isEqualTo("PENDING");

        o.addNode("n1", 100, 100); // an id may come back after removal
        assertThat(o.placement("a")).isEqualTo("n1");
    }

    // ---------------------------------------------------------------- preemption

    @Test
    void shouldEvictTheLowestPriorityContainerAndOnlyAsManyAsNeeded() {
        Orchestrator o = firstFit();
        o.addNode("n", 1000, 1000);
        o.submit("lowA", 400, 400, 10);
        o.submit("lowB", 400, 400, 20);

        assertThat(o.submit("urgent", 500, 500, 100)).contains("n");

        assertThat(o.placement("lowA")).isEqualTo("PENDING");
        assertThat(o.placement("lowB")).isEqualTo("n");
        assertThat(o.containersOn("n")).containsExactly("lowB", "urgent");
        assertThat(o.pending()).containsExactly("lowA");
    }

    @Test
    void shouldEvictTheNewestOfEqualPriorityVictimsFirst() {
        Orchestrator o = firstFit();
        o.addNode("n", 1000, 1000);
        o.submit("old", 400, 400, 10);
        o.submit("new", 400, 400, 10);

        o.submit("urgent", 500, 500, 100);

        assertThat(o.placement("new")).isEqualTo("PENDING");
        assertThat(o.placement("old")).isEqualTo("n");
    }

    @Test
    void shouldNotPreemptWhenPriorityIsBelow100() {
        Orchestrator o = firstFit();
        o.addNode("n", 100, 100);
        o.submit("low", 100, 100, 1);

        assertThat(o.submit("important", 100, 100, 99)).isEmpty();

        assertThat(o.placement("low")).isEqualTo("n");
        assertThat(o.pending()).containsExactly("important");
    }

    @Test
    void shouldOnlyEvictContainersOfStrictlyLowerPriority() {
        Orchestrator o = firstFit();
        o.addNode("n", 100, 100);
        o.submit("r", 100, 100, 100);

        assertThat(o.submit("same", 50, 50, 100)).isEmpty();
        assertThat(o.placement("r")).isEqualTo("n");

        assertThat(o.submit("above", 50, 50, 101)).contains("n");
        assertThat(o.placement("r")).isEqualTo("PENDING");
    }

    @Test
    void shouldNotPreemptWhenTheContainerFitsSomewhereWithoutIt() {
        Orchestrator o = firstFit();
        o.addNode("n1", 100, 100);
        o.addNode("n2", 100, 100);
        o.submit("low", 100, 100, 1); // n1

        assertThat(o.submit("urgent", 100, 100, 500)).contains("n2");
        assertThat(o.placement("low")).isEqualTo("n1");
    }

    @Test
    void shouldPreemptOnTheNodeNeedingTheFewestEvictionsAndBreakTiesByNodeId() {
        Orchestrator o = firstFit();
        o.addNode("n1", 1000, 1000);
        o.addNode("n2", 1000, 1000);
        o.submit("s1", 300, 300, 1); // n1
        o.submit("s2", 300, 300, 1); // n1
        o.submit("s3", 300, 300, 1); // n1
        o.submit("big", 900, 900, 50); // n2

        // n1 needs two evictions (s3, s2); n2 needs one (big)
        assertThat(o.submit("urgent", 600, 600, 100)).contains("n2");
        assertThat(o.placement("big")).isEqualTo("PENDING");
        assertThat(o.containersOn("n1")).containsExactly("s1", "s2", "s3");

        Orchestrator t = firstFit();
        t.addNode("n1", 1000, 1000);
        t.addNode("n2", 1000, 1000);
        t.submit("x", 900, 900, 1); // n1
        t.submit("y", 900, 900, 1); // n2
        assertThat(t.submit("urgent", 500, 500, 100)).contains("n1");
        assertThat(t.placement("x")).isEqualTo("PENDING");
        assertThat(t.placement("y")).isEqualTo("n2");
    }

    @Test
    void shouldEvictNothingWhenNoSingleNodeCanBeFreedEnough() {
        Orchestrator o = firstFit();
        o.addNode("n1", 1000, 1000);
        o.addNode("n2", 1000, 1000);
        o.submit("guard", 800, 800, 500); // n1, cannot be evicted by priority 100
        o.submit("low", 100, 100, 1); // n1

        // larger than any node: nothing can ever be freed enough
        assertThat(o.submit("huge", 1500, 1500, 100)).isEmpty();
        assertThat(o.submit("mid", 900, 900, 100)).contains("n2"); // fits without eviction
        // n1: evicting "low" frees 200 only (guard outranks the new container); n2: "mid" has equal priority
        assertThat(o.submit("mid2", 500, 500, 100)).isEmpty();

        assertThat(o.placement("low")).isEqualTo("n1");
        assertThat(o.placement("mid")).isEqualTo("n2");
        assertThat(o.pending()).containsExactly("huge", "mid2");
    }

    @Test
    void shouldCheckBothCpuAndMemoryWhenCountingWhatEvictionFrees() {
        Orchestrator o = firstFit();
        o.addNode("n", 1000, 1000);
        o.submit("v1", 100, 800, 1);
        o.submit("v2", 600, 100, 2);

        assertThat(o.submit("u1", 300, 500, 100)).contains("n");
        assertThat(o.placement("v1")).isEqualTo("PENDING"); // memory was the missing resource
        assertThat(o.placement("v2")).isEqualTo("n");

        Orchestrator p = firstFit();
        p.addNode("n", 1000, 1000);
        p.submit("v1", 100, 800, 1);
        p.submit("v2", 600, 100, 2);
        assertThat(p.submit("u2", 700, 500, 100)).contains("n");
        assertThat(p.pending()).containsExactly("v2", "v1"); // both evicted; retried highest priority first
    }

    // ---------------------------------------------------------------- validation

    @Test
    void shouldRejectInvalidNodesAndLeaveStateUnchanged() {
        Orchestrator o = firstFit();
        o.addNode("n", 100, 100);

        assertThatThrownBy(() -> o.addNode("n", 5, 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.addNode(" ", 5, 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.addNode(null, 5, 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.addNode("z", 0, 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.addNode("z", 5, -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.removeNode("nope")).isInstanceOf(IllegalArgumentException.class);

        assertThat(o.nodeUsage("n")).isEqualTo("0/100,0/100");
        assertThatThrownBy(() -> o.nodeUsage("z")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectInvalidContainersAndLeaveStateUnchanged() {
        Orchestrator o = firstFit();
        o.addNode("n", 100, 100);
        o.submit("a", 50, 50, 1);
        o.submit("p", 80, 80, 1); // pending

        assertThatThrownBy(() -> o.submit("a", 1, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.submit("p", 1, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.submit(" ", 1, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.submit(null, 1, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.submit("z", 0, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.submit("z", 1, 0, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.remove("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.placement("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.containersOn("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> o.setStrategy(null)).isInstanceOf(IllegalArgumentException.class);

        assertThat(o.nodeUsage("n")).isEqualTo("50/100,50/100");
        assertThat(o.pending()).containsExactly("p");
        assertThatThrownBy(() -> o.placement("z")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldAcceptNegativePriorities() {
        Orchestrator o = firstFit();
        o.addNode("n", 100, 100);
        o.submit("a", 100, 100, 0);
        o.submit("neg", 100, 100, -5);
        o.submit("zero", 100, 100, 0);

        assertThat(o.pending()).containsExactly("zero", "neg");
    }

    // ---------------------------------------------------------------- concurrency

    @Test
    void shouldNeverOverCommitANodeUnderConcurrentSubmits() {
        Orchestrator o = firstFit();
        for (int i = 0; i < 4; i++) {
            o.addNode("n" + i, 1000, 1000);
        }
        AtomicBoolean done = new AtomicBoolean();
        AtomicInteger violations = new AtomicInteger();
        AtomicInteger placed = new AtomicInteger();
        AtomicInteger finished = new AtomicInteger();

        Concurrently.run(17, t -> {
            if (t == 16) {
                while (!done.get()) {
                    for (int n = 0; n < 4; n++) {
                        if (!withinCapacity(o.nodeUsage("n" + n))) {
                            violations.incrementAndGet();
                        }
                    }
                }
                return;
            }
            try {
                for (int i = 0; i < 100; i++) {
                    Optional<String> r = o.submit("t" + t + "-" + i, 10, 10, 1);
                    if (r.isPresent()) {
                        placed.incrementAndGet();
                    }
                }
            } finally {
                if (finished.incrementAndGet() == 16) {
                    done.set(true);
                }
            }
        });

        assertThat(violations.get()).isZero();
        assertThat(placed.get()).isEqualTo(400); // 4 nodes x 100 containers of 10
        for (int n = 0; n < 4; n++) {
            assertThat(o.nodeUsage("n" + n)).isEqualTo("1000/1000,1000/1000");
            assertThat(o.containersOn("n" + n)).hasSize(100);
        }
        assertThat(o.pending()).hasSize(1200);
    }

    @Test
    void shouldStayConsistentWhenSubmitsAndRemovalsRace() {
        Orchestrator o = orch(Strategy.BEST_FIT);
        for (int i = 0; i < 3; i++) {
            o.addNode("n" + i, 500, 500);
        }

        Concurrently.run(16, t -> {
            for (int i = 0; i < 60; i++) {
                o.submit("t" + t + "-" + i, 10, 10, t % 4);
            }
            for (int i = 0; i < 60; i += 2) {
                o.remove("t" + t + "-" + i);
            }
        });

        int running = 0;
        for (int n = 0; n < 3; n++) {
            List<String> on = o.containersOn("n" + n);
            running += on.size();
            assertThat(o.nodeUsage("n" + n)).isEqualTo(on.size() * 10 + "/500," + on.size() * 10 + "/500");
        }
        // 16 x 30 survivors, 3 x 50 slots
        assertThat(running).isEqualTo(150);
        assertThat(o.pending()).hasSize(16 * 30 - 150);
    }

    private static boolean withinCapacity(String usage) {
        String[] parts = usage.split("[/,]");
        return Integer.parseInt(parts[0]) <= Integer.parseInt(parts[1])
                && Integer.parseInt(parts[2]) <= Integer.parseInt(parts[3]);
    }
}
