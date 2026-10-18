package contract;

/** Provided value type. Amounts are in cents. */
public record InvoiceTotals(long subtotal, long tax, long total) {}
