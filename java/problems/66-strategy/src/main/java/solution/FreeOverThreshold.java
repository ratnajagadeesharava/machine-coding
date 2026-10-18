package solution;

import contract.Parcel;
import contract.ShippingStrategy;

/**
 * name() is {@code "free>=" + minValueCents + "/" + fallback.name()}. Parcels worth at least {@code minValueCents}
 * ship free; others cost whatever the fallback strategy says. Negative minimum or null fallback -> IllegalArgumentException.
 */
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
