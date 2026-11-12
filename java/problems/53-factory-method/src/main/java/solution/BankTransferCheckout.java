package solution;

import contract.PaymentGateway;

/**
 * Bank transfer checkout.
 */
public class BankTransferCheckout extends CheckoutFlow {

    @Override
    protected PaymentGateway createGateway() {
        return new BankGateway();
    }
}
