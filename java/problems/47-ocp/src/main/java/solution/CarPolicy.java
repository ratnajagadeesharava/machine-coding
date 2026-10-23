package solution;

import contract.FeePolicy;

/** Parking fee policy for cars. */
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
