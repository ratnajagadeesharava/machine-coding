package contract;

/** Provided observer interface. */
public interface PriceObserver {

    void onPrice(String symbol, long priceCents);
}
