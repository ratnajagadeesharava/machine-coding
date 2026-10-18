package contract;

/**
 * Provided contract. Note {@link #getWallet()}: it exists, but code that only wants a customer to pay should
 * ask the CUSTOMER ({@link #canAfford}, {@link #pay}) instead of reaching through to the wallet.
 */
public interface Customer {

    String name();

    boolean canAfford(long amount);

    /** Pays the amount. Throws IllegalStateException when the customer cannot afford it. */
    void pay(long amount);

    Wallet getWallet();
}
