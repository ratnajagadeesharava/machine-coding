package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ItineraryApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    private ItineraryApi planner() {
        ItineraryApi p = Candidate.of(Solution::create);
        p.createTrip("T", 10, 12, 500);
        p.setTravelMinutes("Hotel", "Museum", 30);
        return p;
    }

    @Test
    void shouldAddActivitiesAndListThemInTimeOrder() {
        ItineraryApi p = planner();

        assertThat(p.addActivity("T", "lunch", "Hotel", 10, 720, 780, 20)).isTrue();
        assertThat(p.addActivity("T", "breakfast", "Hotel", 10, 480, 540, 10)).isTrue();

        assertThat(p.agenda("T", 10)).containsExactly("breakfast", "lunch");
        assertThat(p.agenda("T", 11)).isEmpty();
        assertThat(p.totalCost("T")).isEqualTo(30);
    }

    @Test
    void shouldRefuseActivitiesOutsideTheTripOrWithInvalidData() {
        ItineraryApi p = planner();

        assertThat(p.addActivity("T", "a", "Hotel", 9, 480, 540, 0)).isFalse();
        assertThat(p.addActivity("T", "a", "Hotel", 13, 480, 540, 0)).isFalse();
        assertThat(p.addActivity("T", "a", "Hotel", 10, 540, 540, 0)).isFalse();
        assertThat(p.addActivity("T", "a", "Hotel", 10, 600, 540, 0)).isFalse();
        assertThat(p.addActivity("T", "a", "Hotel", 10, -5, 60, 0)).isFalse();
        assertThat(p.addActivity("T", "a", "Hotel", 10, 1400, 1500, 0)).isFalse();
        assertThat(p.addActivity("T", "a", "Hotel", 10, 480, 540, -1)).isFalse();
        assertThat(p.addActivity("NOPE", "a", "Hotel", 10, 480, 540, 0)).isFalse();
        assertThat(p.agenda("T", 10)).isEmpty();
    }

    @Test
    void shouldRefuseDuplicateActivityIds() {
        ItineraryApi p = planner();
        p.addActivity("T", "a", "Hotel", 10, 480, 540, 5);

        assertThat(p.addActivity("T", "a", "Hotel", 11, 480, 540, 5)).isFalse();
        assertThat(p.totalCost("T")).isEqualTo(5);
    }

    @Test
    void shouldRefuseOverlappingActivitiesOnTheSameDay() {
        ItineraryApi p = planner();
        p.addActivity("T", "a", "Hotel", 10, 480, 600, 0);

        assertThat(p.addActivity("T", "b", "Hotel", 10, 540, 660, 0)).isFalse();
        assertThat(p.addActivity("T", "c", "Hotel", 10, 420, 481, 0)).isFalse();
        assertThat(p.addActivity("T", "d", "Hotel", 10, 500, 520, 0)).isFalse();
        assertThat(p.addActivity("T", "e", "Hotel", 10, 400, 700, 0)).isFalse();
    }

    @Test
    void shouldAllowTouchingActivitiesAtTheSameLocationAndOverlapOnOtherDays() {
        ItineraryApi p = planner();
        p.addActivity("T", "a", "Hotel", 10, 480, 600, 0);

        assertThat(p.addActivity("T", "b", "Hotel", 10, 600, 660, 0)).isTrue();
        assertThat(p.addActivity("T", "c", "Hotel", 10, 420, 480, 0)).isTrue();
        assertThat(p.addActivity("T", "d", "Hotel", 11, 480, 600, 0)).isTrue();
        assertThat(p.agenda("T", 10)).containsExactly("c", "a", "b");
    }

    @Test
    void shouldEnforceTravelTimeBetweenDifferentLocations() {
        ItineraryApi p = planner();
        p.addActivity("T", "a1", "Hotel", 10, 540, 600, 0);

        assertThat(p.addActivity("T", "a2", "Museum", 10, 620, 700, 50)).isFalse();
        assertThat(p.addActivity("T", "a2", "Museum", 10, 630, 700, 50)).isTrue();
        assertThat(p.agenda("T", 10)).containsExactly("a1", "a2");
    }

    @Test
    void shouldApplyTravelTimeSymmetrically() {
        ItineraryApi p = planner();
        p.addActivity("T", "m", "Museum", 10, 600, 660, 0);

        assertThat(p.addActivity("T", "h1", "Hotel", 10, 680, 750, 0)).isFalse(); // gap 20 < 30
        assertThat(p.addActivity("T", "h2", "Hotel", 10, 690, 750, 0)).isTrue();  // gap 30
    }

    @Test
    void shouldCheckBothNeighboursWhenInsertingBetweenTwoActivities() {
        ItineraryApi p = planner();
        p.addActivity("T", "early", "Hotel", 10, 480, 540, 0);
        p.addActivity("T", "late", "Hotel", 10, 700, 760, 0);

        // Museum visit needs 30 min after the hotel and 30 min before the hotel again.
        assertThat(p.addActivity("T", "m1", "Museum", 10, 560, 650, 0)).isFalse(); // 20 after early
        assertThat(p.addActivity("T", "m2", "Museum", 10, 570, 680, 0)).isFalse(); // 20 before late
        assertThat(p.addActivity("T", "m3", "Museum", 10, 570, 670, 0)).isTrue();
    }

    @Test
    void shouldTreatUnspecifiedTravelAsZero() {
        ItineraryApi p = planner();
        p.addActivity("T", "a", "Airport", 10, 480, 540, 0);

        assertThat(p.addActivity("T", "b", "Beach", 10, 540, 600, 0)).isTrue();
    }

    @Test
    void shouldEnforceBudgetAndAllowSpendingItExactly() {
        ItineraryApi p = planner();

        assertThat(p.addActivity("T", "a", "Hotel", 10, 480, 540, 300)).isTrue();
        assertThat(p.addActivity("T", "b", "Hotel", 10, 600, 660, 201)).isFalse();
        assertThat(p.addActivity("T", "c", "Hotel", 10, 600, 660, 200)).isTrue();
        assertThat(p.addActivity("T", "d", "Hotel", 10, 700, 760, 1)).isFalse();
        assertThat(p.totalCost("T")).isEqualTo(500);
    }

    @Test
    void shouldFreeTimeAndBudgetWhenRemovingAnActivity() {
        ItineraryApi p = planner();
        p.addActivity("T", "a", "Hotel", 10, 480, 540, 300);

        assertThat(p.removeActivity("T", "a")).isTrue();

        assertThat(p.totalCost("T")).isZero();
        assertThat(p.agenda("T", 10)).isEmpty();
        assertThat(p.addActivity("T", "b", "Hotel", 10, 480, 540, 500)).isTrue();
    }

    @Test
    void shouldRefuseRemovingUnknownActivitiesOrFromUnknownTrips() {
        ItineraryApi p = planner();

        assertThat(p.removeActivity("T", "ghost")).isFalse();
        assertThat(p.removeActivity("NOPE", "a")).isFalse();
        assertThat(p.totalCost("NOPE")).isZero();
        assertThat(p.agenda("NOPE", 10)).isEmpty();
    }

    @Test
    void shouldKeepTripsIndependent() {
        ItineraryApi p = planner();
        p.createTrip("U", 10, 12, 100);

        p.addActivity("T", "a", "Hotel", 10, 480, 540, 50);

        assertThat(p.addActivity("U", "a", "Hotel", 10, 480, 540, 60)).isTrue();
        assertThat(p.totalCost("T")).isEqualTo(50);
        assertThat(p.totalCost("U")).isEqualTo(60);
    }

    @Test
    void shouldHandleManyActivitiesPerDay() {
        ItineraryApi p = Candidate.of(Solution::create);
        p.createTrip("BIG", 1, 60, 1_000_000);
        for (int day = 1; day <= 60; day++) {
            for (int i = 0; i < 30; i++) {
                assertThat(p.addActivity("BIG", "d" + day + "-" + i, "X", day, i * 40, i * 40 + 30, 1)).isTrue();
            }
        }
        assertThat(p.agenda("BIG", 30)).hasSize(30);
        assertThat(p.agenda("BIG", 30).get(0)).isEqualTo("d30-0");
        assertThat(p.totalCost("BIG")).isEqualTo(1800);
    }
}
