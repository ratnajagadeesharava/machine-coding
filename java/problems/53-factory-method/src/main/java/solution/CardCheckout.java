package solution;

import contract.PaymentGateway;

/** Checks out by card: its factory method creates a {@link CardGateway}. */
public class CardCheckout extends CheckoutFlow {

    @Override
    protected PaymentGateway createGateway() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
