package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.HotelBookingApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class HiddenScenariosTest {
    private final FakeClock clock = new FakeClock(0);

    private HotelBookingApi hotel() {
        HotelBookingApi h = Candidate.of(() -> Solution.create(clock));
        h.addRoom("A", "STD", 50);
        h.addRoom("B", "STD", 50);
        return h;
    }

    @Test
    void shouldAcceptOneNightStays() {
        HotelBookingApi h = hotel();
        String id = h.book("x", "STD", 7, 8).orElseThrow();
        assertThat(h.totalPrice(id)).hasValue(50L);
        assertThat(h.availableRooms("STD", 7)).isEqualTo(1);
        assertThat(h.availableRooms("STD", 8)).isEqualTo(2);
    }

    @Test
    void shouldNotLetTwoPartialOverlapsBlockEveryRoomIncorrectly() {
        HotelBookingApi h = hotel();
        h.book("a", "STD", 1, 3);   // A
        h.book("b", "STD", 4, 6);   // A
        h.book("c", "STD", 2, 5);   // B
        // Night 3 is free in A only; a request for [3,4) must succeed in A.
        String id = h.book("d", "STD", 3, 4).orElseThrow();
        assertThat(h.roomOf(id)).hasValue("A");
    }

    @Test
    void cancellingAnExpiredHoldShouldBeRefused() {
        HotelBookingApi h = hotel();
        String hold = h.hold("x", "STD", 1, 2).orElseThrow();
        clock.advance(20 * 60_000L);
        assertThat(h.cancel(hold)).isFalse();
    }

    @Test
    void typesShouldBeCaseSensitive() {
        HotelBookingApi h = hotel();
        assertThat(h.book("x", "std", 1, 2)).isEmpty();
    }

    @Test
    void expiredHoldShouldNotBlockLaterBookingsOfTheSameRoom() {
        HotelBookingApi h = hotel();
        h.hold("x", "STD", 1, 5);
        h.hold("y", "STD", 1, 5);
        assertThat(h.book("z", "STD", 2, 3)).isEmpty();
        clock.advance(16 * 60_000L);
        assertThat(h.book("z", "STD", 2, 3)).isPresent();
    }
}
