package contract;

/** Provided contract: the pricing decisions. Amounts are in cents. */
public interface PricingRules {

    /** Discount (>= 0) granted on a net amount. */
    long discount(long net);

    /** Tax (>= 0) due on an already-discounted amount. */
    long tax(long taxable);
}
