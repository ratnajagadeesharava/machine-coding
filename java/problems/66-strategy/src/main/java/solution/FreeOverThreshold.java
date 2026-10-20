package solution;

import contract.Parcel;
import contract.ShippingStrategy;

/** Free shipping above a parcel value, otherwise delegates to another strategy. */
public class FreeOverThreshold implements ShippingStrategy {

    public FreeOverThreshold(long minValueCents, ShippingStrategy fallback) {
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
