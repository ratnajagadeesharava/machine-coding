package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.RideSharingApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    @Test
    void shouldIncludeDriversExactlyAtDistanceTenAndExcludeElevenAway() {
        RideSharingApi s = Candidate.of(Solution::create);
        s.addDriver("edge", 10, 0);
        s.addDriver("beyond", 0, 11);

        String ride = s.requestRide("r1", 0, 0, 1, 1).orElseThrow();

        assertThat(s.driverOf(ride)).hasValue("edge");
        assertThat(s.requestRide("r2", 0, 0, 1, 1)).isEmpty();
    }

    @Test
    void shouldHandleNegativeCoordinatesAndSamePickupAndDrop() {
        RideSharingApi s = Candidate.of(Solution::create);
        s.addDriver("d", -3, -3);
        String ride = s.requestRide("r", -4, -3, -4, -3).orElseThrow();
        s.startRide(ride);

        assertThat(s.completeRide(ride)).hasValue(500L);
        assertThat(s.driverLocation("d")).hasValue("-4,-3");
    }

    @Test
    void ridersWithActiveRidesShouldNotBlockOtherRiders() {
        RideSharingApi s = Candidate.of(Solution::create);
        s.addDriver("a", 0, 0);
        s.addDriver("b", 0, 1);
        s.requestRide("r1", 0, 0, 3, 3);
        assertThat(s.requestRide("r2", 0, 0, 3, 3)).isPresent();
    }

    @Test
    void driverShouldBeMatchedFromTheDropLocationAfterACompletedRide() {
        RideSharingApi s = Candidate.of(Solution::create);
        s.addDriver("d1", 0, 0);
        s.addDriver("d2", 30, 30);
        String ride = s.requestRide("r1", 1, 1, 30, 31).orElseThrow();
        s.startRide(ride);
        s.completeRide(ride);

        String next = s.requestRide("r2", 30, 32, 0, 0).orElseThrow();
        assertThat(s.driverOf(next)).hasValue("d1"); // at (30,31), distance 1; d2 is 2 away
    }
}
