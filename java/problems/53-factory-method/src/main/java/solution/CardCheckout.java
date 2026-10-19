package solution;

import contract.PaymentGateway;

/** Card checkout. */
public class CardCheckout extends CheckoutFlow {

    @Override
    protected PaymentGateway createGateway() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
