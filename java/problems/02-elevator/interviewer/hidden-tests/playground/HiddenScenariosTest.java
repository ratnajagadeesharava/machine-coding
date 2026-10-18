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

class HiddenScenariosTest {

    private ElevatorApi system(int floors, int elevators) {
        return Candidate.of(() -> Solution.create(floors, elevators));
    }

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
    void shouldReverseAfterFinishingUpwardTravel() {
        ElevatorApi e = system(20, 1);
        e.requestDestination(0, 10);
        assertThat(served(e, 0, 10)).containsExactly(10);

        e.requestDestination(0, 2);
        assertThat(served(e, 0, 10)).containsExactly(2);
        assertThat(e.floorOf(0)).isEqualTo(2);
    }

    @Test
    void shouldHandleManyElevatorsAndRequests() {
        ElevatorApi e = system(40, 5);
        Random rnd = new Random(7);
        Set<Integer> hall = new HashSet<>();
        Set<Integer> servedAnywhere = new HashSet<>();
        for (int i = 0; i < 100; i++) {
            int f = rnd.nextInt(39);
            e.requestPickup(f, "UP");
            hall.add(f);
            e.step();
            for (int k = 0; k < 5; k++) {
                if (e.doorsOpen(k)) servedAnywhere.add(e.floorOf(k));
            }
        }
        for (int i = 0; i < 2000; i++) {
            e.step();
            for (int k = 0; k < 5; k++) {
                if (e.doorsOpen(k)) servedAnywhere.add(e.floorOf(k));
            }
        }
        assertThat(servedAnywhere).containsAll(hall);
    }

    @Test
    void shouldServeDownwardHallCallFromAboveThenContinue() {
        ElevatorApi e = system(10, 1);
        e.requestPickup(6, "DOWN");
        e.requestDestination(0, 8);
        assertThat(served(e, 0, 20)).containsExactly(8, 6);
    }
}
