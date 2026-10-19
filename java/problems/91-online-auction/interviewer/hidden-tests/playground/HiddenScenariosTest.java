package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.AuctionApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class HiddenScenariosTest {

    @Test
    void shouldHandleManyAuctionsBeingBidOnAtOnce() {
        FakeClock clock = new FakeClock(0);
        AuctionApi a = Candidate.of(() -> Solution.create(clock));
        for (int i = 0; i < 50; i++) {
            a.createAuction("a" + i, "seller", 1, 0, 3_600_000);
        }

        Concurrently.run(16, 60_000, t -> {
            for (int round = 1; round <= 500; round++) {
                a.placeBid("a" + (round % 50), "b" + t, (long) round * 16 + t);
            }
        });

        for (int i = 0; i < 50; i++) {
            assertThat(a.highestBid("a" + i)).isPresent();
        }
    }

    @Test
    void shouldKeepExtendingWhileBidsKeepArrivingInTheLastMinutes() {
        FakeClock clock = new FakeClock(0);
        AuctionApi a = Candidate.of(() -> Solution.create(clock));
        a.createAuction("x", "s", 1, 0, 600_000);

        for (int i = 1; i <= 100; i++) {
            clock.advance(4 * 60_000); // always inside the last 5 minutes
            assertThat(a.placeBid("x", "b" + (i % 2), i)).isTrue();
        }

        assertThat(a.endTimeOf("x")).isEqualTo(clock.millis() + 5 * 60_000);
    }
}
