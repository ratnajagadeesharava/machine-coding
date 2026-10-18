package solution;

import contract.Order;

/** Prices shipping from the order alone. */
public class ShippingCalculator {

    /**
     * 0 when order.itemTotal() >= 10000. Otherwise by order.shippingCity(), case-insensitive:
     * "paris" 1500, "lyon" 1800, any other city 2500.
     */
    public long cost(Order order) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
