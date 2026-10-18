package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SeatBookingApi;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private SeatBookingApi system(int seats) {
        SeatBookingApi api = Candidate.of(Solution::create);
        api.addTrip("T", List.of("A", "B", "C", "D", "E"), seats);
        return api;
    }

    @Test
    void shouldHandleOverlappingButNotNestedJourneys() {
        SeatBookingApi api = system(2);
        String a = api.book("T", "A", "C", "p1").orElseThrow(); // seat 1
        String b = api.book("T", "B", "D", "p2").orElseThrow(); // seat 2
        String c = api.book("T", "C", "E", "p3").orElseThrow(); // seat 1 (free from C)
        assertThat(api.seatOf(a)).hasValue(1);
        assertThat(api.seatOf(b)).hasValue(2);
        assertThat(api.seatOf(c)).hasValue(1);
        assertThat(api.book("T", "A", "E", "p4")).isEmpty();
    }

    @Test
    void bookSeatShouldSucceedWhenSeatIsBusyOnlyOutsideTheJourney() {
        SeatBookingApi api = system(1);
        api.bookSeat("T", 1, "A", "B", "p1");
        assertThat(api.bookSeat("T", 1, "D", "E", "p2")).isPresent();
        assertThat(api.bookSeat("T", 1, "B", "D", "p3")).isPresent();
        assertThat(api.availableSeats("T", "A", "E")).isZero();
    }

    @Test
    void cancellingOneBookingMustNotFreeNeighbouringSegments() {
        SeatBookingApi api = system(1);
        String a = api.book("T", "A", "B", "p1").orElseThrow();
        api.book("T", "B", "C", "p2");
        api.cancel(a);
        assertThat(api.availableSeats("T", "A", "B")).isEqualTo(1);
        assertThat(api.availableSeats("T", "B", "C")).isZero();
        assertThat(api.book("T", "A", "C", "p3")).isEmpty();
    }

    @Test
    void wholeRouteJourneyShouldNeedAFullyFreeSeat() {
        SeatBookingApi api = system(2);
        api.bookSeat("T", 1, "B", "C", "p1");
        api.bookSeat("T", 2, "D", "E", "p2");
        assertThat(api.book("T", "A", "E", "p3")).isEmpty();
        assertThat(api.availableSeats("T", "A", "B")).isEqualTo(2);
    }
}
