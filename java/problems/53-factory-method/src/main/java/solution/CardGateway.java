package solution;

import contract.PaymentGateway;

/**
 * Card payment gateway.
 */
public class CardGateway implements PaymentGateway {

    @Override
    public String name() {
        return "card";
    }

    @Override
    public boolean charge(String customerId, long amountCents) {
        return amountCents <= 100000;
    }
}
