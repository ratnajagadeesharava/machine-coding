package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.CartService;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldAllocateTheCouponByLargestRemainderBeforeAnyLowestSkuTieBreak() {
        CartService s = Candidate.of(Solution::create);
        s.addProduct("a", "A", 1000, "x", 5);
        s.addProduct("b", "B", 1000, "x", 5);
        s.addProduct("c", "C", 1100, "x", 5);
        s.setTaxRate("x", 1000);
        s.defineCoupon("OFF1000", "FIXED", 1000, 0, 1);
        s.createCart("c1", "u1");
        s.addItem("c1", "a", 1);
        s.addItem("c1", "b", 1);
        s.addItem("c1", "c", 1);
        s.applyCoupon("c1", "OFF1000");

        // floors 322, 322, 354 (sum 998); remainders 1800, 1800, 2600: "c" gets one cent, then "a" (tie, lowest sku)
        // shares 323, 322, 355 -> taxable 677, 678, 745 -> tax 67 + 67 + 74
        assertThat(s.total("c1")).isEqualTo("3100|0|1000|208|2308");
    }

    @Test
    void shouldAllocateAPercentCouponAndKeepTheTotalConsistentWithItsParts() {
        CartService s = Candidate.of(Solution::create);
        s.addProduct("a", "A", 333, "x", 50);
        s.addProduct("b", "B", 777, "y", 50);
        s.setTaxRate("x", 1800);
        s.setTaxRate("y", 575);
        s.defineCoupon("P7", "PERCENT", 7, 0, 5);
        s.createCart("c1", "u1");
        s.addItem("c1", "a", 3); // 999
        s.addItem("c1", "b", 2); // 1554
        s.applyCoupon("c1", "P7");

        // subtotal 2553, coupon floor(178.71) = 178.
        // shares: a 178*999/2553 = 69.65 -> 69 (rem 0.65), b 178*1554/2553 = 108.35 -> 108 (rem 0.35); 1 left -> a: 70, 108
        // taxable 929 and 1446: tax floor(167.22) = 167 and floor(83.145) = 83
        assertThat(s.total("c1")).isEqualTo("2553|0|178|250|2625");
    }

    @Test
    void shouldCombinePromotionCouponAndTaxOnSeveralLines() {
        CartService s = Candidate.of(Solution::create);
        s.addProduct("A", "A", 999, "x", 100);
        s.addProduct("B", "B", 1, "y", 100);
        s.setBuyXGetY("A", 3, 2);
        s.setPercentOff("B", 90);
        s.setTaxRate("x", 2000);
        s.defineCoupon("FIX", "FIXED", 123, 1000, 2);
        s.createCart("c1", "u1");
        s.addItem("c1", "A", 11); // 10989, 2 groups of 5 -> 4 free = 3996 -> 6993
        s.addItem("c1", "B", 5); // 5, 90% -> floor(4.5) = 4 -> 1
        s.applyCoupon("c1", "FIX");

        // S = 6994, coupon 123; shares a: 123*6993/6994 = 122.98 -> 122, b: 0 (rem .017); 1 left -> a (rem .98): 123, 0
        // taxable a 6870 -> 1374, b 1 at 0 bp -> 0
        assertThat(s.total("c1")).isEqualTo("10994|4000|123|1374|8245");
    }

    @Test
    void shouldUseLongArithmeticForLargeAmounts() {
        CartService s = Candidate.of(Solution::create);
        s.addProduct("gold", "Gold bar", 999_999L, "lux", 5_000);
        s.setTaxRate("lux", 10_000);
        s.defineCoupon("HALF", "PERCENT", 50, 0, 1);
        s.createCart("c1", "u1");
        s.addItem("c1", "gold", 1_000);
        s.applyCoupon("c1", "HALF");

        // 999_999_000 subtotal (more than an int holds once multiplied by 10_000); the rest is taxed at 100%
        assertThat(s.total("c1")).isEqualTo("999999000|0|499999500|499999500|999999000");
    }

    @Test
    void shouldGiveEachConcurrentCheckoutADistinctSequentialOrderId() {
        CartService s = Candidate.of(Solution::create);
        s.addProduct("pen", "Blue pen", 250, "stationery", 1_000);
        int carts = 40;
        for (int i = 0; i < carts; i++) {
            s.createCart("c" + i, "u" + i);
            s.addItem("c" + i, "pen", 2);
        }

        List<String> ids = Concurrently.collect(carts, i -> s.checkout("c" + i));

        Set<String> expected = new HashSet<>();
        for (int i = 1; i <= carts; i++) {
            expected.add("ORD-" + i);
        }
        assertThat(new HashSet<>(ids)).isEqualTo(expected);
        assertThat(s.stock("pen")).isEqualTo(1_000 - 2 * carts);
    }

    @Test
    void shouldNeverDeadlockWhenCartsHoldTheSameSkusInOppositeOrders() {
        CartService s = Candidate.of(Solution::create);
        s.addProduct("a", "A", 100, "x", 10_000);
        s.addProduct("b", "B", 100, "x", 10_000);
        int carts = 16;
        for (int i = 0; i < carts; i++) {
            s.createCart("c" + i, "u" + i);
        }

        Concurrently.run(carts, 60_000, t -> {
            String cid = "c" + t;
            for (int round = 0; round < 300; round++) {
                if (t % 2 == 0) {
                    s.addItem(cid, "a", 1);
                    s.addItem(cid, "b", 1);
                } else {
                    s.addItem(cid, "b", 1);
                    s.addItem(cid, "a", 1);
                }
                s.checkout(cid);
            }
        });

        assertThat(s.stock("a")).isEqualTo(10_000 - carts * 300);
        assertThat(s.stock("b")).isEqualTo(10_000 - carts * 300);
        int orders = 0;
        for (int i = 0; i < carts; i++) {
            orders += s.orders("u" + i).size();
        }
        assertThat(orders).isEqualTo(carts * 300);
    }

    @Test
    void shouldNotLetAddItemOvershootStockWhenManyThreadsAddToTheSameCart() {
        CartService s = Candidate.of(Solution::create);
        s.addProduct("pen", "Blue pen", 250, "stationery", 500);
        s.createCart("c1", "u1");
        AtomicInteger accepted = new AtomicInteger();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 100; i++) {
                try {
                    s.addItem("c1", "pen", 1);
                    accepted.incrementAndGet();
                } catch (IllegalArgumentException expected) {
                    // out of stock
                }
            }
        });

        assertThat(accepted.get()).isEqualTo(500);
        assertThat(s.quantity("c1", "pen")).isEqualTo(500);
        assertThat(s.total("c1")).isEqualTo("125000|0|0|0|125000");
    }

    @Test
    void shouldKeepAFailedCheckoutAtomicWhileOthersCheckOutTheSameSkus() {
        CartService s = Candidate.of(Solution::create);
        s.addProduct("a", "A", 100, "x", 100);
        s.addProduct("b", "B", 100, "x", 5);
        s.defineCoupon("C", "FIXED", 10, 0, 1_000);
        int carts = 24;
        for (int i = 0; i < carts; i++) {
            s.createCart("c" + i, "u" + i);
            s.addItem("c" + i, "a", 1);
            s.addItem("c" + i, "b", 1);
            s.applyCoupon("c" + i, "C");
        }

        List<Boolean> ok = Concurrently.collect(carts, i -> {
            try {
                s.checkout("c" + i);
                return true;
            } catch (IllegalArgumentException e) {
                assertThat(e.getMessage()).contains("insufficient stock").contains("b");
                return false;
            }
        });

        assertThat(ok.stream().filter(b -> b).count()).isEqualTo(5);
        assertThat(s.stock("b")).isZero();
        assertThat(s.stock("a")).isEqualTo(95); // failed checkouts took nothing from "a"
        assertThat(s.couponUses("C")).isEqualTo(5);
        for (int i = 0; i < carts; i++) {
            if (!ok.get(i)) {
                assertThat(s.quantity("c" + i, "a")).isEqualTo(1);
                assertThat(s.orders("u" + i)).isEmpty();
            }
        }
    }
}
