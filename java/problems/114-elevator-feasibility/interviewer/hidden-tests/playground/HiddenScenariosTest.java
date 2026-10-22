package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.LiftController;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private LiftController lift(int floors, int capacity, int maxStops) {
        return Candidate.of(() -> Solution.create(floors, capacity, maxStops));
    }

    @Test
    void shouldCountAStopAtTheCurrentFloorAsAnExtraStopForOnboardRiders() {
        LiftController l = lift(10, 4, 0);
        l.request("r1", 0, 9, 1);
        l.step(); // at 0, r1 onboard

        // doors would reopen at 0 once more: r1 would see an intermediate stop
        assertThat(l.request("r2", 0, 9, 1)).isFalse();
        assertThat(l.plan()).containsExactly(9);
    }

    @Test
    void shouldKeepPromisesAcrossManyAcceptedRequests() {
        // a rider accepted earlier is never degraded by later acceptances, whatever order they come in
        LiftController l = lift(12, 6, 2);
        Random rnd = new Random(7);
        Map<String, int[]> accepted = new HashMap<>();
        for (int i = 0; i < 60; i++) {
            int from = rnd.nextInt(12);
            int to = rnd.nextInt(12);
            if (from == to) {
                continue;
            }
            String id = "r" + i;
            if (l.request(id, from, to, 1 + rnd.nextInt(3))) {
                accepted.put(id, new int[] {from, to});
            }
            if (i % 7 == 0) {
                l.step();
            }
        }
        assertThat(accepted).isNotEmpty();

        List<Integer> planned = l.plan();
        List<Integer> executed = new ArrayList<>();
        for (int s = l.step(); s != -1; s = l.step()) {
            executed.add(s);
            assertThat(l.onboard()).isLessThanOrEqualTo(6);
        }
        assertThat(executed).isEqualTo(planned);
        accepted.keySet().forEach(id -> assertThat(l.status(id)).isEqualTo("DELIVERED"));
    }

    @Test
    void shouldLeaveThePlanByteForByteIdenticalAfterManyRejections() {
        LiftController l = lift(10, 4, 0);
        l.request("a", 0, 5, 4);
        l.request("b", 5, 9, 4);
        l.step();
        List<Integer> before = l.plan();
        int floor = l.currentFloor();
        int load = l.onboard();

        for (int i = 0; i < 100; i++) {
            assertThat(l.request("x" + i, 1 + i % 4, 2 + i % 4, 1 + i % 4)).isFalse();
        }

        assertThat(l.plan()).isEqualTo(before);
        assertThat(l.currentFloor()).isEqualTo(floor);
        assertThat(l.onboard()).isEqualTo(load);
        for (int i = 0; i < 100; i++) {
            assertThat(l.status("x" + i)).isEqualTo("UNKNOWN");
        }
    }

    @Test
    void shouldFreeCapacityAfterRidersAreDelivered() {
        LiftController l = lift(10, 4, 0);
        for (int i = 0; i < 4; i++) {
            assertThat(l.request("a" + i, 0, 5, 1)).isTrue();
        }
        assertThat(l.request("late", 0, 5, 1)).isFalse();

        assertThat(l.step()).isZero();
        assertThat(l.step()).isEqualTo(5);

        assertThat(l.request("late", 0, 5, 4)).isTrue();
        assertThat(l.plan()).containsExactly(0, 5);
    }

    @Test
    void shouldWaitAFullLapForARiderWhoseDirectionIsAlreadyPast() {
        LiftController l = lift(10, 4, 3);
        l.request("down", 7, 1, 1);
        l.step(); // 7, heading down
        l.step(); // 1, delivered; the lift is on its way down at floor 1

        l.request("up", 4, 6, 1);
        l.request("down2", 8, 3, 1);

        // from floor 1 heading down: bottom turn, up through 4 -> 6, top turn, down through 8 -> 3
        assertThat(l.plan()).containsExactly(4, 6, 8, 3);
    }

    @Test
    void shouldRejectWhenALaterBoardingOverlapsAWaitingRiderOnTheReturnSweep() {
        LiftController l = lift(10, 4, 5);
        assertThat(l.request("d1", 9, 0, 3)).isTrue();

        assertThat(l.request("d2", 6, 2, 2)).isFalse(); // d1 still inside (3 + 2)
        assertThat(l.request("d3", 6, 2, 1)).isTrue();
        assertThat(l.plan()).containsExactly(9, 6, 2, 0);
    }

    @Test
    @Timeout(60)
    void shouldHandleATallBuildingWithManyRequests() {
        LiftController l = lift(500, 20, 400);
        Random rnd = new Random(11);
        int accepted = 0;
        for (int i = 0; i < 300; i++) {
            int from = rnd.nextInt(500);
            int to = rnd.nextInt(500);
            if (from != to && l.request("t" + i, from, to, 1)) {
                accepted++;
            }
        }
        assertThat(accepted).isGreaterThan(50);

        int stops = 0;
        for (int s = l.step(); s != -1; s = l.step()) {
            stops++;
        }
        assertThat(stops).isGreaterThan(0).isLessThanOrEqualTo(2 * accepted);
        assertThat(l.onboard()).isZero();
    }

    @Test
    void shouldKeepTheCapacityPromiseWhileRequestsAndStepsRace() {
        LiftController l = lift(20, 5, 6);
        AtomicBoolean overloaded = new AtomicBoolean();
        AtomicInteger acceptedCount = new AtomicInteger();
        List<String> acceptedIds = java.util.Collections.synchronizedList(new ArrayList<>());

        Concurrently.run(6, t -> {
            if (t == 5) {
                for (int i = 0; i < 3_000; i++) {
                    l.step();
                    if (l.onboard() > 5) {
                        overloaded.set(true);
                    }
                }
                return;
            }
            Random rnd = new Random(100 + t);
            for (int i = 0; i < 100; i++) {
                int from = rnd.nextInt(20);
                int to = rnd.nextInt(20);
                if (from == to) {
                    continue;
                }
                String id = "r-" + t + "-" + i;
                if (l.request(id, from, to, 1 + rnd.nextInt(2))) {
                    acceptedCount.incrementAndGet();
                    acceptedIds.add(id);
                }
            }
        });

        assertThat(overloaded.get()).isFalse();
        assertThat(acceptedCount.get()).isGreaterThan(0);
        while (l.step() != -1) {
            assertThat(l.onboard()).isLessThanOrEqualTo(5);
        }
        acceptedIds.forEach(id -> assertThat(l.status(id)).isEqualTo("DELIVERED"));
    }
}
