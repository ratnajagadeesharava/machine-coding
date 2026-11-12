package solution;

import contract.PaymentGateway;

/**
 * Card checkout.
 */
public class CardCheckout extends CheckoutFlow {

    @Override
    protected PaymentGateway createGateway() {
        return new CardGateway();
    }
}
