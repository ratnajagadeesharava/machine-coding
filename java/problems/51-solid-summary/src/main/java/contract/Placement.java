package contract;

/** Provided value type: the outcome of placing an order. */
public record Placement(String status, long totalCents, String reference) {}
