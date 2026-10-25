package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.CartService;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private CartService shop() {
        return Candidate.of(Solution::create);
    }

    private static void cart(CartService s, String cartId, String userId, String sku, int qty) {
        s.createCart(cartId, userId);
        s.addItem(cartId, sku, qty);
    }

    // ---- catalogue and cart lines ----

    @Test
    void shouldStoreProductsAndReportTheirStock() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 5);

        assertThat(s.stock("pen")).isEqualTo(5);
        s.setStock("pen", 12);
        assertThat(s.stock("pen")).isEqualTo(12);
        s.setStock("pen", 0);
        assertThat(s.stock("pen")).isZero();
    }

    @Test
    void shouldRejectInvalidCatalogueInput() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 5);

        assertThatThrownBy(() -> s.addProduct("pen", "Again", 100, "stationery", 1))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addProduct("a", "A", 0, "c", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addProduct("a", "A", -5, "c", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addProduct("a", "A", 5, "c", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addProduct(" ", "A", 5, "c", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addProduct("a", "", 5, "c", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addProduct("a", "A", 5, " ", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.setStock("pen", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.setStock("nope", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.stock("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.setTaxRate("stationery", 10_001)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.setTaxRate("stationery", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.stock("pen")).isEqualTo(5);
    }

    @Test
    void shouldAddToAnExistingLineInsteadOfCreatingASecondOne() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 10);
        s.createCart("c1", "u1");

        s.addItem("c1", "pen", 2);
        s.addItem("c1", "pen", 3);

        assertThat(s.quantity("c1", "pen")).isEqualTo(5);
        assertThat(s.total("c1")).isEqualTo("1250|0|0|0|1250");
    }

    @Test
    void shouldRejectAddingBeyondStockAndLeaveTheCartUnchanged() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 5);
        cart(s, "c1", "u1", "pen", 4);

        assertThatThrownBy(() -> s.addItem("c1", "pen", 2))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("insufficient stock");
        assertThat(s.quantity("c1", "pen")).isEqualTo(4);
        s.addItem("c1", "pen", 1);
        assertThat(s.quantity("c1", "pen")).isEqualTo(5);
        assertThatThrownBy(() -> s.addItem("c1", "pen", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.quantity("c1", "pen")).isEqualTo(5);
    }

    @Test
    void shouldNotReserveStockWhenItemsAreAddedToCarts() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 1);

        cart(s, "c1", "u1", "pen", 1);
        cart(s, "c2", "u2", "pen", 1);

        assertThat(s.stock("pen")).isEqualTo(1);
        assertThat(s.quantity("c1", "pen")).isEqualTo(1);
        assertThat(s.quantity("c2", "pen")).isEqualTo(1);
    }

    @Test
    void shouldSetQuantityExactlyAndRemoveTheLineAtZero() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 5);
        cart(s, "c1", "u1", "pen", 2);

        s.setQuantity("c1", "pen", 4);
        assertThat(s.quantity("c1", "pen")).isEqualTo(4);
        s.setQuantity("c1", "pen", 1);
        assertThat(s.quantity("c1", "pen")).isEqualTo(1);
        assertThatThrownBy(() -> s.setQuantity("c1", "pen", 6)).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.quantity("c1", "pen")).isEqualTo(1);
        assertThatThrownBy(() -> s.setQuantity("c1", "pen", -1)).isInstanceOf(IllegalArgumentException.class);
        s.setQuantity("c1", "pen", 0);
        assertThat(s.quantity("c1", "pen")).isZero();
        assertThat(s.total("c1")).isEqualTo("0|0|0|0|0");
        s.setQuantity("c1", "pen", 0); // nothing to remove: no error
        s.setQuantity("c1", "pen", 3); // creates the line
        assertThat(s.quantity("c1", "pen")).isEqualTo(3);
    }

    @Test
    void shouldRemoveALineAndRejectRemovingOneThatIsNotThere() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 5);
        s.addProduct("ink", "Ink", 400, "stationery", 5);
        cart(s, "c1", "u1", "pen", 2);

        s.removeItem("c1", "pen");

        assertThat(s.quantity("c1", "pen")).isZero();
        assertThatThrownBy(() -> s.removeItem("c1", "pen")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.removeItem("c1", "ink")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.removeItem("c1", "ghost")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectUnknownIdsDuplicateCartsAndBadQuantities() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 5);
        s.createCart("c1", "u1");

        assertThatThrownBy(() -> s.createCart("c1", "u2")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.createCart(" ", "u2")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.createCart("c2", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addItem("nope", "pen", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addItem("c1", "ghost", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addItem("c1", "pen", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addItem("c1", "pen", -2)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.total("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.couponStatus("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.checkout("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.quantity("c1", "pen")).isZero();
    }

    // ---- pricing ----

    @Test
    void shouldTotalAnEmptyCartAsZeros() {
        CartService s = shop();
        s.createCart("c1", "u1");

        assertThat(s.total("c1")).isEqualTo("0|0|0|0|0");
        assertThat(s.couponStatus("c1")).isEqualTo("NONE");
    }

    @Test
    void shouldGiveYUnitsFreeInEveryCompleteGroupOfXPlusY() {
        CartService s = shop();
        s.addProduct("tea", "Tea", 1000, "food", 100);
        s.setBuyXGetY("tea", 2, 1);
        for (int qty : new int[] {1, 2, 3, 5, 6, 7}) {
            s.createCart("c" + qty, "u" + qty);
            s.addItem("c" + qty, "tea", qty);
        }

        assertThat(s.total("c1")).isEqualTo("1000|0|0|0|1000");
        assertThat(s.total("c2")).isEqualTo("2000|0|0|0|2000");
        assertThat(s.total("c3")).isEqualTo("3000|1000|0|0|2000");
        assertThat(s.total("c5")).isEqualTo("5000|1000|0|0|4000");
        assertThat(s.total("c6")).isEqualTo("6000|2000|0|0|4000");
        assertThat(s.total("c7")).isEqualTo("7000|2000|0|0|5000");
    }

    @Test
    void shouldTakeAPercentOffTheLineSubtotalWithFloorDivision() {
        CartService s = shop();
        s.addProduct("gum", "Gum", 333, "food", 100);
        s.setPercentOff("gum", 10);
        cart(s, "c1", "u1", "gum", 3);
        cart(s, "c2", "u2", "gum", 1);

        assertThat(s.total("c1")).isEqualTo("999|99|0|0|900");
        assertThat(s.total("c2")).isEqualTo("333|33|0|0|300");
    }

    @Test
    void shouldKeepOnlyOnePromotionPerSkuAndAllowClearingIt() {
        CartService s = shop();
        s.addProduct("tea", "Tea", 1000, "food", 100);
        cart(s, "c1", "u1", "tea", 3);

        s.setBuyXGetY("tea", 2, 1);
        assertThat(s.total("c1")).isEqualTo("3000|1000|0|0|2000");
        s.setPercentOff("tea", 50);
        assertThat(s.total("c1")).isEqualTo("3000|1500|0|0|1500");
        s.clearPromotion("tea");
        assertThat(s.total("c1")).isEqualTo("3000|0|0|0|3000");
        s.clearPromotion("tea");
    }

    @Test
    void shouldRejectInvalidPromotions() {
        CartService s = shop();
        s.addProduct("tea", "Tea", 1000, "food", 100);

        assertThatThrownBy(() -> s.setPercentOff("tea", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.setPercentOff("tea", 91)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.setBuyXGetY("tea", 0, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.setBuyXGetY("tea", 2, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.setPercentOff("ghost", 10)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.setBuyXGetY("ghost", 1, 1)).isInstanceOf(IllegalArgumentException.class);
        cart(s, "c1", "u1", "tea", 3);
        assertThat(s.total("c1")).isEqualTo("3000|0|0|0|3000");
    }

    @Test
    void shouldTakeAPercentCouponOffThePostPromotionSubtotalWithFloorDivision() {
        CartService s = shop();
        s.addProduct("gum", "Gum", 333, "food", 100);
        s.defineCoupon("TEN", "PERCENT", 10, 0, 5);
        cart(s, "c1", "u1", "gum", 1);
        s.applyCoupon("c1", "TEN");

        assertThat(s.total("c1")).isEqualTo("333|0|33|0|300");

        s.setPercentOff("gum", 50); // promo first: 333 - 166 = 167, then floor(16.7) = 16
        assertThat(s.total("c1")).isEqualTo("333|166|16|0|151");
    }

    @Test
    void shouldCapAFixedCouponAtTheSubtotal() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 10);
        s.defineCoupon("OFF800", "FIXED", 800, 0, 5);
        cart(s, "c1", "u1", "pen", 2);
        cart(s, "c2", "u2", "pen", 4);
        s.applyCoupon("c1", "OFF800");
        s.applyCoupon("c2", "OFF800");

        assertThat(s.total("c1")).isEqualTo("500|0|500|0|0");
        assertThat(s.total("c2")).isEqualTo("1000|0|800|0|200");
    }

    @Test
    void shouldReplaceTheCartCouponWhenAnotherIsApplied() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 1000, "stationery", 10);
        s.defineCoupon("TEN", "PERCENT", 10, 0, 5);
        s.defineCoupon("FLAT50", "FIXED", 50, 0, 5);
        cart(s, "c1", "u1", "pen", 1);

        s.applyCoupon("c1", "TEN");
        assertThat(s.total("c1")).isEqualTo("1000|0|100|0|900");
        s.applyCoupon("c1", "FLAT50");
        assertThat(s.total("c1")).isEqualTo("1000|0|50|0|950");
        s.removeCoupon("c1");
        assertThat(s.couponStatus("c1")).isEqualTo("NONE");
        assertThat(s.total("c1")).isEqualTo("1000|0|0|0|1000");
        assertThatThrownBy(() -> s.applyCoupon("c1", "GHOST")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.applyCoupon("nope", "TEN")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldIgnoreACouponWhileTheMinimumSubtotalIsNotMet() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 1000, "stationery", 10);
        s.defineCoupon("BIG", "PERCENT", 10, 3000, 5);
        cart(s, "c1", "u1", "pen", 2);
        s.applyCoupon("c1", "BIG");

        assertThat(s.couponStatus("c1")).isEqualTo("NOT_ELIGIBLE");
        assertThat(s.total("c1")).isEqualTo("2000|0|0|0|2000");

        s.addItem("c1", "pen", 1); // exactly the minimum
        assertThat(s.couponStatus("c1")).isEqualTo("APPLIED");
        assertThat(s.total("c1")).isEqualTo("3000|0|300|0|2700");

        s.setQuantity("c1", "pen", 2);
        assertThat(s.couponStatus("c1")).isEqualTo("NOT_ELIGIBLE");
        assertThat(s.total("c1")).isEqualTo("2000|0|0|0|2000");
    }

    @Test
    void shouldMeasureTheMinimumSubtotalAfterPromotions() {
        CartService s = shop();
        s.addProduct("tea", "Tea", 1000, "food", 100);
        s.setBuyXGetY("tea", 1, 1);
        s.defineCoupon("BIG", "PERCENT", 10, 3000, 5);
        cart(s, "c1", "u1", "tea", 4);
        s.applyCoupon("c1", "BIG");

        assertThat(s.couponStatus("c1")).isEqualTo("NOT_ELIGIBLE"); // 4000 before promos, 2000 after
        assertThat(s.total("c1")).isEqualTo("4000|2000|0|0|2000");

        s.setQuantity("c1", "tea", 8);
        assertThat(s.couponStatus("c1")).isEqualTo("APPLIED");
        assertThat(s.total("c1")).isEqualTo("8000|4000|400|0|3600");
    }

    @Test
    void shouldTaxEachLineByItsCategoryAndFloorPerLine() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 10);
        s.addProduct("milk", "Milk", 199, "grocery", 10);
        s.addProduct("x1", "X1", 15, "small", 10);
        s.addProduct("x2", "X2", 15, "small", 10);
        s.setTaxRate("stationery", 725);
        s.setTaxRate("small", 1000);
        s.createCart("c1", "u1");
        s.addItem("c1", "pen", 3); // 750 * 7.25% = 54.375 -> 54
        s.addItem("c1", "milk", 1); // no rate for "grocery" -> 0
        s.createCart("c2", "u2");
        s.addItem("c2", "x1", 1); // 15 * 10% = 1.5 -> 1
        s.addItem("c2", "x2", 1); // 1.5 -> 1 (floored per line, not on the sum)

        assertThat(s.total("c1")).isEqualTo("949|0|0|54|1003");
        assertThat(s.total("c2")).isEqualTo("30|0|0|2|32");
    }

    @Test
    void shouldSpreadTheCouponOverLinesByLargestRemainderBeforeTaxing() {
        CartService s = shop();
        for (String sku : new String[] {"a", "b", "c"}) {
            s.addProduct(sku, sku, 100, "x", 10);
        }
        s.setTaxRate("x", 1000);
        s.defineCoupon("OFF100", "FIXED", 100, 0, 5);
        s.createCart("c1", "u1");
        for (String sku : new String[] {"c", "b", "a"}) {
            s.addItem("c1", sku, 1);
        }
        s.applyCoupon("c1", "OFF100");

        // shares 33.33 each: 34 to "a" (remainders tie, lowest sku first), 33, 33 -> taxable 66, 67, 67 -> tax 6 + 6 + 6
        assertThat(s.total("c1")).isEqualTo("300|0|100|18|218");
    }

    @Test
    void shouldComputeTheWorkedExampleFromTheReadme() {
        CartService s = shop();
        s.addProduct("A", "Anvil", 1000, "goods", 20);
        s.addProduct("B", "Bolt", 333, "food", 20);
        s.setTaxRate("goods", 1000);
        s.setTaxRate("food", 500);
        s.setBuyXGetY("A", 2, 1);
        s.setPercentOff("B", 10);
        s.defineCoupon("SAVE10", "PERCENT", 10, 5000, 3);
        s.createCart("c1", "u1");
        s.addItem("c1", "A", 7);
        s.addItem("c1", "B", 3);
        s.applyCoupon("c1", "SAVE10");

        assertThat(s.total("c1")).isEqualTo("7999|2099|590|490|5800");
        assertThat(s.couponStatus("c1")).isEqualTo("APPLIED");
    }

    // ---- coupons definitions ----

    @Test
    void shouldRejectInvalidCouponDefinitions() {
        CartService s = shop();
        s.defineCoupon("OK", "PERCENT", 10, 0, 1);

        assertThatThrownBy(() -> s.defineCoupon("OK", "FIXED", 5, 0, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.defineCoupon(" ", "FIXED", 5, 0, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.defineCoupon("A", "BOGUS", 5, 0, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.defineCoupon("A", "PERCENT", 0, 0, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.defineCoupon("A", "PERCENT", 101, 0, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.defineCoupon("A", "FIXED", 0, 0, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.defineCoupon("A", "FIXED", 5, -1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.defineCoupon("A", "FIXED", 5, 0, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.couponUses("GHOST")).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.couponUses("OK")).isZero();
    }

    // ---- checkout ----

    @Test
    void shouldCheckOutAndDecrementStockEmptyTheCartAndRecordTheOrder() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 5);
        s.setTaxRate("stationery", 1000);
        cart(s, "c1", "u1", "pen", 2);

        String order = s.checkout("c1");

        assertThat(order).isEqualTo("ORD-1");
        assertThat(s.stock("pen")).isEqualTo(3);
        assertThat(s.quantity("c1", "pen")).isZero();
        assertThat(s.total("c1")).isEqualTo("0|0|0|0|0");
        assertThat(s.orders("u1")).containsExactly("ORD-1|550");
        s.addItem("c1", "pen", 3); // the cart is reusable
        assertThatThrownBy(() -> s.addItem("c1", "pen", 1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNameTheFirstLackingSkuInSkuOrderAndChangeNothing() {
        CartService s = shop();
        for (String sku : new String[] {"zeta", "alpha", "mid"}) {
            s.addProduct(sku, sku, 100, "x", 5);
        }
        s.defineCoupon("TEN", "PERCENT", 10, 0, 5);
        s.createCart("c1", "u1");
        for (String sku : new String[] {"zeta", "alpha", "mid"}) {
            s.addItem("c1", sku, 5);
        }
        s.applyCoupon("c1", "TEN");
        s.setStock("zeta", 1);
        s.setStock("mid", 1);

        assertThatThrownBy(() -> s.checkout("c1"))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("insufficient stock")
                .hasMessageContaining("mid")
                .hasMessageNotContaining("zeta");
        assertThat(s.stock("alpha")).isEqualTo(5);
        assertThat(s.stock("mid")).isEqualTo(1);
        assertThat(s.stock("zeta")).isEqualTo(1);
        assertThat(s.quantity("c1", "alpha")).isEqualTo(5);
        assertThat(s.couponUses("TEN")).isZero();
        assertThat(s.couponStatus("c1")).isEqualTo("APPLIED");
        assertThat(s.orders("u1")).isEmpty();
    }

    @Test
    void shouldRejectCheckingOutAnEmptyCart() {
        CartService s = shop();
        s.createCart("c1", "u1");

        assertThatThrownBy(() -> s.checkout("c1")).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.orders("u1")).isEmpty();
    }

    @Test
    void shouldConsumeACouponUseOnlyOnSuccessfulCheckoutsAndIgnoreItOnceExhausted() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 1000, "stationery", 10);
        s.defineCoupon("ONCE", "PERCENT", 10, 0, 1);
        cart(s, "c1", "u1", "pen", 1);
        cart(s, "c2", "u2", "pen", 1);
        s.applyCoupon("c1", "ONCE");
        s.applyCoupon("c2", "ONCE");
        assertThat(s.couponStatus("c2")).isEqualTo("APPLIED");

        assertThat(s.checkout("c1")).isEqualTo("ORD-1");

        assertThat(s.couponUses("ONCE")).isEqualTo(1);
        assertThat(s.orders("u1")).containsExactly("ORD-1|900");
        assertThat(s.couponStatus("c1")).isEqualTo("NONE"); // checkout detaches the coupon
        assertThat(s.couponStatus("c2")).isEqualTo("NOT_ELIGIBLE");
        assertThat(s.total("c2")).isEqualTo("1000|0|0|0|1000");
        assertThat(s.checkout("c2")).isEqualTo("ORD-2");
        assertThat(s.couponUses("ONCE")).isEqualTo(1);
        assertThat(s.orders("u2")).containsExactly("ORD-2|1000");
    }

    @Test
    void shouldCheckOutWithoutTheCouponWhenItBecameIneligibleAndNotConsumeIt() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 1000, "stationery", 10);
        s.defineCoupon("BIG", "FIXED", 500, 2000, 5);
        cart(s, "c1", "u1", "pen", 2);
        s.applyCoupon("c1", "BIG");
        assertThat(s.total("c1")).isEqualTo("2000|0|500|0|1500");

        s.setQuantity("c1", "pen", 1);
        s.checkout("c1");

        assertThat(s.couponUses("BIG")).isZero();
        assertThat(s.orders("u1")).containsExactly("ORD-1|1000");
    }

    @Test
    void shouldListOrdersPerUserInCheckoutOrder() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 50);
        s.createCart("c1", "u1");
        s.createCart("c2", "u2");
        s.createCart("c3", "u1");

        s.addItem("c1", "pen", 1);
        s.addItem("c2", "pen", 2);
        s.addItem("c3", "pen", 3);
        s.checkout("c3");
        s.checkout("c2");
        s.checkout("c1");

        assertThat(s.orders("u1")).containsExactly("ORD-1|750", "ORD-3|250");
        assertThat(s.orders("u2")).containsExactly("ORD-2|500");
        assertThat(s.orders("nobody")).isEmpty();
        assertThatThrownBy(() -> s.orders(" ")).isInstanceOf(IllegalArgumentException.class);
    }

    // ---- concurrency ----

    @Test
    void shouldLetExactlyOneOfManyCartsCheckOutTheLastUnit() {
        CartService s = shop();
        s.addProduct("ps5", "Console", 49_900, "tech", 1);
        int carts = 16;
        for (int i = 0; i < carts; i++) {
            cart(s, "c" + i, "u" + i, "ps5", 1);
        }

        List<Boolean> results = Concurrently.collect(carts, i -> {
            try {
                s.checkout("c" + i);
                return true;
            } catch (IllegalArgumentException e) {
                return false;
            }
        });

        assertThat(results.stream().filter(b -> b).count()).isEqualTo(1);
        assertThat(s.stock("ps5")).isZero();
        int orders = 0;
        for (int i = 0; i < carts; i++) {
            orders += s.orders("u" + i).size();
        }
        assertThat(orders).isEqualTo(1);
    }

    @Test
    void shouldNeverOversellUnderConcurrentCheckouts() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 250, "stationery", 30);
        int carts = 40;
        for (int i = 0; i < carts; i++) {
            cart(s, "c" + i, "u" + i, "pen", 1 + i % 2);
        }

        List<Integer> sold = Concurrently.collect(carts, i -> {
            try {
                s.checkout("c" + i);
                return 1 + i % 2;
            } catch (IllegalArgumentException e) {
                return 0;
            }
        });

        int units = sold.stream().mapToInt(Integer::intValue).sum();
        assertThat(units).isLessThanOrEqualTo(30);
        assertThat(s.stock("pen")).isEqualTo(30 - units);
        assertThat(units).isGreaterThanOrEqualTo(29); // only a 2-unit cart can be left stranded by 1 unit
    }

    @Test
    void shouldHonourACouponMaxUsesUnderConcurrentCheckouts() {
        CartService s = shop();
        s.addProduct("pen", "Blue pen", 1000, "stationery", 100);
        s.defineCoupon("THREE", "PERCENT", 10, 0, 3);
        int carts = 12;
        for (int i = 0; i < carts; i++) {
            cart(s, "c" + i, "u" + i, "pen", 1);
            s.applyCoupon("c" + i, "THREE");
        }

        Concurrently.run(carts, i -> s.checkout("c" + i));

        assertThat(s.couponUses("THREE")).isEqualTo(3);
        List<String> all = new ArrayList<>();
        for (int i = 0; i < carts; i++) {
            all.addAll(s.orders("u" + i));
        }
        assertThat(all).hasSize(carts);
        assertThat(all.stream().filter(o -> o.endsWith("|900")).count()).isEqualTo(3);
        assertThat(all.stream().filter(o -> o.endsWith("|1000")).count()).isEqualTo(9);
    }
}
