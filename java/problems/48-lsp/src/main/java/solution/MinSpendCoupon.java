package solution;

import contract.Coupon;

/**
 * Takes offCents off, but only when the amount is at least minSpendCents. Below the minimum the amount is returned
 * unchanged (it must NOT throw). The price never goes below 0.
 */
public class MinSpendCoupon implements Coupon {

    public MinSpendCoupon(String code, long minSpendCents, long offCents) {
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
