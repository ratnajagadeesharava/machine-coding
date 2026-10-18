package solution;

/** An order OWNS its lines: it creates them, hands out no way to add foreign ones, and they die with it. */
public class Order {

    public Order(String id) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String id() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Creates a line inside the order. qty <= 0, price <= 0 or blank sku -> IllegalArgumentException.
     * Adding an SKU that is already on the order increases that line's quantity (the unit price must match,
     * otherwise IllegalArgumentException). A cancelled order -> IllegalStateException.
     */
    public void addLine(String sku, int quantity, long unitPriceCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int lineCount() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** The order's own line object at that position (same instance every call). */
    public LineItem lineAt(int index) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Removes the line for that SKU; it is destroyed with its owner's reference. Returns whether it existed. */
    public boolean removeLine(String sku) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public long total() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Destroys all lines and blocks further addLine calls. */
    public void cancel() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public boolean isCancelled() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
