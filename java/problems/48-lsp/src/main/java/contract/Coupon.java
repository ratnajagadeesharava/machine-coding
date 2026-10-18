package contract;

/**
 * Provided contract. EVERY coupon must obey ALL of this, so that code written against {@code Coupon}
 * works with any of them:
 * <ul>
 *   <li>{@code code()} is a non-blank string.</li>
 *   <li>{@code apply(amount)} returns the price after the discount, for any {@code amount >= 0}:
 *       {@code 0 <= result <= amount}. It never throws for such an amount (even if the coupon "does not apply").</li>
 *   <li>{@code apply(0) == 0}.</li>
 *   <li>A negative amount throws {@link IllegalArgumentException} (always that type, from every coupon).</li>
 *   <li>It is deterministic: the same amount always gives the same result.</li>
 * </ul>
 */
public interface Coupon {

    String code();

    long apply(long amountCents);
}
