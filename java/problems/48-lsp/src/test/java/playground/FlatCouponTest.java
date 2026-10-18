package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Coupon;
import org.junit.jupiter.api.Test;
import solution.FlatCoupon;

class FlatCouponTest extends CouponContractTest {

    @Override
    protected Coupon coupon() {
        return new FlatCoupon("FIVE", 500);
    }

    @Test
    void shouldTakeAFixedAmountOff() {
        assertThat(new FlatCoupon("A", 500).apply(1000)).isEqualTo(500);
    }

    @Test
    void shouldClampAtZeroWhenTheDiscountIsBiggerThanThePrice() {
        assertThat(new FlatCoupon("A", 500).apply(120)).isZero();
        assertThat(new FlatCoupon("HUGE", Long.MAX_VALUE).apply(5)).isZero();
    }

    @Test
    void shouldRejectANegativeDiscount() {
        assertThatThrownBy(() -> new FlatCoupon("X", -1)).isInstanceOf(IllegalArgumentException.class);
    }
}
