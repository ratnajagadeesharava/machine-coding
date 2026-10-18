package solution;

import contract.PriceObserver;
import contract.Subscription;

/**
 * The subject. It knows observers only through {@link PriceObserver}; it never learns what they do.
 */
public class PriceFeed {

    /** Listens to every symbol. Null observer -> IllegalArgumentException. */
    public Subscription subscribe(PriceObserver observer) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Listens to one symbol (exact match). Null observer or blank symbol -> IllegalArgumentException. */
    public Subscription subscribe(String symbol, PriceObserver observer) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Number of active (not cancelled) subscriptions. */
    public int subscriberCount() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Tells the interested observers about a new price. Blank symbol or {@code priceCents <= 0} ->
     * IllegalArgumentException (nobody is notified). Rules:
     * <ul>
     *   <li>observers are notified in the order they subscribed;</li>
     *   <li>the set of observers is fixed when publish starts: an observer that subscribes during the
     *       notification does NOT get this price;</li>
     *   <li>an observer cancelled before its turn (by anyone, even during this publish) is skipped;</li>
     *   <li>an observer that throws a RuntimeException does not stop the others.</li>
     * </ul>
     *
     * @return how many observers threw
     */
    public int publish(String symbol, long priceCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
