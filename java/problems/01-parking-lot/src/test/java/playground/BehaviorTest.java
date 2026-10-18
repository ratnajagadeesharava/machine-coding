package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ParkingLotApi;
import java.time.Duration;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {
    private static final long MINUTE = 60_000L;
    private static final long HOUR = 60 * MINUTE;

    private final FakeClock clock = new FakeClock(1_000_000L);

    private ParkingLotApi lot(int floors, int small, int medium, int large) {
        return Candidate.of(() -> Solution.create(floors, small, medium, large, clock));
    }

    @Test
    void shouldParkVehicleSuccessfully() {
        ParkingLotApi lot = lot(1, 0, 1, 0);

        Optional<String> ticket = lot.enter("CAR", "KA-01");

        assertThat(ticket).isPresent();
        assertThat(ticket.get()).isNotBlank();
        assertThat(lot.availableSpots("CAR")).isZero();
    }

    @Test
    void shouldRejectVehicleWhenParkingIsFull() {
        ParkingLotApi lot = lot(1, 0, 1, 1);

        assertThat(lot.enter("CAR", "A")).isPresent();
        assertThat(lot.enter("CAR", "B")).isPresent();
        assertThat(lot.enter("CAR", "C")).isEmpty();
    }

    @Test
    void shouldRejectTruckWhenNoLargeSpotExists() {
        ParkingLotApi lot = lot(2, 3, 3, 0);

        assertThat(lot.enter("TRUCK", "T-1")).isEmpty();
        assertThat(lot.availableSpots("TRUCK")).isZero();
    }

    @Test
    void shouldRejectUnknownVehicleType() {
        ParkingLotApi lot = lot(1, 1, 1, 1);

        assertThat(lot.enter("HOVERCRAFT", "H-1")).isEmpty();
        assertThat(lot.availableSpots("CAR")).isEqualTo(2);
    }

    @Test
    void shouldUseSmallestFittingSpotFirst() {
        ParkingLotApi lot = lot(1, 1, 1, 0);

        assertThat(lot.enter("MOTORCYCLE", "M-1")).isPresent(); // takes the small spot
        assertThat(lot.availableSpots("CAR")).isEqualTo(1);     // medium untouched
        assertThat(lot.enter("CAR", "C-1")).isPresent();
        assertThat(lot.enter("MOTORCYCLE", "M-2")).isEmpty();
    }

    @Test
    void shouldReportAvailabilityPerVehicleType() {
        ParkingLotApi lot = lot(1, 1, 1, 1);

        assertThat(lot.availableSpots("MOTORCYCLE")).isEqualTo(3);
        assertThat(lot.availableSpots("CAR")).isEqualTo(2);
        assertThat(lot.availableSpots("TRUCK")).isEqualTo(1);
    }

    @Test
    void shouldAssignLowerFloorFirst() {
        ParkingLotApi lot = lot(3, 0, 1, 0);

        String t0 = lot.enter("CAR", "A").orElseThrow();
        String t1 = lot.enter("CAR", "B").orElseThrow();
        String t2 = lot.enter("CAR", "C").orElseThrow();

        assertThat(lot.floorOf(t0)).hasValue(0);
        assertThat(lot.floorOf(t1)).hasValue(1);
        assertThat(lot.floorOf(t2)).hasValue(2);
    }

    @Test
    void shouldCalculateCorrectFeeForPartialHours() {
        ParkingLotApi lot = lot(1, 0, 1, 0);
        String ticket = lot.enter("CAR", "A").orElseThrow();

        clock.advance(90 * MINUTE);

        assertThat(lot.exit(ticket)).hasValue(4000L);
    }

    @Test
    void shouldChargeMinimumOneHour() {
        ParkingLotApi lot = lot(1, 0, 1, 0);
        String ticket = lot.enter("CAR", "A").orElseThrow();

        assertThat(lot.exit(ticket)).hasValue(2000L);
    }

    @Test
    void shouldTreatExactHourBoundaryAsOneHour() {
        ParkingLotApi lot = lot(1, 0, 2, 0);
        String exact = lot.enter("CAR", "A").orElseThrow();
        String over = lot.enter("CAR", "B").orElseThrow();

        clock.advance(HOUR);
        assertThat(lot.exit(exact)).hasValue(2000L);
        clock.advance(MINUTE);
        assertThat(lot.exit(over)).hasValue(4000L);
    }

    @Test
    void shouldApplyRatePerVehicleType() {
        ParkingLotApi lot = lot(1, 1, 0, 1);
        String bike = lot.enter("MOTORCYCLE", "M").orElseThrow();
        String truck = lot.enter("TRUCK", "T").orElseThrow();

        clock.advance(2 * HOUR);

        assertThat(lot.exit(bike)).hasValue(2000L);
        assertThat(lot.exit(truck)).hasValue(10000L);
    }

    @Test
    void shouldReleaseSpotAfterVehicleExits() {
        ParkingLotApi lot = lot(1, 0, 1, 0);
        String ticket = lot.enter("CAR", "A").orElseThrow();
        assertThat(lot.enter("CAR", "B")).isEmpty();

        lot.exit(ticket);

        assertThat(lot.availableSpots("CAR")).isEqualTo(1);
        assertThat(lot.enter("CAR", "B")).isPresent();
    }

    @Test
    void shouldRejectSecondEntryForSamePlate() {
        ParkingLotApi lot = lot(1, 0, 3, 0);

        assertThat(lot.enter("CAR", "SAME")).isPresent();
        assertThat(lot.enter("CAR", "SAME")).isEmpty();
        assertThat(lot.availableSpots("CAR")).isEqualTo(2);
    }

    @Test
    void shouldRejectExitForUnknownOrAlreadyUsedTicket() {
        ParkingLotApi lot = lot(1, 0, 1, 0);
        String ticket = lot.enter("CAR", "A").orElseThrow();

        assertThat(lot.exit("no-such-ticket")).isEmpty();
        assertThat(lot.exit(ticket)).isPresent();
        assertThat(lot.exit(ticket)).isEmpty();
        assertThat(lot.availableSpots("CAR")).isEqualTo(1); // second exit did not free a phantom spot
    }

    @Test
    void shouldIssueDistinctTickets() {
        ParkingLotApi lot = lot(1, 0, 5, 0);
        Set<String> tickets = new HashSet<>();
        for (int i = 0; i < 5; i++) {
            tickets.add(lot.enter("CAR", "P" + i).orElseThrow());
        }
        assertThat(tickets).hasSize(5);
    }

    @Test
    void shouldNeverDoubleAllocateUnderConcurrentEntry() {
        ParkingLotApi lot = lot(1, 0, 0, 10);

        List<Optional<String>> results = Concurrently.collect(50, i -> lot.enter("TRUCK", "T-" + i));

        List<String> tickets = results.stream().flatMap(Optional::stream).toList();
        assertThat(tickets).hasSize(10);
        assertThat(new HashSet<>(tickets)).hasSize(10);
        assertThat(lot.availableSpots("TRUCK")).isZero();
    }

    @Test
    void shouldKeepCapacityConsistentUnderConcurrentEntryAndExit() {
        ParkingLotApi lot = lot(2, 2, 2, 2);

        Concurrently.run(12, t -> {
            for (int i = 0; i < 300; i++) {
                String plate = "T" + t + "-" + i;
                lot.enter("CAR", plate).ifPresent(lot::exit);
            }
        });

        assertThat(lot.availableSpots("MOTORCYCLE")).isEqualTo(12);
    }

    @Test
    void shouldHandleManyOperationsQuickly() {
        ParkingLotApi lot = lot(100, 30, 40, 30);
        org.junit.jupiter.api.Assertions.assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            for (int i = 0; i < 100_000; i++) {
                String ticket = lot.enter("CAR", "P" + i).orElseThrow();
                lot.exit(ticket);
            }
        });
    }
}
