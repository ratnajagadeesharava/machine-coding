package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Coupon;
import org.junit.jupiter.api.Test;
import solution.MinSpendCoupon;

class MinSpendCouponTest extends CouponContractTest {

    @Override
    protected Coupon coupon() {
        return new MinSpendCoupon("BIG", 1000, 300);
    }

    @Test
    void shouldApplyOnlyFromTheMinimumSpend() {
        Coupon c = new MinSpendCoupon("BIG", 1000, 300);

        assertThat(c.apply(999)).isEqualTo(999);
        assertThat(c.apply(1000)).isEqualTo(700);
        assertThat(c.apply(5000)).isEqualTo(4700);
    }

    @Test
    void shouldNotThrowBelowTheMinimumSpend() {
        Coupon c = new MinSpendCoupon("BIG", 1000, 300);

        assertThat(c.apply(0)).isZero();
        assertThat(c.apply(1)).isEqualTo(1);
    }

    @Test
    void shouldClampAtZero() {
        assertThat(new MinSpendCoupon("X", 10, 500).apply(10)).isZero();
    }
}
