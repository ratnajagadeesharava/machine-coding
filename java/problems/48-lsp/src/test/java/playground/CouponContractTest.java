package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Coupon;
import org.junit.jupiter.api.Test;

/**
 * The shared test suite. Each concrete subclass supplies ONE coupon, and every test below must pass for it.
 * A subtype that cannot pass this suite is not substitutable for {@link Coupon}.
 */
abstract class CouponContractTest {

    private static final long[] AMOUNTS = {0, 1, 99, 100, 999, 1000, 1001, 10_000, 123_456, 1_000_000_000L};

    protected abstract Coupon coupon();

    @Test
    void shouldHaveANonBlankCode() {
        assertThat(coupon().code()).isNotBlank();
    }

    @Test
    void shouldNeverIncreaseThePrice() {
        Coupon c = coupon();
        for (long amount : AMOUNTS) {
            assertThat(c.apply(amount)).as("apply(%d)", amount).isLessThanOrEqualTo(amount);
        }
    }

    @Test
    void shouldNeverReturnANegativePrice() {
        Coupon c = coupon();
        for (long amount : AMOUNTS) {
            assertThat(c.apply(amount)).as("apply(%d)", amount).isGreaterThanOrEqualTo(0);
        }
    }

    @Test
    void shouldLeaveZeroAtZeroAndNeverThrowForValidAmounts() {
        Coupon c = coupon();

        assertThat(c.apply(0)).isZero();
        for (long amount : AMOUNTS) {
            c.apply(amount);
        }
    }

    @Test
    void shouldRejectNegativeAmountsWithIllegalArgumentException() {
        Coupon c = coupon();

        assertThatThrownBy(() -> c.apply(-1)).isExactlyInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.apply(Long.MIN_VALUE)).isExactlyInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldBeDeterministic() {
        Coupon c = coupon();
        for (long amount : AMOUNTS) {
            assertThat(c.apply(amount)).as("apply(%d)", amount).isEqualTo(c.apply(amount));
        }
    }
}
