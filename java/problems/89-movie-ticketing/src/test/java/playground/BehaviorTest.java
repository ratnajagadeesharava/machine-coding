package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.TicketingApi;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {

    private final FakeClock clock = new FakeClock(0);

    private TicketingApi api() {
        TicketingApi t = Candidate.of(() -> Solution.create(clock));
        t.addScreen("PVR", "blr", "s1", 4, 5); // rows A,B premium; C,D regular; 20 seats
        t.addShow("show1", "PVR", "s1", "dune", 600, 150, 200, 350);
        return t;
    }

    private void minutes(long m) {
        clock.advance(m * 60_000);
    }

    @Test
    void shouldListAllSeatsInRowThenNumberOrder() {
        TicketingApi t = api();

        List<String> seats = t.availableSeats("show1");

        assertThat(seats).hasSize(20);
        assertThat(seats.subList(0, 6)).containsExactly("A1", "A2", "A3", "A4", "A5", "B1");
        assertThat(seats.get(19)).isEqualTo("D5");
    }

    @Test
    void shouldSearchUpcomingShowsByCityAndMovieInTimeOrder() {
        TicketingApi t = api();
        t.addShow("late", "PVR", "s1", "dune", 900, 150, 200, 350);
        t.addShow("other", "PVR", "s1", "barbie", 750, 100, 200, 350);
        t.addScreen("INOX", "del", "s9", 2, 2);
        t.addShow("delhi", "INOX", "s9", "dune", 610, 150, 100, 100);

        assertThat(t.searchShows("blr", "dune")).containsExactly("show1", "late");
        assertThat(t.searchShows("del", "dune")).containsExactly("delhi");
        assertThat(t.searchShows("blr", "nothing")).isEmpty();

        minutes(700); // show1 (600) has started
        assertThat(t.searchShows("blr", "dune")).containsExactly("late");
    }

    @Test
    void shouldRejectOverlappingShowsOnTheSameScreenButAllowTouching() {
        TicketingApi t = api(); // show1 600..750

        assertThatThrownBy(() -> t.addShow("clash", "PVR", "s1", "x", 700, 60, 1, 1)).isInstanceOf(IllegalStateException.class);
        t.addShow("next", "PVR", "s1", "x", 750, 60, 1, 1);
        assertThatThrownBy(() -> t.addShow("show1", "PVR", "s1", "x", 2_000, 60, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.addShow("bad", "PVR", "nope", "x", 2_000, 60, 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.addShow("bad", "PVR", "s1", "x", 2_000, 0, 1, 1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldHideHeldSeatsFromOthersAndRefuseDoubleHolds() {
        TicketingApi t = api();

        String hold = t.hold("show1", "ann", List.of("A1", "A2"));

        assertThat(hold).isNotBlank();
        assertThat(t.availableSeats("show1")).hasSize(18).doesNotContain("A1", "A2");
        assertThatThrownBy(() -> t.hold("show1", "bob", List.of("A2", "A3"))).isInstanceOf(IllegalStateException.class);
        assertThat(t.availableSeats("show1")).contains("A3"); // all-or-nothing: A3 was not held by the failed attempt
    }

    @Test
    void shouldRejectInvalidHolds() {
        TicketingApi t = api();

        assertThatThrownBy(() -> t.hold("nope", "ann", List.of("A1"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.hold("show1", "ann", List.of("Z9"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.hold("show1", "ann", List.of("A6"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.hold("show1", "ann", List.of())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.hold("show1", "ann", List.of("A1", "A1"))).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldReleaseExpiredHoldsAfterFiveMinutes() {
        TicketingApi t = api();
        String hold = t.hold("show1", "ann", List.of("A1"));

        minutes(4);
        assertThat(t.availableSeats("show1")).doesNotContain("A1");
        minutes(1);

        assertThat(t.availableSeats("show1")).contains("A1");
        assertThatThrownBy(() -> t.confirm(hold, "ann")).isInstanceOf(IllegalStateException.class);
        t.hold("show1", "bob", List.of("A1"));
    }

    @Test
    void shouldConfirmAHoldIntoABookingWithTieredPrices() {
        TicketingApi t = api();
        String hold = t.hold("show1", "ann", List.of("B2", "A1", "C3"));

        String booking = t.confirm(hold, "ann");

        assertThat(t.seatsOf(booking)).containsExactly("A1", "B2", "C3");
        assertThat(t.totalOf(booking)).isEqualTo(350 + 350 + 200);
        assertThat(t.availableSeats("show1")).hasSize(17);
        minutes(10); // a confirmed seat never comes back
        assertThat(t.availableSeats("show1")).hasSize(17).doesNotContain("A1");
    }

    @Test
    void shouldProtectHoldsFromOtherUsersAndDoubleConfirm() {
        TicketingApi t = api();
        String hold = t.hold("show1", "ann", List.of("A1"));

        assertThatThrownBy(() -> t.confirm(hold, "bob")).isInstanceOf(SecurityException.class);
        assertThatThrownBy(() -> t.release(hold, "bob")).isInstanceOf(SecurityException.class);
        t.confirm(hold, "ann");
        assertThatThrownBy(() -> t.confirm(hold, "ann")).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldFreeSeatsWhenAHoldIsReleased() {
        TicketingApi t = api();
        String hold = t.hold("show1", "ann", List.of("A1", "A2"));

        t.release(hold, "ann");

        assertThat(t.availableSeats("show1")).hasSize(20);
        assertThatThrownBy(() -> t.release(hold, "ann")).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldRefuseToHoldAfterTheShowStarted() {
        TicketingApi t = api();
        minutes(600);

        assertThatThrownBy(() -> t.hold("show1", "ann", List.of("A1"))).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldRefundAccordingToHowCloseTheShowIs() {
        TicketingApi t = api(); // show at minute 600
        String b1 = t.confirm(t.hold("show1", "ann", List.of("A1")), "ann"); // 350
        String b2 = t.confirm(t.hold("show1", "ann", List.of("C1", "C2", "C3")), "ann"); // 600
        String b3 = t.confirm(t.hold("show1", "ann", List.of("D1")), "ann"); // 200

        assertThat(t.cancel(b1, "ann")).isEqualTo(350); // 600 minutes away
        minutes(500); // 100 minutes before the show
        assertThat(t.cancel(b2, "ann")).isEqualTo(300);
        minutes(75); // 25 minutes before
        assertThat(t.cancel(b3, "ann")).isZero();
        assertThat(t.availableSeats("show1")).hasSize(20);
        assertThatThrownBy(() -> t.cancel(b3, "ann")).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldNotLetOthersCancelABooking() {
        TicketingApi t = api();
        String b = t.confirm(t.hold("show1", "ann", List.of("A1")), "ann");

        assertThatThrownBy(() -> t.cancel(b, "bob")).isInstanceOf(SecurityException.class);
    }

    @Test
    void shouldLetExactlyOneThreadHoldTheSameSeat() {
        TicketingApi t = api();

        List<Boolean> wins = Concurrently.collect(16, i -> {
            try {
                t.hold("show1", "user" + i, List.of("A1"));
                return true;
            } catch (IllegalStateException e) {
                return false;
            }
        });

        assertThat(wins.stream().filter(w -> w).count()).isEqualTo(1);
    }

    @Test
    void shouldNeverSellTheSameSeatTwiceUnderLoad() {
        TicketingApi t = Candidate.of(() -> Solution.create(clock));
        t.addScreen("PVR", "blr", "big", 10, 20); // 200 seats
        t.addShow("big", "PVR", "big", "m", 10_000, 120, 100, 200);
        Set<String> sold = ConcurrentHashMap.newKeySet();
        AtomicInteger duplicates = new AtomicInteger();

        Concurrently.run(16, u -> {
            java.util.Random r = new java.util.Random(u);
            for (int i = 0; i < 100; i++) {
                String a = "" + (char) ('A' + r.nextInt(10)) + (1 + r.nextInt(20));
                String b = "" + (char) ('A' + r.nextInt(10)) + (1 + r.nextInt(20));
                List<String> seats = a.equals(b) ? List.of(a) : List.of(a, b);
                try {
                    String booking = t.confirm(t.hold("big", "user" + u, seats), "user" + u);
                    for (String s : t.seatsOf(booking)) {
                        if (!sold.add(s)) {
                            duplicates.incrementAndGet();
                        }
                    }
                } catch (IllegalStateException e) {
                    // seat taken: fine
                }
            }
        });

        assertThat(duplicates).hasValue(0);
        assertThat(t.availableSeats("big")).hasSize(200 - sold.size());
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            TicketingApi t = api();
            Set<String> won = ConcurrentHashMap.newKeySet();

            Concurrently.run(8, u -> {
                try {
                    t.hold("show1", "u" + u, List.of("A1", "A2"));
                    won.add("u" + u);
                } catch (IllegalStateException e) {
                    // lost
                }
            });

            assertThat(won).as("run %d", run).hasSize(1);
            assertThat(t.availableSeats("show1")).as("run %d", run).hasSize(18);
        }
    }
}
