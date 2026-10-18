package solution;

import contract.Parcel;
import contract.ShippingStrategy;
import java.util.List;

public final class ShippingAdvisor {
    private ShippingAdvisor() {}

    /**
     * The strategy with the lowest cost for this parcel; on a tie the one that comes first in the list.
     * Null/empty list or an invalid parcel (same rules as {@link ShippingCalculator#quote}) -> IllegalArgumentException.
     */
    public static ShippingStrategy cheapest(List<ShippingStrategy> strategies, Parcel parcel) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
