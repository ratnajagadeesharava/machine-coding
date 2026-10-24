package solution;

import contract.PaymentGateway;

/**
 * Checkout algorithm shared by all payment channels.
 */
public abstract class CheckoutFlow {

    protected abstract PaymentGateway createGateway();

    public final String checkout(String customerId, long amountCents) {
        if (customerId == null || customerId.isBlank()) {
            throw new IllegalArgumentException("illegal customer_");

        }
        if (amountCents <= 0) {

            throw new IllegalArgumentException("illegal amount");
        }
        PaymentGateway gateway = createGateway();
        String result = "";
        if (gateway.charge(customerId, amountCents)) {
            result = "RECEIPT " + gateway.name() + " " + customerId + " " + amountCents;
        } else {
            result = "DECLINED " + gateway.name() + " " + customerId + " " + amountCents;
        }
        return result;

    }
}
