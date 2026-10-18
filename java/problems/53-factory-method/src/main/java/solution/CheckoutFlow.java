package solution;

import contract.PaymentGateway;

/**
 * The creator. {@link #checkout} is the fixed algorithm; WHICH gateway it uses is decided by subclasses through
 * the factory method {@link #createGateway()}. The base class never says {@code new CardGateway()}.
 */
public abstract class CheckoutFlow {

    /** The factory method: subclasses decide which concrete gateway to create. */
    protected abstract PaymentGateway createGateway();

    /**
     * Blank customer or amount <= 0 -> IllegalArgumentException (createGateway is NOT called).
     * Otherwise calls createGateway() exactly once, charges, and returns
     * {@code "RECEIPT <gateway name> <customerId> <amount>"} if accepted or
     * {@code "DECLINED <gateway name> <customerId> <amount>"} if not.
     * This method must be {@code final}: subclasses customise the gateway, not the algorithm.
     */
    public final String checkout(String customerId, long amountCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
