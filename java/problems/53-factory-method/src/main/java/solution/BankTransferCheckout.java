package solution;

import contract.PaymentGateway;

/** Checks out by bank transfer: its factory method creates a {@link BankGateway}. */
public class BankTransferCheckout extends CheckoutFlow {

    @Override
    protected PaymentGateway createGateway() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
