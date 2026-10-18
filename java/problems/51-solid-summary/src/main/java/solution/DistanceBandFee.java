package solution;

import contract.DeliveryFeePolicy;

/**
 * 0 up to and including 1000 m; 300 up to and including 3000 m; beyond that 300 plus 100 for every started
 * kilometre past 3000 m (3001 -> 400, 4000 -> 400, 4001 -> 500). A negative distance -> IllegalArgumentException.
 */
public class DistanceBandFee implements DeliveryFeePolicy {

    @Override
    public long fee(int distanceMeters) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
