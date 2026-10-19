package solution;

import contract.PriceObserver;
import contract.Subscription;

/** Publishes prices to subscribed observers. */
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

    /** Notifies the interested observers of a new price; returns how many observers threw. */
    public int publish(String symbol, long priceCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
