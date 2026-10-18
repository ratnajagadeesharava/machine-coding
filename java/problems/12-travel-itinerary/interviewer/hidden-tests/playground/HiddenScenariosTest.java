package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ItineraryApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    @Test
    void removalThatWouldBreakTravelTimeBetweenNewNeighboursShouldBeRefused() {
        ItineraryApi p = Candidate.of(Solution::create);
        p.createTrip("T", 1, 1, 1000);
        p.setTravelMinutes("X", "Y", 30);
        p.setTravelMinutes("Y", "Z", 30);
        p.setTravelMinutes("X", "Z", 180);
        assertThat(p.addActivity("T", "A", "X", 1, 540, 600, 0)).isTrue();
        assertThat(p.addActivity("T", "B", "Y", 1, 630, 690, 0)).isTrue();
        assertThat(p.addActivity("T", "C", "Z", 1, 720, 780, 0)).isTrue();

        assertThat(p.removeActivity("T", "B")).isFalse();

        assertThat(p.agenda("T", 1)).containsExactly("A", "B", "C");
    }

    @Test
    void removalThatKeepsConsecutiveActivitiesFeasibleShouldSucceed() {
        ItineraryApi p = Candidate.of(Solution::create);
        p.createTrip("T", 1, 1, 1000);
        p.setTravelMinutes("X", "Z", 100);
        p.addActivity("T", "A", "X", 1, 540, 600, 0);
        p.addActivity("T", "B", "X", 1, 600, 650, 0);
        p.addActivity("T", "C", "Z", 1, 760, 800, 0);

        assertThat(p.removeActivity("T", "B")).isTrue();
        assertThat(p.agenda("T", 1)).containsExactly("A", "C");
    }

    @Test
    void touchingActivitiesAtDifferentLocationsNeedZeroTravelTimeToBeAllowed() {
        ItineraryApi p = Candidate.of(Solution::create);
        p.createTrip("T", 1, 1, 1000);
        p.setTravelMinutes("X", "Y", 1);
        p.addActivity("T", "A", "X", 1, 540, 600, 0);

        assertThat(p.addActivity("T", "B", "Y", 1, 600, 660, 0)).isFalse();
        assertThat(p.addActivity("T", "B", "Y", 1, 601, 660, 0)).isTrue();
    }

    @Test
    void travelTimeSetAfterPlanningShouldNotInvalidateExistingPlans() {
        ItineraryApi p = Candidate.of(Solution::create);
        p.createTrip("T", 1, 1, 1000);
        p.addActivity("T", "A", "X", 1, 540, 600, 0);
        p.addActivity("T", "B", "Y", 1, 600, 660, 0);
        p.setTravelMinutes("X", "Y", 120);

        assertThat(p.agenda("T", 1)).containsExactly("A", "B");
        assertThat(p.addActivity("T", "C", "Y", 1, 700, 720, 0)).isTrue();
    }

    @Test
    void shouldCheckBudgetBeforeMutating() {
        ItineraryApi p = Candidate.of(Solution::create);
        p.createTrip("T", 1, 1, 10);
        assertThat(p.addActivity("T", "A", "X", 1, 540, 600, 11)).isFalse();
        assertThat(p.agenda("T", 1)).isEmpty();
        assertThat(p.addActivity("T", "A", "X", 1, 540, 600, 10)).isTrue();
    }
}
