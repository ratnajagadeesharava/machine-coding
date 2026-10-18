package solution;

import java.util.List;

/**
 * A warehouse stock record. The invariants (0 <= quantity <= capacity, unique tags) must hold no matter
 * what a caller does with the public API. All fields must be private; there are no setters.
 */
public class StockItem {

    /** sku must not be blank and capacity must be > 0, otherwise IllegalArgumentException. Starts empty. */
    public StockItem(String sku, int capacity) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String sku() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int capacity() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int quantity() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** n must be > 0 (IllegalArgumentException). Exceeding capacity -> IllegalStateException, nothing changes. */
    public void add(int n) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** n must be > 0 (IllegalArgumentException). More than is in stock -> IllegalStateException, nothing changes. */
    public void remove(int n) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Moves n units to another item atomically: if either side would break its invariant, neither changes. */
    public void transferTo(StockItem other, int n) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Blank tags are ignored; a tag already present is ignored; tags are stored trimmed and lower-case. */
    public void addTag(String tag) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Tags in insertion order. Changing the returned list must NOT change the item. */
    public List<String> tags() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** One entry per successful change, e.g. "+5", "-2", "-3 (transfer)", "+3 (transfer)". Read-only. */
    public List<String> history() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
