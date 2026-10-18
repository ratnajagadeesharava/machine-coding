package contract;

/** Provided observer interface. Implementations may be plain lambdas. May throw a RuntimeException. */
public interface PriceObserver {

    void onPrice(String symbol, long priceCents);
}
