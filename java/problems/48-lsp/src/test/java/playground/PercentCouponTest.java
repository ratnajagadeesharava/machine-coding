package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Coupon;
import org.junit.jupiter.api.Test;
import solution.PercentCoupon;

class PercentCouponTest extends CouponContractTest {

    @Override
    protected Coupon coupon() {
        return new PercentCoupon("SAVE25", 25);
    }

    @Test
    void shouldTakeThePercentageOffRoundingTheDiscountDown() {
        assertThat(new PercentCoupon("A", 25).apply(1001)).isEqualTo(751);
        assertThat(new PercentCoupon("B", 10).apply(1000)).isEqualTo(900);
        assertThat(new PercentCoupon("C", 100).apply(777)).isZero();
    }

    @Test
    void shouldRejectPercentagesOutsideOneToHundred() {
        assertThatThrownBy(() -> new PercentCoupon("X", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new PercentCoupon("X", 101)).isInstanceOf(IllegalArgumentException.class);
    }
}
