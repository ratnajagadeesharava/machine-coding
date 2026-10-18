package solution;

import contract.Parcel;
import contract.ShippingStrategy;

/** name() is "weight". Cost = centsPerKg * (number of STARTED kilograms): 1 g -> 1 kg, 1000 g -> 1 kg, 1001 g -> 2 kg. */
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
