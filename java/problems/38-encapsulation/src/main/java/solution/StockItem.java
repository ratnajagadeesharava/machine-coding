package solution;

import java.sql.Array;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

/**
 * A warehouse stock record. The invariants (0 <= quantity <= capacity, unique tags) must hold no matter
 * what a caller does with the public API. All fields must be private; there are no setters.
 */
public class StockItem {
    private final int capacity;
    private final String sku;
    private int quantity;
    private List<String> tags;
    private List<String> history;

    /**
     * sku must not be blank and capacity must be > 0, otherwise IllegalArgumentException. Starts empty.
     */
    public StockItem(String sku, int capacity) {
        if (sku == null || sku.isBlank()) {
            throw new IllegalArgumentException("SKU must not be empty");
        }
        if (capacity <= 0)
            throw new IllegalArgumentException("capacity must not be empty");
        this.sku = sku;
        this.capacity = capacity;
        this.history = new ArrayList<>();
        this.tags = new ArrayList<>();
    }

    public String sku() {
        return this.sku;
    }

    public int capacity() {
        return this.capacity;
    }

    public int quantity() {
        return this.quantity;
    }

    /**
     * n must be > 0 (IllegalArgumentException). Exceeding capacity -> IllegalStateException, nothing changes.
     */
    public void add(int n) {
        if (n <= 0) {
            throw new IllegalArgumentException("n must be greater than 0");
        }
        if (this.quantity + n > capacity) {
            throw new IllegalStateException("quantity should not increase the capacity");
        }
        this.quantity += n;
        this.history.add("+" + n);

    }

    /**
     * n must be > 0 (IllegalArgumentException). More than is in stock -> IllegalStateException, nothing changes.
     */
    public void remove(int n) {
        if (n <= 0) {
            throw new IllegalArgumentException("n must be greater than 0");
        }
        if (this.quantity - n < 0) {
            throw new IllegalStateException("quantity should not increase the capacity");
        }
        this.quantity -= n;
        this.history.add("-" + n);
    }

    /**
     * Moves n units to another item atomically: if either side would break its invariant, neither changes.
     */
    public void transferTo(StockItem other, int n) {
        if (other == null || other == this) {
            throw new IllegalArgumentException("invalid transfer target");
        }
        if (n <= 0) {
            throw new IllegalArgumentException("n must be greater than 0");
        }
        if (this.quantity - n < 0) {
            throw new IllegalStateException("not enough stock");
        }
        if (other.quantity + n > other.capacity) {
            throw new IllegalStateException("target over capacity");
        }
        // all checks passed, now change both
        this.quantity -= n;
        other.quantity += n;
        this.history.add("-" + n + " (transfer)");
        other.history.add("+" + n + " (transfer)");
    }

    /**
     * Blank tags are ignored; a tag already present is ignored; tags are stored trimmed and lower-case.
     */
    public void addTag(String tag) {
        String tag_trimmed = tag.trim().toLowerCase(Locale.ROOT);
        if (tag.isBlank()) return;
        if (tags.contains(tag_trimmed)) {
            return;
        }
        tags.add(tag_trimmed);
    }

    /**
     * Tags in insertion order. Changing the returned list must NOT change the item.
     */
    public List<String> tags() {
        return List.copyOf(tags);
    }

    /**
     * One entry per successful change, e.g. "+5", "-2", "-3 (transfer)", "+3 (transfer)". Read-only.
     */
    public List<String> history() {
        return List.copyOf(history);
    }
}
