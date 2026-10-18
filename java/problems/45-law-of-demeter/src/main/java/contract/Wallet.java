package contract;

/** Provided contract. */
public interface Wallet {

    long balance();

    /** Throws IllegalArgumentException for amount <= 0 and IllegalStateException when funds are insufficient. */
    void debit(long amount);
}
