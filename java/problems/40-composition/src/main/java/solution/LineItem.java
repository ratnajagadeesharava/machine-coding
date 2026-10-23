package solution;

/** One line of an order. */
public class LineItem {
    private int quantity;
    private final long unitPriceCents;

    private final String sku;

    LineItem(String sku, int quantity, long unitPriceCents) {
        if (quantity <= 0 || unitPriceCents <= 0 || sku == null || sku.isBlank()) {
            throw new IllegalArgumentException("illegal arguments");
        }
        this.quantity = quantity;
        this.unitPriceCents = unitPriceCents;
        this.sku = sku;
    }

    public String sku() {
        return this.sku;
    }

    public int quantity() {
        return this.quantity;
    }

    public void increaseQuantity(int quantity) {
        this.quantity += quantity;
    }

    public long unitPrice() {
        return this.unitPriceCents;
    }

    public long subtotal() {
        return quantity * unitPriceCents;
    }
}
