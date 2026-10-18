package solution;

/**
 * The abstract base: it owns the balance and the rules every account shares, and leaves two decisions
 * to subclasses. It must stay abstract and must declare exactly two abstract methods.
 */
public abstract class Account {

    protected Account() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public long balance() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** amount must be > 0 (IllegalArgumentException). */
    public void deposit(long amount) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * amount must be > 0 (IllegalArgumentException). If {@link #canWithdraw(long)} says no, throw
     * IllegalStateException and leave the balance unchanged.
     */
    public void withdraw(long amount) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Subtracts {@link #monthlyFee()} from the balance (the fee is charged even if it overdraws). */
    public void monthEnd() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Fee charged by {@link #monthEnd()}. Decided by the account type. */
    protected abstract long monthlyFee();

    /** Whether withdrawing {@code amount} right now is allowed. Decided by the account type. */
    protected abstract boolean canWithdraw(long amount);
}
