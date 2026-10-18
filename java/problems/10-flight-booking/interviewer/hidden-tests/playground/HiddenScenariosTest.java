package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.FlightBookingApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class HiddenScenariosTest {
    private static final long MINUTE = 60_000L;
    private static final long HOUR = 60 * MINUTE;
    private static final long NOW = 5_000_000L;
    private static final long DEPARTURE = NOW + 100 * HOUR;

    private final FakeClock clock = new FakeClock(NOW);

    private FlightBookingApi system() {
        FlightBookingApi api = Candidate.of(() -> Solution.create(clock));
        api.addFlight("F1", "BLR", "DEL", DEPARTURE, 3, 1, 5001, 20001);
        return api;
    }

    @Test
    void shouldGiveFullRefundExactly24HoursBeforeAndHalfOneMillisecondLater() {
        FlightBookingApi api = system();
        String a = api.reserve("F1", "E1", "A").orElseThrow();
        api.confirm(a, true);
        String b = api.reserve("F1", "E2", "B").orElseThrow();
        api.confirm(b, true);

        clock.set(DEPARTURE - 24 * HOUR);
        assertThat(api.cancel(a)).hasValue(5001L);
        clock.advance(1);
        assertThat(api.cancel(b)).hasValue(5001L / 2);
    }

    @Test
    void shouldRoundHalfRefundDown() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "B1", "A").orElseThrow();
        api.confirm(id, true);
        clock.set(DEPARTURE - HOUR);
        assertThat(api.cancel(id)).hasValue(10000L);
    }

    @Test
    void failedPaymentShouldNotExtendTheHold() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "E1", "A").orElseThrow();
        clock.advance(9 * MINUTE);
        assertThat(api.confirm(id, false)).isFalse();
        clock.advance(MINUTE);
        assertThat(api.confirm(id, true)).isFalse();
    }

    @Test
    void samePassengerMayHoldSeveralSeats() {
        FlightBookingApi api = system();
        assertThat(api.reserve("F1", "E1", "A")).isPresent();
        assertThat(api.reserve("F1", "E2", "A")).isPresent();
        assertThat(api.availableSeats("F1")).containsExactly("B1", "E3");
    }

    @Test
    void cancelledSeatCanBeReservedByAnotherPassenger() {
        FlightBookingApi api = system();
        String id = api.reserve("F1", "E1", "A").orElseThrow();
        api.cancel(id);
        assertThat(api.reserve("F1", "E1", "B")).isPresent();
        assertThat(api.cancel(id)).isEmpty();
    }
}
