package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.TicketingApi;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class HiddenScenariosTest {

    @Test
    void shouldRecycleExpiringHoldsWhileOthersKeepBooking() {
        FakeClock clock = new FakeClock(0);
        TicketingApi t = Candidate.of(() -> Solution.create(clock));
        t.addScreen("PVR", "blr", "s", 5, 10);
        t.addShow("sh", "PVR", "s", "m", 100_000, 60, 10, 20);
        AtomicInteger bookings = new AtomicInteger();

        Concurrently.run(9, 60_000, u -> {
            for (int i = 0; i < 200; i++) {
                if (u == 8) {
                    clock.advance(60_000); // time passes: holds expire
                    continue;
                }
                String seat = "" + (char) ('A' + (u % 5)) + (1 + i % 10);
                try {
                    String hold = t.hold("sh", "u" + u, List.of(seat));
                    if (i % 2 == 0) {
                        t.confirm(hold, "u" + u);
                        bookings.incrementAndGet();
                    }
                } catch (IllegalStateException e) {
                    // taken or expired
                }
            }
        });

        clock.advance(10 * 60_000); // every hold that is left has expired now
        assertThat(t.availableSeats("sh").size() + bookings.get()).isEqualTo(50);
    }

    @Test
    void shouldSearchAcrossManyShows() {
        FakeClock clock = new FakeClock(0);
        TicketingApi t = Candidate.of(() -> Solution.create(clock));
        t.addScreen("PVR", "blr", "s", 2, 2);
        for (int i = 0; i < 2_000; i++) {
            t.addShow("sh" + i, "PVR", "s", i % 2 == 0 ? "a" : "b", 1_000L + i * 200L, 100, 10, 20);
        }

        assertThat(t.searchShows("blr", "a")).hasSize(1_000);
    }
}
