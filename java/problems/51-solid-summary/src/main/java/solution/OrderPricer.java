package solution;

import contract.DeliveryFeePolicy;
import contract.Item;
import java.util.List;

/** SRP: does the pricing arithmetic and nothing else. */
public class OrderPricer {

    /**
     * Sum of quantity * unitCents plus {@code policy.fee(distanceMeters)}.
     * Null/empty items, a quantity <= 0, a negative unit price, a negative distance or a null policy
     * -> IllegalArgumentException.
     */
    public long total(List<Item> items, int distanceMeters, DeliveryFeePolicy policy) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
