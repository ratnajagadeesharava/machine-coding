package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ParkingLotApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class HiddenScenariosTest {
    private static final long HOUR = 3_600_000L;
    private final FakeClock clock = new FakeClock(5_000L);

    private ParkingLotApi lot(int floors, int small, int medium, int large) {
        return Candidate.of(() -> Solution.create(floors, small, medium, large, clock));
    }

    @Test
    void shouldAllowSamePlateAgainAfterExit() {
        ParkingLotApi lot = lot(1, 0, 2, 0);
        String t = lot.enter("CAR", "X").orElseThrow();
        lot.exit(t);
        assertThat(lot.enter("CAR", "X")).isPresent();
    }

    @Test
    void shouldForgetTicketFloorAfterExit() {
        ParkingLotApi lot = lot(1, 0, 1, 0);
        String t = lot.enter("CAR", "X").orElseThrow();
        lot.exit(t);
        assertThat(lot.floorOf(t)).isEmpty();
    }

    @Test
    void shouldSpillMotorcycleThenCarIntoLargerSpots() {
        ParkingLotApi lot = lot(1, 0, 0, 2);
        assertThat(lot.enter("MOTORCYCLE", "M")).isPresent();
        assertThat(lot.enter("CAR", "C")).isPresent();
        assertThat(lot.enter("MOTORCYCLE", "M2")).isEmpty();
    }

    @Test
    void shouldChargeLongStaysByStartedHours() {
        ParkingLotApi lot = lot(1, 0, 1, 0);
        String t = lot.enter("CAR", "X").orElseThrow();
        clock.advance(24 * HOUR + 1);
        assertThat(lot.exit(t)).hasValue(25 * 2000L);
    }

    @Test
    void refusedEntryMustNotConsumeSpotOrPlate() {
        ParkingLotApi lot = lot(1, 0, 0, 1);
        assertThat(lot.enter("CAR", "A")).isPresent();
        assertThat(lot.enter("CAR", "B")).isEmpty(); // full
        assertThat(lot.enter("CAR", "A")).isEmpty(); // duplicate plate
        assertThat(lot.availableSpots("CAR")).isZero();
    }
}
