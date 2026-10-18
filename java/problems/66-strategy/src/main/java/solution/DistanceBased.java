package solution;

import contract.Parcel;
import contract.ShippingStrategy;

/** name() is "distance". Cost = centsPerKm * distanceKm. */
public class DistanceBased implements ShippingStrategy {

    public DistanceBased(long centsPerKm) {
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
