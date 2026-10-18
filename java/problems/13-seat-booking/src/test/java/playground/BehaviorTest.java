package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SeatBookingApi;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private SeatBookingApi system() {
        SeatBookingApi api = Candidate.of(Solution::create);
        api.addTrip("T", List.of("A", "B", "C", "D"), 2);
        return api;
    }

    @Test
    void shouldAssignLowestSeatAndReuseItForNonOverlappingJourneys() {
        SeatBookingApi api = system();

        String p1 = api.book("T", "A", "C", "p1").orElseThrow();
        String p2 = api.book("T", "C", "D", "p2").orElseThrow();

        assertThat(api.seatOf(p1)).hasValue(1);
        assertThat(api.seatOf(p2)).hasValue(1);
    }

    @Test
    void shouldUseAnotherSeatWhenJourneysOverlap() {
        SeatBookingApi api = system();
        api.book("T", "A", "C", "p1");

        String p3 = api.book("T", "B", "D", "p3").orElseThrow();

        assertThat(api.seatOf(p3)).hasValue(2);
    }

    @Test
    void shouldRefuseWhenNoSeatIsFreeForTheWholeJourney() {
        SeatBookingApi api = system();
        api.book("T", "A", "C", "p1");
        api.book("T", "B", "D", "p3");

        assertThat(api.book("T", "B", "C", "p4")).isEmpty();
        assertThat(api.book("T", "A", "D", "p5")).isEmpty();
    }

    @Test
    void shouldReportAvailabilityPerJourney() {
        SeatBookingApi api = system();
        api.book("T", "A", "B", "p1");

        assertThat(api.availableSeats("T", "A", "B")).isEqualTo(1);
        assertThat(api.availableSeats("T", "B", "D")).isEqualTo(2);
        assertThat(api.availableSeats("T", "A", "D")).isEqualTo(1);
        assertThat(api.availableSeats("T", "C", "D")).isEqualTo(2);
    }

    @Test
    void shouldReportZeroAvailabilityWhenEverySeatIsBusyOnSomeSegment() {
        SeatBookingApi api = system();
        api.book("T", "A", "B", "p1"); // seat 1, segment A-B
        api.book("T", "C", "D", "p2"); // seat 1, segment C-D
        api.book("T", "B", "C", "p3"); // seat 1, segment B-C
        api.book("T", "A", "D", "p4"); // seat 2 whole trip

        assertThat(api.availableSeats("T", "A", "D")).isZero();
        assertThat(api.availableSeats("T", "B", "C")).isZero();
    }

    @Test
    void shouldBookSpecificSeat() {
        SeatBookingApi api = system();

        String id = api.bookSeat("T", 2, "A", "B", "p1").orElseThrow();

        assertThat(api.seatOf(id)).hasValue(2);
        assertThat(api.bookSeat("T", 2, "A", "B", "p2")).isEmpty();
        assertThat(api.bookSeat("T", 2, "B", "C", "p3")).isPresent();
        String auto = api.book("T", "A", "B", "p4").orElseThrow();
        assertThat(api.seatOf(auto)).hasValue(1);
    }

    @Test
    void shouldRefuseSeatsOutOfRange() {
        SeatBookingApi api = system();

        assertThat(api.bookSeat("T", 0, "A", "B", "p")).isEmpty();
        assertThat(api.bookSeat("T", 3, "A", "B", "p")).isEmpty();
        assertThat(api.bookSeat("T", -1, "A", "B", "p")).isEmpty();
    }

    @Test
    void shouldRefuseInvalidJourneys() {
        SeatBookingApi api = system();

        assertThat(api.book("T", "B", "B", "p")).isEmpty();
        assertThat(api.book("T", "C", "A", "p")).isEmpty();
        assertThat(api.book("T", "A", "Z", "p")).isEmpty();
        assertThat(api.book("T", "Z", "A", "p")).isEmpty();
        assertThat(api.book("NOPE", "A", "B", "p")).isEmpty();
        assertThat(api.availableSeats("T", "D", "A")).isZero();
        assertThat(api.availableSeats("NOPE", "A", "B")).isZero();
        assertThat(api.availableSeats("T", "A", "D")).isEqualTo(2);
    }

    @Test
    void shouldFreeOnlyTheCancelledSegments() {
        SeatBookingApi api = system();
        String a = api.book("T", "A", "C", "p1").orElseThrow();
        api.book("T", "C", "D", "p2");

        assertThat(api.cancel(a)).isTrue();

        assertThat(api.availableSeats("T", "A", "C")).isEqualTo(2);
        assertThat(api.availableSeats("T", "C", "D")).isEqualTo(1);
        assertThat(api.seatOf(a)).isEmpty();
    }

    @Test
    void shouldRefuseCancellingTwiceOrUnknownBookings() {
        SeatBookingApi api = system();
        String a = api.book("T", "A", "B", "p1").orElseThrow();

        assertThat(api.cancel(a)).isTrue();
        assertThat(api.cancel(a)).isFalse();
        assertThat(api.cancel("nope")).isFalse();
        assertThat(api.availableSeats("T", "A", "B")).isEqualTo(2);
    }

    @Test
    void shouldAssignLowestFreeSeatAgainAfterCancellation() {
        SeatBookingApi api = system();
        String a = api.book("T", "A", "D", "p1").orElseThrow();   // seat 1
        api.book("T", "A", "D", "p2");                            // seat 2

        api.cancel(a);

        String again = api.book("T", "B", "C", "p3").orElseThrow();
        assertThat(api.seatOf(again)).hasValue(1);
    }

    @Test
    void shouldKeepTripsIndependent() {
        SeatBookingApi api = system();
        api.addTrip("U", List.of("A", "B"), 1);

        api.book("T", "A", "B", "p1");
        api.book("T", "A", "B", "p2");

        assertThat(api.availableSeats("U", "A", "B")).isEqualTo(1);
        assertThat(api.book("U", "A", "B", "p3")).isPresent();
    }

    @Test
    void shouldNeverSellMoreSeatsThanExistWhenManyPassengersRace() {
        SeatBookingApi api = Candidate.of(Solution::create);
        api.addTrip("BUS", List.of("A", "B"), 40);

        List<Optional<String>> results = Concurrently.collect(100, i -> api.book("BUS", "A", "B", "p" + i));

        List<String> ids = results.stream().flatMap(Optional::stream).toList();
        assertThat(ids).hasSize(40);
        Set<Integer> seats = new HashSet<>();
        for (String id : ids) {
            seats.add(api.seatOf(id).getAsInt());
        }
        assertThat(seats).hasSize(40);
        assertThat(api.availableSeats("BUS", "A", "B")).isZero();
    }

    @Test
    void shouldNeverDoubleSellASeatOnOverlappingJourneysUnderConcurrency() {
        SeatBookingApi api = Candidate.of(Solution::create);
        List<String> stops = List.of("S0", "S1", "S2", "S3", "S4", "S5");
        api.addTrip("TRAIN", stops, 5);

        List<Optional<String>> results = Concurrently.collect(60, i -> {
            int from = i % 5;
            int to = Math.min(5, from + 1 + (i % 3));
            return api.book("TRAIN", stops.get(from), stops.get(to), "p" + i);
        });

        // Verify no seat is used by two overlapping journeys by checking each segment's occupancy.
        int[][] occupancy = new int[5][6]; // seat -> segment count
        for (int i = 0; i < results.size(); i++) {
            if (results.get(i).isEmpty()) {
                continue;
            }
            int from = i % 5;
            int to = Math.min(5, from + 1 + (i % 3));
            int seat = api.seatOf(results.get(i).get()).getAsInt();
            for (int seg = from; seg < to; seg++) {
                occupancy[seat - 1][seg]++;
                assertThat(occupancy[seat - 1][seg]).as("seat %d segment %d", seat, seg).isEqualTo(1);
            }
        }
    }

    @Test
    void shouldRestoreFullAvailabilityAfterConcurrentBookAndCancel() {
        SeatBookingApi api = Candidate.of(Solution::create);
        List<String> stops = List.of("A", "B", "C", "D");
        api.addTrip("T", stops, 3);

        Concurrently.run(16, t -> {
            for (int i = 0; i < 200; i++) {
                int from = (t + i) % 3;
                int to = i % 2 == 0 ? 3 : from + 1;
                api.book("T", stops.get(from), stops.get(to), "p" + t).ifPresent(api::cancel);
            }
        });

        assertThat(api.availableSeats("T", "A", "D")).isEqualTo(3);
    }
}
