package contract;

/** Provided contract: something that can refund a charge. */
public interface Refundable {

    /** @return true if {@code amountCents} was refunded against that charge reference. */
    boolean refund(String reference, long amountCents);
}
