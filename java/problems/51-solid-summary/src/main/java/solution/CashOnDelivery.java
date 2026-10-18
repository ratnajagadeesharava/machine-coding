package solution;

import contract.PaymentMethod;
import contract.PaymentResult;

/**
 * Approves any charge up to {@code maxCashCents} (inclusive) with references "cod-1", "cod-2", ...; larger
 * charges are declined. Cash cannot be refunded through the system, so this type is NOT Refundable.
 */
public class CashOnDelivery implements PaymentMethod {

    public CashOnDelivery(long maxCashCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public PaymentResult charge(long amountCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
