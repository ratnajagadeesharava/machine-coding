package solution;

import contract.PaymentMethod;
import contract.PaymentResult;
import contract.Refundable;

/**
 * Approves any charge up to {@code limitCents} (inclusive) with references "card-1", "card-2", ... in the order
 * of approval; larger charges are declined. It can refund: a refund is accepted (true) when the reference was
 * issued by this card and the total refunded for that reference stays within the amount charged; otherwise false.
 */
public class CardPayment implements PaymentMethod, Refundable {

    public CardPayment(long limitCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public PaymentResult charge(long amountCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public boolean refund(String reference, long amountCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
