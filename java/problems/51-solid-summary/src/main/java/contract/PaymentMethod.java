package contract;

/**
 * Provided contract (this is what Liskov substitution is checked against). EVERY payment method must:
 * <ul>
 *   <li>throw IllegalArgumentException (exactly that type) for an amount &lt;= 0;</li>
 *   <li>for any amount &gt; 0 NEVER throw: either approve, or decline;</li>
 *   <li>approve with a non-blank, unique reference; decline with the reference "".</li>
 * </ul>
 */
public interface PaymentMethod {

    PaymentResult charge(long amountCents);
}
