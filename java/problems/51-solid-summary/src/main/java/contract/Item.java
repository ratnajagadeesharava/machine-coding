package contract;

/** Provided value type: one line of a meal order. Amounts are in cents. */
public record Item(String name, int quantity, long unitCents) {}
