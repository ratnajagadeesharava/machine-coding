package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.HotelBookingApi;
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

    private final FakeClock clock = new FakeClock(1_000_000L);

    private HotelBookingApi hotel() {
        HotelBookingApi h = Candidate.of(() -> Solution.create(clock));
        h.addRoom("R1", "DOUBLE", 100);
        h.addRoom("R2", "DOUBLE", 100);
        h.addRoom("S1", "SUITE", 300);
        return h;
    }

    @Test
    void shouldBookRoomAndReduceAvailabilityOnlyForTheStayNights() {
        HotelBookingApi h = hotel();

        Optional<String> id = h.book("alice", "DOUBLE", 1, 4);

        assertThat(id).isPresent();
        assertThat(h.availableRooms("DOUBLE", 1)).isEqualTo(1);
        assertThat(h.availableRooms("DOUBLE", 3)).isEqualTo(1);
        assertThat(h.availableRooms("DOUBLE", 4)).isEqualTo(2); // checkout night is free
        assertThat(h.availableRooms("DOUBLE", 0)).isEqualTo(2);
    }

    @Test
    void shouldAssignSmallestFreeRoomIdFirst() {
        HotelBookingApi h = hotel();

        String first = h.book("alice", "DOUBLE", 1, 3).orElseThrow();
        String second = h.book("bob", "DOUBLE", 1, 3).orElseThrow();

        assertThat(h.roomOf(first)).hasValue("R1");
        assertThat(h.roomOf(second)).hasValue("R2");
    }

    @Test
    void shouldAllowBackToBackStaysInTheSameRoom() {
        HotelBookingApi h = hotel();

        String a = h.book("alice", "DOUBLE", 1, 4).orElseThrow();
        String b = h.book("bob", "DOUBLE", 4, 6).orElseThrow();

        assertThat(h.roomOf(a)).hasValue("R1");
        assertThat(h.roomOf(b)).hasValue("R1");
    }

    @Test
    void shouldRefuseWhenNoRoomIsFreeForTheWholeRange() {
        HotelBookingApi h = hotel();
        h.book("alice", "DOUBLE", 1, 4);
        h.book("bob", "DOUBLE", 4, 6);
        h.book("carol", "DOUBLE", 2, 5);

        assertThat(h.book("dave", "DOUBLE", 3, 4)).isEmpty();
        assertThat(h.book("dave", "DOUBLE", 1, 7)).isEmpty();
    }

    @Test
    void shouldFitStaysAroundExistingBookingsWithoutDoubleBooking() {
        HotelBookingApi h = hotel();
        String a = h.book("a", "DOUBLE", 1, 3).orElseThrow(); // R1
        String b = h.book("b", "DOUBLE", 3, 5).orElseThrow(); // R1
        String c = h.book("c", "DOUBLE", 2, 4).orElseThrow(); // R2
        String d = h.book("d", "DOUBLE", 1, 2).orElseThrow(); // R2 (R1 busy on night 1)

        assertThat(h.roomOf(a)).hasValue("R1");
        assertThat(h.roomOf(b)).hasValue("R1");
        assertThat(h.roomOf(c)).hasValue("R2");
        assertThat(h.roomOf(d)).hasValue("R2");
        assertThat(h.book("e", "DOUBLE", 1, 5)).isEmpty();
    }

    @Test
    void shouldKeepRoomTypesIndependent() {
        HotelBookingApi h = hotel();

        assertThat(h.book("alice", "SUITE", 1, 3)).isPresent();
        assertThat(h.book("bob", "SUITE", 2, 4)).isEmpty();
        assertThat(h.availableRooms("DOUBLE", 2)).isEqualTo(2);
    }

    @Test
    void shouldRefuseInvalidRequests() {
        HotelBookingApi h = hotel();

        assertThat(h.book("alice", "DOUBLE", 3, 3)).isEmpty();
        assertThat(h.book("alice", "DOUBLE", 5, 2)).isEmpty();
        assertThat(h.book("alice", "PENTHOUSE", 1, 2)).isEmpty();
        assertThat(h.availableRooms("PENTHOUSE", 1)).isZero();
    }

    @Test
    void shouldChargeNightsTimesRate() {
        HotelBookingApi h = hotel();

        String id = h.book("alice", "SUITE", 10, 13).orElseThrow();

        assertThat(h.totalPrice(id)).hasValue(900L);
        assertThat(h.totalPrice("nope")).isEmpty();
    }

    @Test
    void shouldReleaseRoomOnCancel() {
        HotelBookingApi h = hotel();
        h.book("a", "SUITE", 1, 5);
        String id = h.book("b", "DOUBLE", 1, 5).orElseThrow();

        assertThat(h.cancel(id)).isTrue();

        assertThat(h.availableRooms("DOUBLE", 2)).isEqualTo(2);
        assertThat(h.roomOf(id)).isEmpty();
        assertThat(h.cancel(id)).isFalse();
        assertThat(h.cancel("nope")).isFalse();
    }

    @Test
    void shouldBlockRoomWhileHeldAndConfirmIntoBooking() {
        HotelBookingApi h = hotel();
        String hold = h.hold("alice", "SUITE", 1, 3).orElseThrow();

        assertThat(h.book("bob", "SUITE", 2, 4)).isEmpty();
        assertThat(h.confirm(hold)).isTrue();
        clock.advance(60 * MINUTE);

        assertThat(h.book("bob", "SUITE", 2, 4)).isEmpty();
        assertThat(h.roomOf(hold)).hasValue("S1");
    }

    @Test
    void shouldReleaseRoomWhenHoldExpires() {
        HotelBookingApi h = hotel();
        String hold = h.hold("alice", "SUITE", 1, 3).orElseThrow();

        clock.advance(15 * MINUTE);

        assertThat(h.availableRooms("SUITE", 1)).isEqualTo(1);
        assertThat(h.confirm(hold)).isFalse();
        assertThat(h.book("bob", "SUITE", 1, 3)).isPresent();
    }

    @Test
    void shouldKeepHoldJustBeforeExpiry() {
        HotelBookingApi h = hotel();
        String hold = h.hold("alice", "SUITE", 1, 3).orElseThrow();

        clock.advance(15 * MINUTE - 1);

        assertThat(h.availableRooms("SUITE", 1)).isZero();
        assertThat(h.confirm(hold)).isTrue();
    }

    @Test
    void shouldReleaseHoldOnCancelAndRefuseConfirmingTwice() {
        HotelBookingApi h = hotel();
        String hold = h.hold("alice", "SUITE", 1, 3).orElseThrow();
        assertThat(h.cancel(hold)).isTrue();
        assertThat(h.confirm(hold)).isFalse();

        String hold2 = h.hold("bob", "SUITE", 1, 3).orElseThrow();
        assertThat(h.confirm(hold2)).isTrue();
        assertThat(h.confirm(hold2)).isFalse();
    }

    @Test
    void shouldNeverBookTheSameRoomTwiceUnderConcurrency() {
        HotelBookingApi h = hotel();

        List<Optional<String>> results = Concurrently.collect(30, i -> h.book("g" + i, "SUITE", 5, 9));

        List<String> ids = results.stream().flatMap(Optional::stream).toList();
        assertThat(ids).hasSize(1);
        assertThat(h.availableRooms("SUITE", 6)).isZero();
    }

    @Test
    void shouldAssignDistinctRoomsToConcurrentBookingsOfTheSameDates() {
        HotelBookingApi h = Candidate.of(() -> Solution.create(clock));
        for (int i = 1; i <= 5; i++) {
            h.addRoom("R" + i, "DOUBLE", 100);
        }

        List<Optional<String>> results = Concurrently.collect(40, i -> h.book("g" + i, "DOUBLE", 1, 4));

        List<String> ids = results.stream().flatMap(Optional::stream).toList();
        assertThat(ids).hasSize(5);
        Set<String> rooms = new HashSet<>();
        for (String id : ids) {
            rooms.add(h.roomOf(id).orElseThrow());
        }
        assertThat(rooms).hasSize(5);
    }

    @Test
    void shouldStayConsistentUnderConcurrentBookAndCancelOfOverlappingRanges() {
        HotelBookingApi h = Candidate.of(() -> Solution.create(clock));
        h.addRoom("R1", "DOUBLE", 100);
        h.addRoom("R2", "DOUBLE", 100);

        Concurrently.run(16, t -> {
            for (int i = 0; i < 200; i++) {
                int start = (t + i) % 7;
                h.book("g" + t, "DOUBLE", start, start + 1 + (i % 3)).ifPresent(h::cancel);
            }
        });

        for (int day = 0; day < 12; day++) {
            assertThat(h.availableRooms("DOUBLE", day)).isEqualTo(2);
        }
    }
}
