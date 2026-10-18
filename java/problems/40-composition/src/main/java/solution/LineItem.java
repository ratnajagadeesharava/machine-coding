package solution;

/**
 * One line of an order. It is a PART of {@link Order}: only the order can create one (keep the constructor
 * non-public) and it cannot live without its order.
 */
public class LineItem {

    LineItem(String sku, int quantity, long unitPriceCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String sku() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int quantity() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public long unitPrice() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public long subtotal() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
