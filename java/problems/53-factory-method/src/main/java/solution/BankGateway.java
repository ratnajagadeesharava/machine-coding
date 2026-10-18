package solution;

import contract.PaymentGateway;

/** name() is "bank". Bank transfers have a minimum: accepts charges of at least 1000 cents. */
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
