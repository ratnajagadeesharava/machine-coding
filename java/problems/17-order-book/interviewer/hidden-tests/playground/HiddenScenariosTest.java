package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.OrderBookApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private OrderBookApi book() {
        return Candidate.of(Solution::create);
    }

    private static long buy(OrderBookApi b, long price, long qty) {
        return b.submit("BUY", price, qty).orElseThrow();
    }

    private static long sell(OrderBookApi b, long price, long qty) {
        return b.submit("SELL", price, qty).orElseThrow();
    }

    private static String trade(long buyId, long sellId, long price, long qty) {
        return buyId + ":" + sellId + ":" + price + ":" + qty;
    }

    @Test
    void sweepShouldEndWithTheRemainderRestingAtTheIncomingLimit() {
        OrderBookApi b = book();
        long s1 = sell(b, 101, 2);
        long s2 = sell(b, 102, 2);
        sell(b, 103, 2);

        long buyer = buy(b, 102, 5);

        assertThat(b.trades()).containsExactly(trade(buyer, s1, 101, 2), trade(buyer, s2, 102, 2));
        assertThat(b.bestBid()).hasValue(102);
        assertThat(b.volumeAt("BUY", 102)).isEqualTo(1);
        assertThat(b.bestAsk()).hasValue(103);
    }

    @Test
    void incomingSellShouldSweepBidsFromHighToLowAtTheBidPrices() {
        OrderBookApi b = book();
        long b98 = buy(b, 98, 1);
        long b100 = buy(b, 100, 1);
        long b99 = buy(b, 99, 1);

        long seller = sell(b, 98, 3);

        assertThat(b.trades())
                .containsExactly(trade(b100, seller, 100, 1), trade(b99, seller, 99, 1), trade(b98, seller, 98, 1));
        assertThat(b.bestBid()).isEmpty();
        assertThat(b.bestAsk()).isEmpty();
    }

    @Test
    void aPartiallyFilledRestingOrderShouldKeepItsPlaceInTheQueue() {
        OrderBookApi b = book();
        long s1 = sell(b, 100, 4);
        long s2 = sell(b, 100, 4);

        long first = buy(b, 100, 1);
        long second = buy(b, 100, 4);

        assertThat(b.trades())
                .containsExactly(trade(first, s1, 100, 1), trade(second, s1, 100, 3), trade(second, s2, 100, 1));
        assertThat(b.remaining(s2)).hasValue(3);
    }

    @Test
    void cancellingTheMiddleOfAQueueShouldLeaveTheOthersInOrder() {
        OrderBookApi b = book();
        long s1 = sell(b, 100, 1);
        long s2 = sell(b, 100, 1);
        long s3 = sell(b, 100, 1);
        b.cancel(s2);

        long buyer = buy(b, 100, 2);

        assertThat(b.trades()).containsExactly(trade(buyer, s1, 100, 1), trade(buyer, s3, 100, 1));
        assertThat(b.volumeAt("SELL", 100)).isZero();
    }

    @Test
    void remainingShouldDistinguishFilledFromNeverIssued() {
        OrderBookApi b = book();
        long s = sell(b, 100, 1);
        buy(b, 100, 1);

        assertThat(b.remaining(s)).hasValue(0);
        assertThat(b.remaining(123_456)).isEmpty();
    }
}
