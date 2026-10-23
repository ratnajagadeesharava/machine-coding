package contract;

/** Provided contract. */
public interface Customer {

    String name();

    boolean canAfford(long amount);

    /** Pays the amount. Throws IllegalStateException when the customer cannot afford it. */
    void pay(long amount);

    Wallet getWallet();
}
