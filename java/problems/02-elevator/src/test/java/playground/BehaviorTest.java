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

    /** Counts how often each floor opened its doors, summed over all elevators. */
    private int[] openingsPerFloor(ElevatorApi e, int floors, int elevators, int steps) {
        int[] counts = new int[floors];
        for (int i = 0; i < steps; i++) {
            e.step();
            for (int k = 0; k < elevators; k++) {
                if (e.doorsOpen(k)) counts[e.floorOf(k)]++;
            }
        }
        return counts;
    }

    @Test
    void shouldServeAnOppositeDirectionPickupFromAnIdleElevator() {
        ElevatorApi e = system(10, 1);
        assertThat(e.requestPickup(3, "DOWN")).isTrue();

        assertThat(served(e, 0, 10)).containsExactly(3);
    }

    @Test
    void shouldStayOnTheLastServedFloorWhenIdle() {
        ElevatorApi e = system(10, 1);
        e.requestDestination(0, 5);
        assertThat(served(e, 0, 8)).containsExactly(5);

        for (int i = 0; i < 20; i++) {
            e.step();
            assertThat(e.floorOf(0)).isEqualTo(5);
            assertThat(e.doorsOpen(0)).isFalse();
        }
    }

    @Test
    void shouldServeBehindStopsInDescendingOrderAfterReversing() {
        ElevatorApi e = system(10, 1);
        e.requestDestination(0, 5);
        assertThat(served(e, 0, 8)).containsExactly(5);

        e.requestDestination(0, 1);
        e.requestDestination(0, 4);
        e.requestDestination(0, 3);

        assertThat(served(e, 0, 12)).containsExactly(4, 3, 1);
    }

    @Test
    void shouldServeCabinRequestsAndMatchingHallCallsOnTheWayDownAndTurnAroundForTheRest() {
        ElevatorApi e = system(10, 1);
        e.requestDestination(0, 8);
        assertThat(served(e, 0, 10)).containsExactly(8);

        e.requestPickup(5, "DOWN");
        e.requestPickup(3, "UP");
        e.requestDestination(0, 1);

        assertThat(served(e, 0, 20)).containsExactly(5, 1, 3);
    }

    @Test
    void shouldKeepEarlierPromisesWhenRequestsKeepArrivingMidJourney() {
        ElevatorApi e = system(10, 1);
        e.requestDestination(0, 9);
        for (int i = 0; i < 4; i++) {
            e.step();
        }
        assertThat(e.floorOf(0)).isEqualTo(4);

        assertThat(e.requestDestination(0, 2)).isTrue();
        assertThat(e.requestPickup(6, "DOWN")).isTrue();

        assertThat(served(e, 0, 30)).containsExactly(9, 6, 2);
    }

    @Test
    void shouldServeTheSameFloorAgainWhenItIsRequestedAgainAfterBeingServed() {
        ElevatorApi e = system(10, 1);
        e.requestDestination(0, 3);
        assertThat(served(e, 0, 6)).containsExactly(3);

        e.requestDestination(0, 3);
        e.requestPickup(3, "UP");

        assertThat(served(e, 0, 4)).containsExactly(3);
        assertThat(e.floorOf(0)).isEqualTo(3);
    }

    @Test
    void shouldReopenDoorsWithoutMovingWhenTheCurrentFloorIsRequestedAgainOnceDoorsAreClosed() {
        ElevatorApi e = system(10, 1);
        e.requestDestination(0, 2);
        served(e, 0, 5);
        assertThat(e.doorsOpen(0)).isFalse();

        assertThat(e.requestPickup(2, "UP")).isTrue();
        e.step();

        assertThat(e.floorOf(0)).isEqualTo(2);
        assertThat(e.doorsOpen(0)).isTrue();
    }

    @Test
    void shouldCountOneStopPerRequestedFloorForConsecutiveFloors() {
        ElevatorApi e = system(10, 1);
        e.requestDestination(0, 3);
        e.requestDestination(0, 4);
        e.requestDestination(0, 5);
        e.requestDestination(0, 4);

        assertThat(served(e, 0, 15)).containsExactly(3, 4, 5);
    }

    @Test
    void shouldServeACabinRequestOnlyWithTheElevatorItWasPressedIn() {
        ElevatorApi e = system(10, 3);
        assertThat(e.requestDestination(1, 4)).isTrue();
        assertThat(e.requestDestination(2, 6)).isTrue();

        List<Integer> first = new ArrayList<>();
        List<Integer> second = new ArrayList<>();
        List<Integer> third = new ArrayList<>();
        for (int i = 0; i < 15; i++) {
            e.step();
            if (e.doorsOpen(0)) first.add(e.floorOf(0));
            if (e.doorsOpen(1)) second.add(e.floorOf(1));
            if (e.doorsOpen(2)) third.add(e.floorOf(2));
        }

        assertThat(first).isEmpty();
        assertThat(e.floorOf(0)).isZero();
        assertThat(second).containsExactly(4);
        assertThat(third).containsExactly(6);
    }

    @Test
    void shouldAnswerEveryOverlappingHallCallExactlyOnceAcrossTheBank() {
        ElevatorApi e = system(12, 2);
        e.requestPickup(3, "UP");
        e.requestPickup(5, "UP");
        e.requestPickup(7, "UP");
        e.requestPickup(5, "UP");
        e.requestPickup(9, "DOWN");

        int[] counts = openingsPerFloor(e, 12, 2, 40);

        for (int f = 0; f < 12; f++) {
            boolean requested = f == 3 || f == 5 || f == 7 || f == 9;
            assertThat(counts[f]).as("openings at floor %d", f).isEqualTo(requested ? 1 : 0);
        }
    }

    @Test
    void shouldNotSendASecondElevatorForADuplicateHallCallMadeWhileOneIsOnTheWay() {
        ElevatorApi e = system(12, 3);
        e.requestPickup(8, "UP");
        e.step();
        e.step();
        assertThat(e.requestPickup(8, "UP")).isTrue();

        int[] counts = openingsPerFloor(e, 12, 3, 30);

        assertThat(counts[8]).isEqualTo(1);
        int farFromEight = 0;
        for (int k = 0; k < 3; k++) {
            if (e.floorOf(k) == 0) farFromEight++;
        }
        assertThat(farFromEight).isEqualTo(2);
    }

    @Test
    void shouldAcceptTheBoundaryFloorsAndRejectJustOutsideThem() {
        ElevatorApi e = system(10, 2);

        assertThat(e.requestPickup(0, "UP")).isTrue();
        assertThat(e.requestPickup(9, "DOWN")).isTrue();
        assertThat(e.requestDestination(1, 9)).isTrue();
        assertThat(e.requestDestination(1, 0)).isTrue();
        assertThat(e.requestDestination(1, 10)).isFalse();
        assertThat(e.requestDestination(1, -1)).isFalse();
        assertThat(e.requestDestination(-1, 3)).isFalse();
        assertThat(e.requestDestination(0, 10)).isFalse();
        assertThat(e.requestPickup(0, "UP")).isTrue();
    }

    @Test
    void shouldCompareDirectionsCaseSensitively() {
        ElevatorApi e = system(10, 1);

        assertThat(e.requestPickup(4, "up")).isFalse();
        assertThat(e.requestPickup(4, "Down")).isFalse();
        assertThat(e.requestPickup(4, "UP ")).isFalse();
        assertThat(e.requestPickup(4, "")).isFalse();
        assertThat(e.requestPickup(4, null)).isFalse();

        assertThat(served(e, 0, 10)).isEmpty();
        assertThat(e.floorOf(0)).isZero();
    }

    @Test
    void shouldNotBeDisturbedByRejectedRequestsInTheMiddleOfAJourney() {
        ElevatorApi clean = system(10, 2);
        ElevatorApi noisy = system(10, 2);
        for (ElevatorApi api : List.of(clean, noisy)) {
            api.requestDestination(0, 6);
            api.requestPickup(4, "UP");
            api.requestDestination(1, 3);
        }

        for (int i = 0; i < 25; i++) {
            noisy.requestPickup(10, "UP");
            noisy.requestPickup(0, "DOWN");
            noisy.requestPickup(2, "up");
            noisy.requestDestination(5, 2);
            noisy.requestDestination(0, -3);
            clean.step();
            noisy.step();
            for (int k = 0; k < 2; k++) {
                assertThat(noisy.floorOf(k)).as("floor of %d at step %d", k, i).isEqualTo(clean.floorOf(k));
                assertThat(noisy.doorsOpen(k)).as("doors of %d at step %d", k, i).isEqualTo(clean.doorsOpen(k));
            }
        }
    }

    @Test
    void shouldNotAdvanceTimeWhenReadingState() {
        ElevatorApi e = system(10, 1);
        e.requestDestination(0, 4);
        e.step();

        for (int i = 0; i < 10; i++) {
            assertThat(e.floorOf(0)).isEqualTo(1);
            assertThat(e.doorsOpen(0)).isFalse();
        }
        assertThat(served(e, 0, 6)).containsExactly(4);
    }

    @Test
    void shouldServeAFarHallCallWhileNewNearbyRequestsKeepArriving() {
        ElevatorApi e = system(15, 1);
        e.requestPickup(14, "DOWN");
        boolean farServed = false;

        for (int i = 0; i < 120 && !farServed; i++) {
            e.requestDestination(0, i % 3);
            e.step();
            farServed = e.doorsOpen(0) && e.floorOf(0) == 14;
        }

        assertThat(farServed).isTrue();
    }

    @Test
    void shouldKeepEveryElevatorInRangeAndMoveAtMostOneFloorPerStep() {
        int floors = 20;
        int elevators = 4;
        ElevatorApi e = system(floors, elevators);
        Random rnd = new Random(7);
        int[] previous = new int[elevators];

        for (int i = 0; i < 2000; i++) {
            if (rnd.nextInt(3) == 0) {
                int f = rnd.nextInt(floors);
                String dir = f == 0 ? "UP" : f == floors - 1 ? "DOWN" : (rnd.nextBoolean() ? "UP" : "DOWN");
                if (rnd.nextBoolean()) {
                    e.requestPickup(f, dir);
                } else {
                    e.requestDestination(rnd.nextInt(elevators), f);
                }
            }
            e.step();
            for (int k = 0; k < elevators; k++) {
                int now = e.floorOf(k);
                assertThat(now).isBetween(0, floors - 1);
                assertThat(Math.abs(now - previous[k])).isLessThanOrEqualTo(1);
                previous[k] = now;
            }
        }
    }

    @Test
    void shouldBehaveIdenticallyForIdenticalRequestSequences() {
        int floors = 12;
        int elevators = 3;
        ElevatorApi a = system(floors, elevators);
        ElevatorApi b = system(floors, elevators);
        Random rnd = new Random(99);

        for (int i = 0; i < 600; i++) {
            if (rnd.nextInt(4) == 0) {
                int f = rnd.nextInt(floors);
                String dir = f == 0 ? "UP" : f == floors - 1 ? "DOWN" : (rnd.nextBoolean() ? "UP" : "DOWN");
                boolean hall = rnd.nextBoolean();
                int k = rnd.nextInt(elevators);
                assertThat(hall ? a.requestPickup(f, dir) : a.requestDestination(k, f)).isTrue();
                assertThat(hall ? b.requestPickup(f, dir) : b.requestDestination(k, f)).isTrue();
            }
            a.step();
            b.step();
            for (int k = 0; k < elevators; k++) {
                assertThat(b.floorOf(k)).isEqualTo(a.floorOf(k));
                assertThat(b.doorsOpen(k)).isEqualTo(a.doorsOpen(k));
            }
        }
    }

    @Test
    void shouldSimulateAHundredThousandStepsOnALargeBuildingQuickly() {
        int floors = 100;
        int elevators = 16;
        ElevatorApi e = system(floors, elevators);
        Random rnd = new Random(5);
        long start = System.nanoTime();

        for (int i = 0; i < 100_000; i++) {
            if (i % 5 == 0) {
                int f = 1 + rnd.nextInt(floors - 2);
                if (rnd.nextBoolean()) {
                    e.requestPickup(f, rnd.nextBoolean() ? "UP" : "DOWN");
                } else {
                    e.requestDestination(rnd.nextInt(elevators), f);
                }
            }
            e.step();
        }

        assertThat(System.nanoTime() - start).isLessThan(20_000_000_000L);
    }
}
