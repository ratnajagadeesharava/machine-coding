package solution;

import contract.FeePolicy;

/**
 * Supports "car" (any letter case). 0 minutes -> 0. Otherwise 200 cents for the first 60 minutes plus 100 cents
 * for every started 30 minutes after that, never more than 1500.
 * Examples: 30 -> 200, 60 -> 200, 61 -> 300, 90 -> 300, 91 -> 400, 1000 -> 1500.
 */
public class CarPolicy implements FeePolicy {

    @Override
    public boolean supports(String vehicleType) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public long fee(int minutes) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
