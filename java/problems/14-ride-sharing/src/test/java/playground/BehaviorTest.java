package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.RideSharingApi;
import java.util.Optional;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    private RideSharingApi service() {
        RideSharingApi s = Candidate.of(Solution::create);
        s.addDriver("d1", 0, 0);
        s.addDriver("d2", 3, 0);
        return s;
    }

    @Test
    void shouldMatchTheNearestAvailableDriver() {
        RideSharingApi s = service();

        String ride = s.requestRide("r1", 1, 0, 5, 5).orElseThrow();

        assertThat(s.driverOf(ride)).hasValue("d1");
        assertThat(s.statusOf(ride)).isEqualTo("MATCHED");
    }

    @Test
    void shouldBreakTiesBySmallestDriverId() {
        RideSharingApi s = Candidate.of(Solution::create);
        s.addDriver("d9", 2, 0);
        s.addDriver("d3", 0, 2);
        s.addDriver("d5", 1, 1);

        String ride = s.requestRide("r1", 0, 0, 1, 1).orElseThrow();

        assertThat(s.driverOf(ride)).hasValue("d3");
    }

    @Test
    void shouldUseManhattanDistance() {
        RideSharingApi s = Candidate.of(Solution::create);
        s.addDriver("far-diagonal", 4, 4);   // Manhattan 8, Euclidean ~5.7
        s.addDriver("near-straight", 7, 0);  // Manhattan 7, Euclidean 7

        String ride = s.requestRide("r1", 0, 0, 1, 1).orElseThrow();

        assertThat(s.driverOf(ride)).hasValue("near-straight");
    }

    @Test
    void shouldRefuseWhenNoDriverIsWithinRange() {
        RideSharingApi s = service();

        assertThat(s.requestRide("r1", 50, 50, 0, 0)).isEmpty();
    }

    @Test
    void shouldNotMatchABusyDriver() {
        RideSharingApi s = service();
        s.requestRide("r1", 1, 0, 5, 5);

        String second = s.requestRide("r2", 1, 0, 2, 2).orElseThrow();

        assertThat(s.driverOf(second)).hasValue("d2");
        assertThat(s.requestRide("r3", 1, 0, 2, 2)).isEmpty();
    }

    @Test
    void shouldAllowOnlyOneActiveRidePerRider() {
        RideSharingApi s = service();
        s.requestRide("r1", 1, 0, 5, 5);

        assertThat(s.requestRide("r1", 1, 0, 2, 2)).isEmpty();
    }

    @Test
    void shouldProgressThroughTheRideLifecycle() {
        RideSharingApi s = service();
        String ride = s.requestRide("r1", 1, 0, 5, 5).orElseThrow();

        assertThat(s.completeRide(ride)).isEmpty();      // cannot complete before starting
        assertThat(s.startRide(ride)).isTrue();
        assertThat(s.statusOf(ride)).isEqualTo("IN_PROGRESS");
        assertThat(s.startRide(ride)).isFalse();
        assertThat(s.completeRide(ride)).isPresent();
        assertThat(s.statusOf(ride)).isEqualTo("COMPLETED");
    }

    @Test
    void shouldChargeBaseFareAndPerBlockDistance() {
        RideSharingApi s = service();
        String ride = s.requestRide("r1", 1, 0, 5, 5).orElseThrow();
        s.startRide(ride);

        assertThat(s.completeRide(ride)).hasValue(500L + 100L * (4 + 5));
    }

    @Test
    void shouldMoveDriverToDropLocationAndMakeThemAvailableAgain() {
        RideSharingApi s = Candidate.of(Solution::create);
        s.addDriver("d1", 0, 0);
        String ride = s.requestRide("r1", 1, 0, 8, 8).orElseThrow();
        s.startRide(ride);
        s.completeRide(ride);

        assertThat(s.driverLocation("d1")).hasValue("8,8");
        String next = s.requestRide("r2", 9, 9, 0, 0).orElseThrow();
        assertThat(s.driverOf(next)).hasValue("d1");
    }

    @Test
    void shouldReleaseDriverWhenAMatchedRideIsCancelled() {
        RideSharingApi s = Candidate.of(Solution::create);
        s.addDriver("d1", 0, 0);
        String ride = s.requestRide("r1", 1, 0, 8, 8).orElseThrow();

        assertThat(s.cancelRide(ride)).isTrue();

        assertThat(s.statusOf(ride)).isEqualTo("CANCELLED");
        assertThat(s.driverLocation("d1")).hasValue("0,0");
        assertThat(s.requestRide("r2", 1, 0, 2, 2)).isPresent();
    }

    @Test
    void shouldLetARiderRequestAgainAfterCancellationOrCompletion() {
        RideSharingApi s = service();
        String first = s.requestRide("r1", 1, 0, 5, 5).orElseThrow();
        s.cancelRide(first);
        String second = s.requestRide("r1", 1, 0, 5, 5).orElseThrow();
        s.startRide(second);
        s.completeRide(second);

        assertThat(s.requestRide("r1", 5, 5, 6, 6)).isPresent();
    }

    @Test
    void shouldRefuseIllegalTransitions() {
        RideSharingApi s = service();
        String ride = s.requestRide("r1", 1, 0, 5, 5).orElseThrow();
        s.startRide(ride);

        assertThat(s.cancelRide(ride)).isFalse();         // in progress
        s.completeRide(ride);
        assertThat(s.cancelRide(ride)).isFalse();         // completed
        assertThat(s.startRide(ride)).isFalse();
        assertThat(s.completeRide(ride)).isEmpty();
    }

    @Test
    void shouldHandleUnknownIdsGracefully() {
        RideSharingApi s = service();

        assertThat(s.statusOf("nope")).isEqualTo("UNKNOWN");
        assertThat(s.driverOf("nope")).isEmpty();
        assertThat(s.startRide("nope")).isFalse();
        assertThat(s.cancelRide("nope")).isFalse();
        assertThat(s.completeRide("nope")).isEmpty();
        assertThat(s.driverLocation("ghost")).isEmpty();
    }

    @Test
    void shouldFindTheNearestAmongManyDrivers() {
        RideSharingApi s = Candidate.of(Solution::create);
        for (int i = 0; i < 2000; i++) {
            s.addDriver(String.format("d%05d", i), 1000 + i, 1000 + i);
        }
        s.addDriver("near", 4, 4);

        Optional<String> ride = s.requestRide("r1", 0, 0, 1, 1);

        assertThat(ride).isPresent();
        assertThat(s.driverOf(ride.get())).hasValue("near");
    }
}
