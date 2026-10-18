package solution;

import contract.Coupon;
import java.util.List;

/** Works with ANY Coupon and trusts the contract: no instanceof, no special cases per coupon type. */
public final class Checkout {
    private Checkout() {}

    /**
     * Sum of the prices, then the coupon is applied ONCE to that sum. A null coupon means no discount.
     * A null/empty list gives 0. A negative price -> IllegalArgumentException.
     */
    public static long total(List<Long> pricesCents, Coupon coupon) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
