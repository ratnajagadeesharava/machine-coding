package contract;

/** Provided value type. Amounts are in cents. */
public record InvoiceLine(String description, int quantity, long unitCents) {}
