package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.AuctionApi;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {

    private static final long MIN = 60_000;
    private final FakeClock clock = new FakeClock(1_000_000);

    private AuctionApi auctions() {
        AuctionApi a = Candidate.of(() -> Solution.create(clock));
        a.createAuction("a1", "sam", 100, 500, clock.millis() + 60 * MIN);
        return a;
    }

    @Test
    void shouldAcceptBidsThatBeatTheStartingPriceAndTheCurrentLeader() {
        AuctionApi a = auctions();

        assertThat(a.placeBid("a1", "ann", 99)).isFalse(); // below the starting price
        assertThat(a.placeBid("a1", "ann", 100)).isTrue();
        assertThat(a.placeBid("a1", "bob", 100)).isFalse(); // not strictly higher
        assertThat(a.placeBid("a1", "bob", 150)).isTrue();
        assertThat(a.placeBid("a1", "ann", 120)).isFalse();

        assertThat(a.highestBid("a1")).hasValue("bob:150");
        assertThat(a.bidHistory("a1")).containsExactly("ann:100", "bob:150");
    }

    @Test
    void shouldLetTheLeaderRaiseTheirOwnBid() {
        AuctionApi a = auctions();
        a.placeBid("a1", "ann", 100);

        assertThat(a.placeBid("a1", "ann", 200)).isTrue();

        assertThat(a.highestBid("a1")).hasValue("ann:200");
        assertThat(a.notifications("ann")).isEmpty();
    }

    @Test
    void shouldRejectInvalidBidsAndAuctions() {
        AuctionApi a = auctions();

        assertThatThrownBy(() -> a.placeBid("nope", "ann", 100)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> a.placeBid("a1", "ann", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> a.placeBid("a1", "sam", 500)).isInstanceOf(SecurityException.class);
        assertThatThrownBy(() -> a.createAuction("a1", "x", 1, 0, clock.millis() + MIN)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> a.createAuction("a2", "x", 0, 0, clock.millis() + MIN)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> a.createAuction("a3", "x", 1, 0, clock.millis())).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNotAcceptBidsAfterTheEnd() {
        AuctionApi a = auctions();
        a.placeBid("a1", "ann", 100);

        clock.advance(60 * MIN);

        assertThatThrownBy(() -> a.placeBid("a1", "bob", 500)).isInstanceOf(IllegalStateException.class);
        assertThat(a.highestBid("a1")).hasValue("ann:100");
    }

    @Test
    void shouldExtendTheEndWhenABidArrivesInTheLastFiveMinutes() {
        AuctionApi a = auctions();
        long end = a.endTimeOf("a1");

        clock.advance(54 * MIN); // 6 minutes left: no extension
        a.placeBid("a1", "ann", 100);
        assertThat(a.endTimeOf("a1")).isEqualTo(end);

        clock.advance(2 * MIN); // 4 minutes left
        a.placeBid("a1", "bob", 110);
        assertThat(a.endTimeOf("a1")).isEqualTo(clock.millis() + 5 * MIN);

        clock.advance(4 * MIN); // the old end has passed, the new one has not
        assertThat(a.placeBid("a1", "ann", 120)).isTrue();
        assertThat(a.endTimeOf("a1")).isEqualTo(clock.millis() + 5 * MIN);
    }

    @Test
    void shouldNotExtendOnARejectedBid() {
        AuctionApi a = auctions();
        a.placeBid("a1", "ann", 200);
        clock.advance(58 * MIN);
        long end = a.endTimeOf("a1");

        assertThat(a.placeBid("a1", "bob", 150)).isFalse();

        assertThat(a.endTimeOf("a1")).isEqualTo(end);
    }

    @Test
    void shouldReportTheResultOnlyAfterTheEnd() {
        AuctionApi a = auctions();
        a.placeBid("a1", "ann", 600);

        assertThatThrownBy(() -> a.result("a1")).isInstanceOf(IllegalStateException.class);
        clock.advance(60 * MIN);

        assertThat(a.result("a1")).isEqualTo("SOLD:ann:600");
    }

    @Test
    void shouldNotSellBelowTheReserveOrWithoutBids() {
        AuctionApi a = auctions();
        a.createAuction("a2", "sam", 10, 0, clock.millis() + 60 * MIN);
        a.placeBid("a1", "ann", 400);

        clock.advance(60 * MIN);

        assertThat(a.result("a1")).isEqualTo("RESERVE_NOT_MET");
        assertThat(a.result("a2")).isEqualTo("NO_BIDS");
    }

    @Test
    void shouldNotifyOutbidUsersEachTimeTheyLoseTheLead() {
        AuctionApi a = auctions();
        a.placeBid("a1", "ann", 100);
        a.placeBid("a1", "bob", 150);
        a.placeBid("a1", "ann", 200);
        a.placeBid("a1", "cy", 250);

        assertThat(a.notifications("ann")).containsExactly("OUTBID:a1:150", "OUTBID:a1:250");
        assertThat(a.notifications("bob")).containsExactly("OUTBID:a1:200");
        assertThat(a.notifications("cy")).isEmpty();
    }

    @Test
    void shouldOnlyCancelWithoutBidsAndOnlyAsTheSeller() {
        AuctionApi a = auctions();
        a.createAuction("a2", "sam", 10, 0, clock.millis() + 60 * MIN);
        a.placeBid("a1", "ann", 100);

        assertThatThrownBy(() -> a.cancel("a1", "sam")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> a.cancel("a2", "ann")).isInstanceOf(SecurityException.class);
        a.cancel("a2", "sam");
        assertThatThrownBy(() -> a.placeBid("a2", "bob", 50)).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldKeepAStrictlyIncreasingHistoryUnderConcurrentBidding() {
        AuctionApi a = Candidate.of(() -> Solution.create(clock));
        a.createAuction("race", "sam", 1, 0, clock.millis() + 60 * MIN);
        AtomicInteger accepted = new AtomicInteger();

        Concurrently.run(32, t -> {
            for (int i = 1; i <= 100; i++) {
                if (a.placeBid("race", "bidder" + t, (long) i * 32 + t)) {
                    accepted.incrementAndGet();
                }
            }
        });

        List<String> history = a.bidHistory("race");
        assertThat(history).hasSize(accepted.get());
        long previous = 0;
        for (String h : history) {
            long amount = Long.parseLong(h.substring(h.indexOf(':') + 1));
            assertThat(amount).isGreaterThan(previous);
            previous = amount;
        }
        assertThat(a.highestBid("race")).hasValue("bidder31:3231"); // 100 * 32 + 31
        assertThat(history.get(history.size() - 1)).isEqualTo("bidder31:3231");
    }

    @Test
    void shouldNotifyConsistentlyUnderConcurrentBidding() {
        AuctionApi a = Candidate.of(() -> Solution.create(clock));
        a.createAuction("race", "sam", 1, 0, clock.millis() + 60 * MIN);

        Concurrently.run(8, t -> {
            for (int i = 1; i <= 200; i++) {
                a.placeBid("race", "b" + t, (long) i * 8 + t);
            }
        });

        // every accepted bid except the first outbids exactly one previous leader (or the same bidder raising)
        int outbids = 0;
        for (int t = 0; t < 8; t++) {
            outbids += a.notifications("b" + t).size();
        }
        List<String> history = a.bidHistory("race");
        int selfRaises = 0;
        for (int i = 1; i < history.size(); i++) {
            if (history.get(i).split(":")[0].equals(history.get(i - 1).split(":")[0])) {
                selfRaises++;
            }
        }
        assertThat(outbids).isEqualTo(history.size() - 1 - selfRaises);
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            AuctionApi a = Candidate.of(() -> Solution.create(clock));
            a.createAuction("r", "sam", 1, 0, clock.millis() + 60 * MIN);

            Concurrently.run(8, t -> a.placeBid("r", "b" + t, 10 + t));

            assertThat(a.highestBid("r")).as("run %d", run).hasValue("b7:17");
            assertThat(a.bidHistory("r").get(a.bidHistory("r").size() - 1)).isEqualTo("b7:17");
        }
    }
}
