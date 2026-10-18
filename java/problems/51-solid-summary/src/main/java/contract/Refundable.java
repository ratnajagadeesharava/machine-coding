package contract;

/** Provided contract: only some payment methods can give money back, so refunding is a separate, small interface. */
public interface Refundable {

    /** @return true if {@code amountCents} was refunded against that charge reference. */
    boolean refund(String reference, long amountCents);
}
