package solution;

import contract.FeePolicy;
import java.util.List;

/**
 * Prices a stay by asking the registered policies. It must contain NO knowledge of any vehicle type: adding a new
 * kind of vehicle means adding a new FeePolicy, never editing this class.
 */
public class FeeCalculator {

    public FeeCalculator(List<FeePolicy> policies) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Adds a policy. When several policies support a type, the most recently registered one wins. */
    public void register(FeePolicy policy) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Fee from the winning policy. Null type, negative minutes, or no policy supporting the type
     * -> IllegalArgumentException.
     */
    public long fee(String vehicleType, int minutes) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
