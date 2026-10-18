package solution;

import contract.FeePolicy;

/** Supports "motorcycle" (any letter case). Charges half of what a car would pay (integer division). */
public class MotorcyclePolicy implements FeePolicy {

    @Override
    public boolean supports(String vehicleType) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public long fee(int minutes) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
