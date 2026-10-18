package solution;

import contract.Parcel;
import contract.ShippingStrategy;

/** The context. The pricing algorithm is a replaceable strategy it is given, never an if/else of its own. */
public class ShippingCalculator {

    /** Null strategy -> IllegalArgumentException. */
    public ShippingCalculator(ShippingStrategy strategy) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Swaps the algorithm at runtime. Null -> IllegalArgumentException. */
    public void setStrategy(ShippingStrategy strategy) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public ShippingStrategy strategy() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** The current strategy's cost. A null parcel, weight < 1 g, negative distance or negative value -> IllegalArgumentException. */
    public long quote(Parcel parcel) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
