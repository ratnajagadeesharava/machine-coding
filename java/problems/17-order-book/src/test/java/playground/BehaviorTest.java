package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

import contract.OrderBookApi;
import java.time.Duration;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

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
    void shouldReportAnEmptyBook() {
        OrderBookApi b = book();

        assertThat(b.bestBid()).isEmpty();
        assertThat(b.bestAsk()).isEmpty();
        assertThat(b.trades()).isEmpty();
        assertThat(b.volumeAt("BUY", 100)).isZero();
        assertThat(b.remaining(1)).isEmpty();
    }

    @Test
    void shouldRestNonCrossingOrdersAndReportTheBestPrices() {
        OrderBookApi b = book();
        long b1 = buy(b, 99, 10);
        buy(b, 98, 5);
        sell(b, 101, 7);
        sell(b, 102, 1);

        assertThat(b.bestBid()).hasValue(99);
        assertThat(b.bestAsk()).hasValue(101);
        assertThat(b.volumeAt("BUY", 99)).isEqualTo(10);
        assertThat(b.volumeAt("SELL", 101)).isEqualTo(7);
        assertThat(b.volumeAt("BUY", 101)).isZero();
        assertThat(b.remaining(b1)).hasValue(10);
        assertThat(b.trades()).isEmpty();
    }

    @Test
    void shouldTradeAtTheRestingSellPriceWhenAnIncomingBuyIsHigher() {
        OrderBookApi b = book();
        long s = sell(b, 100, 5);

        long buyer = buy(b, 105, 5);

        assertThat(b.trades()).containsExactly(trade(buyer, s, 100, 5));
        assertThat(b.bestAsk()).isEmpty();
        assertThat(b.bestBid()).isEmpty();
        assertThat(b.remaining(s)).hasValue(0);
        assertThat(b.remaining(buyer)).hasValue(0);
    }

    @Test
    void shouldTradeAtTheRestingBuyPriceWhenAnIncomingSellIsLower() {
        OrderBookApi b = book();
        long buyer = buy(b, 100, 5);

        long seller = sell(b, 90, 5);

        assertThat(b.trades()).containsExactly(trade(buyer, seller, 100, 5));
    }

    @Test
    void shouldTradeWhenPricesAreEqual() {
        OrderBookApi b = book();
        long s = sell(b, 100, 3);

        long buyer = buy(b, 100, 3);

        assertThat(b.trades()).containsExactly(trade(buyer, s, 100, 3));
    }

    @Test
    void shouldNotTradeWhenPricesDoNotCross() {
        OrderBookApi b = book();
        sell(b, 101, 3);
        buy(b, 100, 3);

        assertThat(b.trades()).isEmpty();
        assertThat(b.bestBid()).hasValue(100);
        assertThat(b.bestAsk()).hasValue(101);
    }

    @Test
    void shouldRestTheRemainderOfAPartiallyFilledIncomingOrder() {
        OrderBookApi b = book();
        long s = sell(b, 100, 5);

        long buyer = buy(b, 105, 8);

        assertThat(b.trades()).containsExactly(trade(buyer, s, 100, 5));
        assertThat(b.remaining(buyer)).hasValue(3);
        assertThat(b.bestBid()).hasValue(105);
        assertThat(b.volumeAt("BUY", 105)).isEqualTo(3);
        assertThat(b.bestAsk()).isEmpty();
    }

    @Test
    void shouldKeepTheRemainderOfAPartiallyFilledRestingOrder() {
        OrderBookApi b = book();
        long s = sell(b, 100, 10);

        long buyer = buy(b, 100, 4);

        assertThat(b.trades()).containsExactly(trade(buyer, s, 100, 4));
        assertThat(b.remaining(s)).hasValue(6);
        assertThat(b.volumeAt("SELL", 100)).isEqualTo(6);
        assertThat(b.bestBid()).isEmpty();
        assertThat(b.remaining(buyer)).hasValue(0);
    }

    @Test
    void shouldPreferTheBestPriceAndSweepSeveralLevels() {
        OrderBookApi b = book();
        long s102 = sell(b, 102, 1);
        long s101 = sell(b, 101, 1);
        long s103 = sell(b, 103, 1);

        long buyer = buy(b, 102, 3);

        assertThat(b.trades()).containsExactly(trade(buyer, s101, 101, 1), trade(buyer, s102, 102, 1));
        assertThat(b.remaining(buyer)).hasValue(1);
        assertThat(b.bestBid()).hasValue(102);
        assertThat(b.bestAsk()).hasValue(103);
        assertThat(b.remaining(s103)).hasValue(1);
    }

    @Test
    void shouldServeTheEarliestOrderFirstAtTheSamePrice() {
        OrderBookApi b = book();
        long s1 = sell(b, 100, 2);
        long s2 = sell(b, 100, 2);

        long buyer = buy(b, 100, 3);

        assertThat(b.trades()).containsExactly(trade(buyer, s1, 100, 2), trade(buyer, s2, 100, 1));
        assertThat(b.remaining(s1)).hasValue(0);
        assertThat(b.remaining(s2)).hasValue(1);
        assertThat(b.volumeAt("SELL", 100)).isEqualTo(1);
    }

    @Test
    void shouldCancelARestingOrder() {
        OrderBookApi b = book();
        long b1 = buy(b, 99, 10);
        buy(b, 98, 5);

        assertThat(b.cancel(b1)).isTrue();

        assertThat(b.remaining(b1)).hasValue(0);
        assertThat(b.volumeAt("BUY", 99)).isZero();
        assertThat(b.bestBid()).hasValue(98);
    }

    @Test
    void shouldCancelOnlyTheRemainingPartOfAPartiallyFilledOrder() {
        OrderBookApi b = book();
        long s = sell(b, 100, 10);
        long buyer = buy(b, 100, 4);

        assertThat(b.cancel(s)).isTrue();

        assertThat(b.bestAsk()).isEmpty();
        assertThat(b.trades()).containsExactly(trade(buyer, s, 100, 4));
    }

    @Test
    void shouldRefuseToCancelUnknownFilledOrAlreadyCancelledOrders() {
        OrderBookApi b = book();
        long s = sell(b, 100, 1);
        long buyer = buy(b, 100, 1);
        long resting = buy(b, 50, 1);
        b.cancel(resting);

        assertThat(b.cancel(s)).isFalse();
        assertThat(b.cancel(buyer)).isFalse();
        assertThat(b.cancel(resting)).isFalse();
        assertThat(b.cancel(9_999)).isFalse();
    }

    @Test
    void shouldNeverMatchACancelledOrderAndPromoteTheNextInLine() {
        OrderBookApi b = book();
        long s1 = sell(b, 100, 5);
        long s2 = sell(b, 100, 5);
        b.cancel(s1);

        long buyer = buy(b, 100, 5);

        assertThat(b.trades()).containsExactly(trade(buyer, s2, 100, 5));
    }

    @Test
    void shouldRefuseInvalidOrdersWithoutChangingTheBook() {
        OrderBookApi b = book();

        assertThat(b.submit("HOLD", 100, 1)).isEmpty();
        assertThat(b.submit("buy", 100, 1)).isEmpty();
        assertThat(b.submit("BUY", 0, 1)).isEmpty();
        assertThat(b.submit("BUY", -5, 1)).isEmpty();
        assertThat(b.submit("SELL", 100, 0)).isEmpty();
        assertThat(b.submit("SELL", 100, -3)).isEmpty();

        assertThat(b.bestBid()).isEmpty();
        assertThat(b.bestAsk()).isEmpty();
        assertThat(b.volumeAt("BUY", 100)).isZero();
        assertThat(b.volumeAt("SELL", 100)).isZero();
    }

    @Test
    void shouldAggregateVolumePerLevelAndSideAndIgnoreInvalidSides() {
        OrderBookApi b = book();
        buy(b, 90, 4);
        long second = buy(b, 90, 6);
        buy(b, 80, 1);
        sell(b, 110, 2);

        assertThat(b.volumeAt("BUY", 90)).isEqualTo(10);
        b.cancel(second);
        assertThat(b.volumeAt("BUY", 90)).isEqualTo(4);
        assertThat(b.volumeAt("SELL", 90)).isZero();
        assertThat(b.volumeAt("SELL", 110)).isEqualTo(2);
        assertThat(b.volumeAt("HOLD", 90)).isZero();
    }

    @Test
    void shouldFallBackToTheNextLevelWhenTheBestLevelIsExhausted() {
        OrderBookApi b = book();
        buy(b, 100, 1);
        buy(b, 99, 1);
        buy(b, 98, 1);

        sell(b, 99, 2);

        assertThat(b.bestBid()).hasValue(98);
        assertThat(b.trades()).hasSize(2);
    }

    @Test
    void shouldGiveEachOrderAUniqueIdAndReturnTradesAsASnapshot() {
        OrderBookApi b = book();
        long a = buy(b, 100, 1);
        long c = sell(b, 100, 1);
        long d = buy(b, 10, 1);
        List<String> snapshot = b.trades();

        sell(b, 10, 1);

        assertThat(List.of(a, c, d)).doesNotHaveDuplicates();
        assertThat(snapshot).hasSize(1);
        assertThat(b.trades()).hasSize(2);
    }

    @Test
    void shouldStayConsistentUnderARandomWorkload() {
        OrderBookApi b = book();
        long seed = 12345;
        Map<Long, Long> quantities = new HashMap<>();
        List<Long> ids = new ArrayList<>();
        Map<Long, Boolean> cancelled = new HashMap<>();
        for (int i = 0; i < 3000; i++) {
            seed = seed * 6364136223846793005L + 1442695040888963407L;
            int r = (int) ((seed >>> 33) % 1000);
            if (r < 150 && !ids.isEmpty()) {
                long id = ids.get((int) ((seed >>> 20) % ids.size()));
                if (b.cancel(id)) {
                    cancelled.put(id, true);
                }
            } else {
                String side = r % 2 == 0 ? "BUY" : "SELL";
                long price = 95 + (r / 2) % 11;
                long qty = 1 + (r / 7) % 5;
                long id = b.submit(side, price, qty).orElseThrow();
                ids.add(id);
                quantities.put(id, qty);
            }
            if (b.bestBid().isPresent() && b.bestAsk().isPresent()) {
                assertThat(b.bestBid().getAsLong()).isLessThan(b.bestAsk().getAsLong());
            }
        }

        Map<Long, Long> traded = new HashMap<>();
        for (String t : b.trades()) {
            String[] p = t.split(":");
            long qty = Long.parseLong(p[3]);
            traded.merge(Long.parseLong(p[0]), qty, Long::sum);
            traded.merge(Long.parseLong(p[1]), qty, Long::sum);
        }
        for (long id : ids) {
            long remaining = b.remaining(id).orElseThrow();
            if (cancelled.containsKey(id)) {
                assertThat(remaining).isZero();
            } else {
                assertThat(remaining).as("order %d", id).isEqualTo(quantities.get(id) - traded.getOrDefault(id, 0L));
            }
        }
    }

    @Test
    void shouldHandleTwoHundredThousandOperationsWithoutScanningTheBook() {
        OrderBookApi b = book();
        assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            long[] ids = new long[100_000];
            for (int i = 0; i < ids.length; i++) {
                ids[i] = b.submit("BUY", 1 + i % 50_000, 1).orElseThrow();
            }
            for (int i = 50_000; i < ids.length; i++) {
                if (!b.cancel(ids[i])) {
                    throw new AssertionError("cancel failed for " + ids[i]);
                }
            }
            assertThat(b.bestBid()).hasValue(50_000);
            for (int i = 0; i < 50_000; i++) {
                b.submit("SELL", 50_001 + i, 1).orElseThrow();
            }
            assertThat(b.bestAsk()).hasValue(50_001);
            assertThat(b.volumeAt("BUY", 25_000)).isEqualTo(1);

            long sweeper = b.submit("SELL", 1, 50_000).orElseThrow();

            assertThat(b.bestBid()).isEmpty();
            List<String> trades = b.trades();
            assertThat(trades).hasSize(50_000);
            assertThat(trades.get(0)).isEqualTo(ids[49_999] + ":" + sweeper + ":50000:1");
            assertThat(trades.get(49_999)).isEqualTo(ids[0] + ":" + sweeper + ":1:1");
        });
    }
}
