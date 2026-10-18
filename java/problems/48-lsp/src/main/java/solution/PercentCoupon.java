package solution;

import contract.Coupon;

/** percent in 1..100 (otherwise IllegalArgumentException). Discount = amount * percent / 100, rounded down. */
public class PercentCoupon implements Coupon {

    public PercentCoupon(String code, int percent) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String code() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public long apply(long amountCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
