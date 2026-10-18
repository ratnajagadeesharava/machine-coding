package contract;

/** Provided value type. {@code reference} is non-blank when approved and "" when declined. */
public record PaymentResult(boolean approved, String reference) {}
