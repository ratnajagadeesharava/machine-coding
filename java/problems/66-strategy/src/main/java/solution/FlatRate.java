package solution;

import contract.Parcel;
import contract.ShippingStrategy;

/** name() is "flat". Every parcel costs {@code cents}. A negative amount -> IllegalArgumentException. */
public class FlatRate implements ShippingStrategy {

    public FlatRate(long cents) {
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
