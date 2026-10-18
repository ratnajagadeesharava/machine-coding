package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Coupon;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Checkout;
import solution.FlatCoupon;
import solution.PercentCoupon;

/**
 * A new subtype invented in the test: it applies one coupon and then another. The same shared suite verifies it
 * (it only passes if the coupons it is built from honour the contract), and Checkout must accept it unchanged.
 */
class StackedCouponTest extends CouponContractTest {

    private static final class Stacked implements Coupon {
        private final Coupon first;
        private final Coupon second;

        Stacked(Coupon first, Coupon second) {
            this.first = first;
            this.second = second;
        }

        public String code() { return "STACK"; }
        public long apply(long amount) { return second.apply(first.apply(amount)); }
    }

    private static Coupon stacked() {
        return new Stacked(new PercentCoupon("P", 10), new FlatCoupon("F", 100));
    }

    @Override
    protected Coupon coupon() {
        return stacked();
    }

    @Test
    void shouldLetCheckoutUseACouponItHasNeverSeen() {
        assertThat(Checkout.total(List.of(600L, 400L), stacked())).isEqualTo(800);
    }
}
