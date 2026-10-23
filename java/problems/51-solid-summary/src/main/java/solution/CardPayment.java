package solution;

import contract.PaymentMethod;
import contract.PaymentResult;
import contract.Refundable;

/** Card payment method. */
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
