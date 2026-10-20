package solution;

import contract.PaymentGateway;

/** Card payment gateway. */
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
