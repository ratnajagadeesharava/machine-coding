package contract;

/** Provided value type: the outcome of sending one message. {@code error} is "" when ok. */
public record Delivery(boolean ok, String error) {}
