package solution;

import contract.PaymentGateway;

/**
 * Bank payment gateway.
 */
public class BankGateway implements PaymentGateway {

    @Override
    public String name() {
        return "bank";
    }

    @Override
    public boolean charge(String customerId, long amountCents) {
        return amountCents >= 1000;
    }
}
