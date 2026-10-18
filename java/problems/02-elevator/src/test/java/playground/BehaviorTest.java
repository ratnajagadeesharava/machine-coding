package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ElevatorApi;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Random;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    private ElevatorApi system(int floors, int elevators) {
        return Candidate.of(() -> Solution.create(floors, elevators));
    }

    /** Steps up to maxSteps and returns the floors at which the given elevator opened its doors, in order. */
    private List<Integer> served(ElevatorApi e, int elevator, int maxSteps) {
        List<Integer> floors = new ArrayList<>();
        for (int i = 0; i < maxSteps; i++) {
            e.step();
            if (e.doorsOpen(elevator)) {
                floors.add(e.floorOf(elevator));
            }
        }
        return floors;
    }

    @Test
    void shouldMoveOneFloorPerStepAndOpenDoorsOnArrival() {
        ElevatorApi e = system(10, 1);
        assertThat(e.requestPickup(3, "UP")).isTrue();

        e.step();
        assertThat(e.floorOf(0)).isEqualTo(1);
        assertThat(e.doorsOpen(0)).isFalse();
        e.step();
        assertThat(e.floorOf(0)).isEqualTo(2);
        e.step();
        assertThat(e.floorOf(0)).isEqualTo(3);
        assertThat(e.doorsOpen(0)).isTrue();
        e.step();
        assertThat(e.floorOf(0)).isEqualTo(3);
        assertThat(e.doorsOpen(0)).isFalse();
    }

    @Test
    void shouldOpenDoorsWithoutMovingForRequestOnCurrentFloor() {
        ElevatorApi e = system(10, 1);
        assertThat(e.requestDestination(0, 0)).isTrue();

        e.step();

        assertThat(e.floorOf(0)).isZero();
        assertThat(e.doorsOpen(0)).isTrue();
    }

    @Test
    void shouldStopAtIntermediateFloorInTravelDirection() {
        ElevatorApi e = system(10, 1);
        e.requestDestination(0, 5);
        e.requestPickup(3, "UP");

        assertThat(served(e, 0, 12)).containsExactly(3, 5);
    }

    @Test
    void shouldNotStopForOppositeDirectionPickupUntilItReturns() {
        ElevatorApi e = system(10, 1);
        e.requestDestination(0, 5);
        e.requestPickup(3, "DOWN");

        assertThat(served(e, 0, 12)).containsExactly(5, 3);
    }

    @Test
    void shouldPickUpNewRequestsAheadOfMovingElevator() {
        ElevatorApi e = system(10, 1);
        e.requestDestination(0, 5);
        e.step();
        e.step();
        assertThat(e.floorOf(0)).isEqualTo(2);

        e.requestDestination(0, 3);

        assertThat(served(e, 0, 10)).containsExactly(3, 5);
    }

    @Test
    void shouldServeRequestsBehindAfterFinishingCurrentDirection() {
        ElevatorApi e = system(10, 1);
        e.requestDestination(0, 5);
        e.step();
        e.step();

        e.requestDestination(0, 1);

        assertThat(served(e, 0, 12)).containsExactly(5, 1);
    }

    @Test
    void shouldRejectInvalidRequestsWithoutSideEffects() {
        ElevatorApi e = system(10, 2);

        assertThat(e.requestPickup(10, "UP")).isFalse();
        assertThat(e.requestPickup(-1, "UP")).isFalse();
        assertThat(e.requestPickup(3, "SIDEWAYS")).isFalse();
        assertThat(e.requestPickup(0, "DOWN")).isFalse();
        assertThat(e.requestPickup(9, "UP")).isFalse();
        assertThat(e.requestDestination(2, 4)).isFalse();
        assertThat(e.requestDestination(0, 99)).isFalse();

        for (int i = 0; i < 20; i++) {
            e.step();
            assertThat(e.floorOf(0)).isZero();
            assertThat(e.floorOf(1)).isZero();
            assertThat(e.doorsOpen(0)).isFalse();
        }
    }

    @Test
    void shouldNotServeDuplicateRequestTwice() {
        ElevatorApi e = system(10, 1);
        e.requestPickup(3, "UP");
        e.requestPickup(3, "UP");
        e.requestDestination(0, 3);

        assertThat(served(e, 0, 10)).containsExactly(3);
    }

    @Test
    void shouldStayIdleWhenNothingIsRequested() {
        ElevatorApi e = system(10, 3);
        for (int i = 0; i < 50; i++) {
            e.step();
        }
        for (int k = 0; k < 3; k++) {
            assertThat(e.floorOf(k)).isZero();
            assertThat(e.doorsOpen(k)).isFalse();
        }
    }

    @Test
    void shouldSendOnlyOneElevatorToAHallCall() {
        ElevatorApi e = system(10, 2);
        e.requestPickup(5, "UP");

        for (int i = 0; i < 5; i++) {
            e.step();
        }

        int atFive = 0;
        int atZero = 0;
        for (int k = 0; k < 2; k++) {
            if (e.floorOf(k) == 5 && e.doorsOpen(k)) atFive++;
            if (e.floorOf(k) == 0) atZero++;
        }
        assertThat(atFive).isEqualTo(1);
        assertThat(atZero).isEqualTo(1);
    }

    @Test
    void shouldPreferElevatorAlreadyHeadingTowardTheCall() {
        ElevatorApi e = system(12, 2);
        e.requestDestination(0, 8);
        for (int i = 0; i < 3; i++) {
            e.step();
        }
        assertThat(e.floorOf(0)).isEqualTo(3);

        e.requestPickup(5, "UP");
        e.step();
        e.step();

        assertThat(e.floorOf(0)).isEqualTo(5);
        assertThat(e.doorsOpen(0)).isTrue();
        assertThat(e.floorOf(1)).isZero();
    }

    @Test
    void shouldPreferNearbyIdleElevatorOverOneThatAlreadyPassedTheFloor() {
        ElevatorApi e = system(12, 2);
        e.requestDestination(0, 8);
        for (int i = 0; i < 3; i++) {
            e.step();
        }

        e.requestPickup(2, "UP");
        e.step();
        e.step();

        assertThat(e.floorOf(1)).isEqualTo(2);
        assertThat(e.doorsOpen(1)).isTrue();
        assertThat(e.floorOf(0)).isEqualTo(5);
    }

    @Test
    void shouldEventuallyServeEveryRequestUnderRandomLoad() {
        int floors = 15;
        int elevators = 3;
        ElevatorApi e = system(floors, elevators);
        Random rnd = new Random(42);
        Set<Integer> hallFloors = new HashSet<>();
        List<Set<Integer>> cabinFloors = new ArrayList<>();
        List<Set<Integer>> servedAt = new ArrayList<>();
        for (int k = 0; k < elevators; k++) {
            cabinFloors.add(new HashSet<>());
            servedAt.add(new HashSet<>());
        }
        Set<Integer> servedAnywhere = new HashSet<>();

        for (int i = 0; i < 540; i++) {
            if (i < 40) {
                if (rnd.nextBoolean()) {
                    int f = rnd.nextInt(floors);
                    String dir = f == 0 ? "UP" : f == floors - 1 ? "DOWN" : (rnd.nextBoolean() ? "UP" : "DOWN");
                    assertThat(e.requestPickup(f, dir)).isTrue();
                    hallFloors.add(f);
                } else {
                    int k = rnd.nextInt(elevators);
                    int f = rnd.nextInt(floors);
                    assertThat(e.requestDestination(k, f)).isTrue();
                    cabinFloors.get(k).add(f);
                }
            }
            e.step();
            for (int k = 0; k < elevators; k++) {
                if (e.doorsOpen(k)) {
                    servedAt.get(k).add(e.floorOf(k));
                    servedAnywhere.add(e.floorOf(k));
                }
            }
        }

        assertThat(servedAnywhere).containsAll(hallFloors);
        for (int k = 0; k < elevators; k++) {
            assertThat(servedAt.get(k)).containsAll(cabinFloors.get(k));
        }
    }
}
