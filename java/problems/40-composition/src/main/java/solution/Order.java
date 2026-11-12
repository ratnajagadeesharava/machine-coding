package solution;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * An order OWNS its lines: it creates them, hands out no way to add foreign ones, and they die with it.
 */
public class Order {
    private final String id;
    private List<LineItem> lines;
    private Map<String, LineItem> map;
    private long total;
    private boolean blockAdd;

    public Order(String id) {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("id cannot be empty");
        }
        this.id = id;
        this.lines = new ArrayList<>();
        this.map = new HashMap<>();
        this.total = 0;
        this.blockAdd = false;
    }

    public String id() {
        return this.id;
    }

    /**
     * Creates a line inside the order. qty <= 0, price <= 0 or blank sku -> IllegalArgumentException.
     * Adding an SKU that is already on the order increases that line's quantity (the unit price must match,
     * otherwise IllegalArgumentException). A cancelled order -> IllegalStateException.
     */
    public void addLine(String sku, int quantity, long unitPriceCents) {
        if (this.blockAdd) {

            throw  new IllegalStateException("state");
        }
        if (quantity <= 0 || unitPriceCents <= 0 || sku == null || sku.isBlank()) {
            throw new IllegalArgumentException("illegal arguments");
        }
        if (this.map.containsKey(sku)) {
            LineItem line = this.map.get(sku);
            if (line.unitPrice() == unitPriceCents) {


                line.increaseQuantity(quantity);
            } else {
                throw new IllegalArgumentException("already present with different unit price");

            }
        } else {
            LineItem lineItem = new LineItem(sku, quantity, unitPriceCents);
            this.map.put(sku, lineItem);
            this.lines.add(lineItem);
        }
        this.total += quantity * unitPriceCents;


    }

    public int lineCount() {
        return this.lines.size();
    }

    /**
     * The order's own line object at that position (same instance every call).
     */
    public LineItem lineAt(int index) {
        if (index >= this.lines.size()) {
            throw new IllegalStateException("Invalid index");
        }
        return this.lines.get(index);
    }

    /**
     * Removes the line for that SKU; it is destroyed with its owner's reference. Returns whether it existed.
     */
    public boolean removeLine(String sku) {
        if (this.map.containsKey(sku)) {
            LineItem lineItem = this.map.get(sku);
            this.lines.remove(lineItem);
            this.total -= lineItem.subtotal();
            this.map.remove(sku);
            return true;
        } else {
            return false;
        }
    }

    public long total() {
        return this.total;
    }

    /**
     * Destroys all lines and blocks further addLine calls.
     */
    public void cancel() {
        this.map.clear();
        this.lines.clear();
        this.total = 0;
        this.blockAdd = true;
    }

    public boolean isCancelled() {
        return this.blockAdd;
    }
}
