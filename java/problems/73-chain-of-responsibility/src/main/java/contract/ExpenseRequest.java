package contract;

/** Provided value type: an expense claim. {@code amountCents} is in cents. */
public record ExpenseRequest(String employee, long amountCents, String category) {}
