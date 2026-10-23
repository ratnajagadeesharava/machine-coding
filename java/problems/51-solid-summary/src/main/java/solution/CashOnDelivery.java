package solution;

import contract.PaymentMethod;
import contract.PaymentResult;

/** Cash-on-delivery payment method. */
public class CashOnDelivery implements PaymentMethod {

    public CashOnDelivery(long maxCashCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public PaymentResult charge(long amountCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
