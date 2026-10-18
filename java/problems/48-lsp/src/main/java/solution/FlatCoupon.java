package solution;

import contract.Coupon;

/** Takes a fixed number of cents off (offCents >= 0, otherwise IllegalArgumentException). The price never goes below 0. */
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
