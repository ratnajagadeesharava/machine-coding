package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.FlightBookingApi;
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
    private static final long NOW = 1_000_000_000L;
    private static final long DEPARTURE = NOW + 72 * HOUR;

    private final FakeClock clock = new FakeClock(NOW);

    private FlightBookingApi system() {
        FlightBookingApi api = Candidate.of(() -> Solution.create(clock));
        api.addFlight("F1", "BLR", "DEL", DEPARTURE, 3, 1, 5000, 20000);
        return api;
    }

    @Test
    void shouldSearchFlightsByRouteOrderedByDeparture() {
        FlightBookingApi api = system();
        api.addFlight("F0", "BLR", "DEL", DEPARTURE - 5 * HOUR, 2, 0, 9000, 0);
        api.addFlight("F2", "BLR", "DEL", DEPARTURE, 2, 0, 4000, 0);
        api.addFlight("X1", "BLR", "BOM", DEPARTURE, 2, 0, 3000, 0);

        assertThat(api.search("BLR", "DEL", NOW)).containsExactly("F0", "F2", "F1");
        assertThat(api.search("BLR", "DEL", DEPARTURE)).containsExactly("F2", "F1");
        assertThat(api.search("DEL", "BLR", NOW)).isEmpty();
    }

    @Test
    void shouldListAvailableSeatsBusinessFirst() {
        FlightBookingApi api = system();

        assertThat(api.availableSeats("F1")).containsExactly("B1", "E1", "E2", "E3");
        assertThat(api.availableSeats("nope")).isEmpty();
    }

    @Test
    void shouldReserveSeatAndHideItFromAvailability() {
        FlightBookingApi api = system();

        Optional<String> r = api.reserve("F1", "E2", "Asha");

        assertThat(r).isPresent();
        assertThat(api.availableSeats("F1")).containsExactly("B1", "E1", "E3");
    }

    @Test
    void shouldRefuseSeatThatIsTakenUnknownOrOnUnknownFlight() {
        FlightBookingApi api = system();
        api.reserve("F1", "E2", "Asha");

        assertThat(api.reserve("F1", "E2", "Ravi")).isEmpty();
        assertThat(api.reserve("F1", "E9", "Ravi")).isEmpty();
        assertThat(api.reserve("F9", "E1", "Ravi")).isEmpty();
        assertThat(api.reserve("F1", "E1", " ")).isEmpty();
    }

    @Test
    void shouldKeepConfirmedBookingBeyondTheHoldWindow() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "E2", "Asha").orElseThrow();

        assertThat(api.confirm(id, true)).isTrue();
        clock.advance(2 * HOUR);

        assertThat(api.availableSeats("F1")).doesNotContain("E2");
        assertThat(api.reserve("F1", "E2", "Ravi")).isEmpty();
    }

    @Test
    void shouldReleaseSeatWhenHoldExpires() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "E2", "Asha").orElseThrow();

        clock.advance(10 * MINUTE);

        assertThat(api.availableSeats("F1")).contains("E2");
        assertThat(api.confirm(id, true)).isFalse();
        assertThat(api.reserve("F1", "E2", "Ravi")).isPresent();
    }

    @Test
    void shouldKeepHoldJustBeforeExpiry() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "E2", "Asha").orElseThrow();

        clock.advance(10 * MINUTE - 1);

        assertThat(api.availableSeats("F1")).doesNotContain("E2");
        assertThat(api.confirm(id, true)).isTrue();
    }

    @Test
    void shouldKeepHoldAfterFailedPaymentAndAllowRetry() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "E2", "Asha").orElseThrow();

        assertThat(api.confirm(id, false)).isFalse();
        assertThat(api.availableSeats("F1")).doesNotContain("E2");
        clock.advance(5 * MINUTE);
        assertThat(api.confirm(id, true)).isTrue();
    }

    @Test
    void shouldNotConfirmTwice() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "E2", "Asha").orElseThrow();

        assertThat(api.confirm(id, true)).isTrue();
        assertThat(api.confirm(id, true)).isFalse();
        assertThat(api.confirm("unknown", true)).isFalse();
    }

    @Test
    void shouldRefundFullFareWhenCancellingWellBeforeDeparture() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "E2", "Asha").orElseThrow();
        api.confirm(id, true);

        assertThat(api.cancel(id)).hasValue(5000L);
        assertThat(api.availableSeats("F1")).contains("E2");
    }

    @Test
    void shouldRefundHalfWhenCancellingWithinTwentyFourHours() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "B1", "Asha").orElseThrow();
        api.confirm(id, true);

        clock.advance(72 * HOUR - 23 * HOUR);

        assertThat(api.cancel(id)).hasValue(10000L);
    }

    @Test
    void shouldReleaseSeatWithZeroRefundWhenCancellingAnUnconfirmedReservation() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "E2", "Asha").orElseThrow();

        assertThat(api.cancel(id)).hasValue(0L);
        assertThat(api.availableSeats("F1")).contains("E2");
    }

    @Test
    void shouldRefuseToCancelTwiceOrCancelUnknownIds() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "E2", "Asha").orElseThrow();
        api.confirm(id, true);

        assertThat(api.cancel(id)).isPresent();
        assertThat(api.cancel(id)).isEmpty();
        assertThat(api.cancel("nope")).isEmpty();
    }

    @Test
    void shouldRefuseToCancelExpiredReservation() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "E2", "Asha").orElseThrow();

        clock.advance(11 * MINUTE);

        assertThat(api.cancel(id)).isEmpty();
    }

    @Test
    void shouldApplyFareByCabin() {
        FlightBookingApi api = system();

        assertThat(api.fareOf("F1", "B1")).hasValue(20000L);
        assertThat(api.fareOf("F1", "E3")).hasValue(5000L);
        assertThat(api.fareOf("F1", "E4")).isEmpty();
        assertThat(api.fareOf("F9", "E1")).isEmpty();
    }

    @Test
    void shouldNotReserveOnFlightsThatAlreadyDeparted() {
        FlightBookingApi api = system();

        clock.set(DEPARTURE + 1);

        assertThat(api.reserve("F1", "E1", "Asha")).isEmpty();
    }

    @Test
    void shouldHideFullFlightsFromSearchUntilHoldsExpire() {
        FlightBookingApi api = Candidate.of(() -> Solution.create(clock));
        api.addFlight("S1", "BLR", "DEL", DEPARTURE, 1, 0, 5000, 0);
        api.reserve("S1", "E1", "Asha");

        assertThat(api.search("BLR", "DEL", NOW)).isEmpty();
        clock.advance(10 * MINUTE);
        assertThat(api.search("BLR", "DEL", NOW)).containsExactly("S1");
    }

    @Test
    void shouldAllowOnlyOneReservationWhenManyCustomersRaceForTheSameSeat() {
        FlightBookingApi api = system();

        List<Optional<String>> results = Concurrently.collect(32, i -> api.reserve("F1", "E1", "p" + i));

        assertThat(results.stream().filter(Optional::isPresent).count()).isEqualTo(1);
        assertThat(api.availableSeats("F1")).doesNotContain("E1");
    }

    @Test
    void shouldNeverDoubleBookWhenManyCustomersRaceForManySeats() {
        FlightBookingApi api = Candidate.of(() -> Solution.create(clock));
        api.addFlight("BIG", "BLR", "DEL", DEPARTURE, 40, 10, 5000, 20000);
        List<String> seats = api.availableSeats("BIG");
        assertThat(seats).hasSize(50);

        List<Optional<String>> results = Concurrently.collect(64, i -> {
            Optional<String> got = Optional.empty();
            for (int k = 0; k < seats.size() && got.isEmpty(); k++) {
                got = api.reserve("BIG", seats.get((i + k) % seats.size()), "p" + i);
            }
            return got;
        });

        List<String> reservationIds = results.stream().flatMap(Optional::stream).toList();
        assertThat(reservationIds).hasSize(50);
        assertThat(new HashSet<>(reservationIds)).hasSize(50);
        assertThat(api.availableSeats("BIG")).isEmpty();
        Set<String> confirmed = new HashSet<>();
        for (String id : reservationIds) {
            if (api.confirm(id, true)) {
                confirmed.add(id);
            }
        }
        assertThat(confirmed).hasSize(50);
    }
}
