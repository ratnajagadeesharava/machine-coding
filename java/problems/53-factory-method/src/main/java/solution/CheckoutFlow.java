package solution;

import contract.PaymentGateway;

/** Checkout algorithm shared by all payment channels. */
public abstract class CheckoutFlow {

    protected abstract PaymentGateway createGateway();

    public final String checkout(String customerId, long amountCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
