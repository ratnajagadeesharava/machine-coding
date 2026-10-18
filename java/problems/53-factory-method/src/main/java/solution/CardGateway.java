package solution;

import contract.PaymentGateway;

/** name() is "card". Accepts charges of up to 100000 cents (inclusive). */
public class CardGateway implements PaymentGateway {

    @Override
    public String name() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public boolean charge(String customerId, long amountCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
