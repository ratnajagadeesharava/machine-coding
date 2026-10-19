package contract;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface Bank {

    /** Creates an account with an initial balance. IllegalArgumentException if it exists or balance < 0. */
    void open(String account, long balance);

    /** Current balance. IllegalArgumentException for an unknown account. */
    long balance(String account);

    /**
     * Moves money atomically: either both balances change or neither does. Returns false (nothing changes) when
     * {@code from} has less than {@code amount}; a transfer of exactly the balance succeeds.
     * IllegalArgumentException for an unknown account, from == to, or amount <= 0.
     */
    boolean transfer(String from, String to, long amount);

    /** Sum of all balances. Must never include a half-applied transfer (no money "in flight"). */
    long total();
}
