package contract;

import java.util.List;
import java.util.OptionalLong;

/** Thin test entry point. Your domain model lives behind it. */
public interface OrderBookApi {

    /** side is "BUY" or "SELL". Returns the order id, or empty if the request is invalid. */
    OptionalLong submit(String side, long price, long quantity);

    /** True if the order was resting with quantity left and has been removed. */
    boolean cancel(long orderId);

    OptionalLong bestBid();

    OptionalLong bestAsk();

    /** Total remaining quantity resting at exactly this price on that side. */
    long volumeAt(String side, long price);

    /** Unfilled quantity; 0 if filled or cancelled; empty if the id was never issued. */
    OptionalLong remaining(long orderId);

    /** "buyId:sellId:price:quantity", oldest first. */
    List<String> trades();
}
