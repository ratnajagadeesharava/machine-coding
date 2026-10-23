package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.LiftController;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    /** 10 floors (0..9), capacity 4. */
    private LiftController lift(int maxStops) {
        return Candidate.of(() -> Solution.create(10, 4, maxStops));
    }

    private LiftController lift(int floors, int capacity, int maxStops) {
        return Candidate.of(() -> Solution.create(floors, capacity, maxStops));
    }

    private static List<Integer> drain(LiftController l) {
        List<Integer> stops = new java.util.ArrayList<>();
        for (int s = l.step(); s != -1; s = l.step()) {
            stops.add(s);
        }
        return stops;
    }

    @Test
    void shouldStartIdleAtTheGroundFloor() {
        LiftController l = lift(2);

        assertThat(l.currentFloor()).isZero();
        assertThat(l.onboard()).isZero();
        assertThat(l.plan()).isEmpty();
        assertThat(l.step()).isEqualTo(-1);
        assertThat(l.currentFloor()).isZero();
    }

    @Test
    void shouldAcceptASingleRequestAndPlanItsPickupAndDropOff() {
        LiftController l = lift(0);

        assertThat(l.request("r1", 0, 5, 1)).isTrue();

        assertThat(l.plan()).containsExactly(0, 5);
    }

    @Test
    void shouldServeADownwardRideOnTheDownwardSweep() {
        LiftController l = lift(0);
        assertThat(l.request("r1", 6, 2, 1)).isTrue();

        assertThat(l.plan()).containsExactly(6, 2);
        assertThat(l.step()).isEqualTo(6);
        assertThat(l.currentFloor()).isEqualTo(6);
        assertThat(l.onboard()).isEqualTo(1);
        assertThat(l.step()).isEqualTo(2);
        assertThat(l.currentFloor()).isEqualTo(2);
        assertThat(l.onboard()).isZero();
        assertThat(l.step()).isEqualTo(-1);
    }

    @Test
    void shouldReportTheLifecycleOfARequest() {
        LiftController l = lift(0);
        assertThat(l.status("r1")).isEqualTo("UNKNOWN");

        l.request("r1", 0, 5, 2);
        assertThat(l.status("r1")).isEqualTo("WAITING");
        l.step();
        assertThat(l.status("r1")).isEqualTo("ONBOARD");
        assertThat(l.onboard()).isEqualTo(2);
        l.step();
        assertThat(l.status("r1")).isEqualTo("DELIVERED");
        assertThat(l.onboard()).isZero();
    }

    @Test
    void shouldReturnExactlyThePlannedStopsWhenStepping() {
        LiftController l = lift(5);
        l.request("a", 0, 7, 1);
        l.request("b", 3, 5, 1);
        l.request("c", 8, 1, 2);
        List<Integer> planned = l.plan();

        assertThat(planned).isNotEmpty();
        assertThat(drain(l)).isEqualTo(planned);
        assertThat(l.plan()).isEmpty();
    }

    @Test
    void shouldMergeRidersWithTheSameRouteIntoTheSameStops() {
        LiftController l = lift(0);

        assertThat(l.request("r1", 0, 5, 1)).isTrue();
        assertThat(l.request("r2", 0, 5, 1)).isTrue();
        assertThat(l.request("r3", 0, 5, 1)).isTrue();

        assertThat(l.plan()).containsExactly(0, 5);
    }

    @Test
    void shouldRejectARequestWhoseStopsBreakAnExistingRidersLimit() {
        LiftController l = lift(1);
        assertThat(l.request("r1", 0, 9, 1)).isTrue();

        // 3 -> 5 would add two stops (3 and 5) between r1's pickup and drop-off
        assertThat(l.request("r2", 3, 5, 1)).isFalse();
        // 0 -> 5 adds only one (5)
        assertThat(l.request("r3", 0, 5, 1)).isTrue();
        assertThat(l.plan()).containsExactly(0, 5, 9);
        // 0 -> 3 would now give r1 two intermediate stops (3 and 5)
        assertThat(l.request("r4", 0, 3, 1)).isFalse();
    }

    @Test
    void shouldRejectARequestThatWouldBreakItsOwnLimit() {
        LiftController l = lift(1);
        assertThat(l.request("r1", 0, 3, 1)).isTrue();
        assertThat(l.request("r2", 0, 5, 1)).isTrue();

        // r3 would pass stops 3 and 5 on the way to 9 (two intermediate stops); nobody else is hurt
        assertThat(l.request("r3", 0, 9, 1)).isFalse();
        assertThat(l.plan()).containsExactly(0, 3, 5);
    }

    @Test
    void shouldAllowOnlyNonStopRidesWhenTheLimitIsZero() {
        LiftController l = lift(0);
        assertThat(l.request("r1", 0, 5, 1)).isTrue();

        assertThat(l.request("r2", 0, 7, 1)).isFalse(); // would stop at 5 on the way
        assertThat(l.request("r3", 5, 8, 1)).isTrue(); // starts exactly where r1 ends
        assertThat(l.plan()).containsExactly(0, 5, 8);
    }

    @Test
    void shouldRejectWhenCapacityIsExceededAtPickup() {
        LiftController l = lift(4);
        assertThat(l.request("r1", 0, 5, 3)).isTrue();

        assertThat(l.request("r2", 0, 5, 2)).isFalse();
        assertThat(l.request("r3", 0, 5, 1)).isTrue();
        assertThat(l.request("r4", 0, 5, 1)).isFalse();
    }

    @Test
    void shouldRejectWhenRoutesOverlapBeyondCapacityMidRoute() {
        LiftController l = lift(10, 4, 5);
        assertThat(l.request("r1", 0, 8, 3)).isTrue();

        assertThat(l.request("r2", 3, 6, 2)).isFalse(); // 3 + 2 persons between floors 3 and 6
        assertThat(l.request("r3", 3, 6, 1)).isTrue();
        assertThat(l.request("r4", 7, 9, 3)).isFalse(); // r1 is still inside at 7 (3 + 3)
        assertThat(l.request("r5", 8, 9, 4)).isTrue(); // r1 leaves at 8 before r5 boards
    }

    @Test
    void shouldLetRidersLeaveBeforeOthersBoardAtTheSameStop() {
        LiftController l = lift(0);
        assertThat(l.request("r1", 0, 5, 4)).isTrue();

        assertThat(l.request("r2", 5, 8, 4)).isTrue();

        assertThat(l.plan()).containsExactly(0, 5, 8);
    }

    @Test
    void shouldRejectARequestLargerThanTheCapacity() {
        LiftController l = lift(10, 4, 5);

        assertThat(l.request("big", 0, 5, 5)).isFalse();
        assertThat(l.plan()).isEmpty();
        assertThat(l.status("big")).isEqualTo("UNKNOWN");
    }

    @Test
    void shouldLeaveEverythingUnchangedWhenARequestIsRejected() {
        LiftController l = lift(0);
        l.request("r1", 0, 5, 1);
        l.step();
        List<Integer> planBefore = l.plan();

        assertThat(l.request("r2", 2, 8, 1)).isFalse();

        assertThat(l.plan()).isEqualTo(planBefore);
        assertThat(l.status("r2")).isEqualTo("UNKNOWN");
        assertThat(l.status("r1")).isEqualTo("ONBOARD");
        assertThat(l.onboard()).isEqualTo(1);
        assertThat(l.currentFloor()).isZero();
        // the id was never taken, so it can be used again for a request that fits
        assertThat(l.request("r2", 5, 8, 1)).isTrue();
    }

    @Test
    void shouldCountStopsAlreadyMadeAgainstRidersThatAreOnboard() {
        LiftController fresh = lift(1);
        fresh.request("r1", 0, 9, 1);
        assertThat(fresh.request("r3", 6, 9, 1)).isTrue(); // only one stop (6) in between

        LiftController l = lift(1);
        l.request("r1", 0, 9, 1);
        l.request("r2", 0, 4, 1);
        assertThat(l.step()).isEqualTo(0);
        assertThat(l.step()).isEqualTo(4); // r1 has already seen one intermediate stop

        assertThat(l.request("r3", 6, 9, 1)).isFalse();
        assertThat(l.plan()).containsExactly(9);
    }

    @Test
    void shouldPickUpAnUpwardRiderOnTheNextUpwardSweep() {
        LiftController l = lift(5);
        l.request("down", 6, 2, 1);
        assertThat(l.step()).isEqualTo(6); // lift is now at 6 and heading down

        assertThat(l.request("up", 3, 5, 1)).isTrue();

        // continues down to 2, turns at the bottom and picks "up" up at 3 on the way back
        assertThat(l.plan()).containsExactly(2, 3, 5);
    }

    @Test
    void shouldCarryADownwardRiderThatStartsAboveTheLiftAfterTheUpwardSweep() {
        LiftController l = lift(5);
        l.request("up", 0, 3, 1);

        assertThat(l.request("down", 8, 4, 1)).isTrue();

        assertThat(l.plan()).containsExactly(0, 3, 8, 4);
    }

    @Test
    void shouldServeRidersAtTheTopAndBottomFloors() {
        LiftController l = lift(0);

        assertThat(l.request("r1", 0, 9, 1)).isTrue();
        assertThat(l.request("r2", 9, 0, 1)).isTrue();

        assertThat(l.plan()).containsExactly(0, 9, 0);
    }

    @Test
    void shouldBoardAtTheCurrentFloorWhenTheDirectionMatches() {
        LiftController l = lift(2);
        l.request("r1", 0, 5, 1);
        l.step();
        l.step();
        assertThat(l.currentFloor()).isEqualTo(5);

        assertThat(l.request("r2", 5, 8, 1)).isTrue();
        assertThat(l.request("r3", 5, 2, 1)).isTrue();

        assertThat(l.plan()).containsExactly(5, 8, 5, 2);
    }

    @Test
    void shouldRejectInvalidInput() {
        LiftController l = lift(2);

        assertThatThrownBy(() -> l.request("r", -1, 3, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.request("r", 0, 10, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.request("r", 4, 4, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.request("r", 0, 3, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.request("r", 0, 3, -2)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.request(" ", 0, 3, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.request(null, 0, 3, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThat(l.plan()).isEmpty();
    }

    @Test
    void shouldRejectADuplicateIdOfAnActiveRequest() {
        LiftController l = lift(2);
        l.request("r1", 0, 3, 1);

        assertThatThrownBy(() -> l.request("r1", 1, 4, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThat(l.plan()).containsExactly(0, 3);
    }

    @Test
    void shouldAcceptExactlyCapacityWorthOfIdenticalRequestsFromManyThreads() {
        LiftController l = lift(10, 4, 0);
        AtomicInteger accepted = new AtomicInteger();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 5; i++) {
                if (l.request("r-" + t + "-" + i, 0, 5, 1)) {
                    accepted.incrementAndGet();
                }
            }
        });

        assertThat(accepted.get()).isEqualTo(4);
        assertThat(l.plan()).containsExactly(0, 5);
        assertThat(drain(l)).containsExactly(0, 5);
        assertThat(l.onboard()).isZero();
    }

    @Test
    void shouldLoseNoRequestWhenManyThreadsRequestAtOnce() {
        LiftController l = lift(50, 1000, 1000);

        Concurrently.run(8, t -> {
            for (int i = 0; i < 20; i++) {
                assertThat(l.request("r-" + t + "-" + i, t, t + 20, 1)).isTrue();
            }
        });

        drain(l);
        for (int t = 0; t < 8; t++) {
            for (int i = 0; i < 20; i++) {
                assertThat(l.status("r-" + t + "-" + i)).isEqualTo("DELIVERED");
            }
        }
        assertThat(l.onboard()).isZero();
    }
}
