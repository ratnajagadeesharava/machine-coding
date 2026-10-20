package solution;

import contract.Coupon;

/** Fixed-amount coupon. */
public class FlatCoupon implements Coupon {

    public FlatCoupon(String code, long offCents) {
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
