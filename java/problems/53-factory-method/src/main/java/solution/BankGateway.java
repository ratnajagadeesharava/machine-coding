package solution;

import contract.PaymentGateway;

/** Bank payment gateway. */
public class BankGateway implements PaymentGateway {

    @Override
    public String name() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public boolean charge(String customerId, long amountCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
