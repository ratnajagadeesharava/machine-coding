package solution;

import contract.Parcel;
import contract.ShippingStrategy;

/** Weight-based shipping. */
public class ByWeight implements ShippingStrategy {

    public ByWeight(long centsPerKg) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String name() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public long cost(Parcel parcel) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
