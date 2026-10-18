package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Coupon;
import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Checkout;
import solution.FlatCoupon;
import solution.MinSpendCoupon;
import solution.PercentCoupon;

class BehaviorTest {

    @Test
    void shouldSumThePricesAndApplyTheCouponOnce() {
        assertThat(Checkout.total(List.of(600L, 600L), new FlatCoupon("F", 500))).isEqualTo(700);
        assertThat(Checkout.total(List.of(500L, 500L), new PercentCoupon("P", 10))).isEqualTo(900);
    }

    @Test
    void shouldTreatEveryCouponTypeTheSameWay() {
        List<Coupon> coupons = List.of(
                new PercentCoupon("P", 50), new FlatCoupon("F", 100), new MinSpendCoupon("M", 5000, 100));

        long[] totals = coupons.stream().mapToLong(c -> Checkout.total(List.of(300L, 200L), c)).toArray();

        assertThat(totals).containsExactly(250, 400, 500);
    }

    @Test
    void shouldGiveNoDiscountWithoutACoupon() {
        assertThat(Checkout.total(List.of(300L, 200L), null)).isEqualTo(500);
    }

    @Test
    void shouldHandleEmptyAndNullBaskets() {
        assertThat(Checkout.total(List.of(), new FlatCoupon("F", 100))).isZero();
        assertThat(Checkout.total(null, new FlatCoupon("F", 100))).isZero();
    }

    @Test
    void shouldRejectNegativePrices() {
        assertThatThrownBy(() -> Checkout.total(Arrays.asList(100L, -1L), null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Checkout.total(List.of(-5L), new FlatCoupon("F", 1))).isInstanceOf(IllegalArgumentException.class);
    }
}
