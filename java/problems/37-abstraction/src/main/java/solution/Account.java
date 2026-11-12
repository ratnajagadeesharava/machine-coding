package solution;

/**
 * The abstract base: it owns the balance and the rules every account shares, and leaves two decisions
 * to subclasses. It must stay abstract and must declare exactly two abstract methods.
 */
public abstract class Account {
    private long balance;

    protected Account() {
        this.balance = 0;
    }

    public long balance() {
        return this.balance;
    }

    /**
     * amount must be > 0 (IllegalArgumentException).
     */
    public void deposit(long amount) {
        if (amount <= 0) {
            throw new IllegalArgumentException("amount must be postive");
        }
        this.balance += amount;
    }

    /**
     * amount must be > 0 (IllegalArgumentException).
     * If {@link #canWithdraw(long)} says no, throw
     * IllegalStateException and leave the balance unchanged.
     */
    public void withdraw(long amount) {
        if (amount <= 0) {
            throw new IllegalArgumentException("amount must be postive");
        }

        if (!canWithdraw(amount)) {
            throw new IllegalStateException("Amount is illegal");
        }

        balance -= amount;
    }

    /**
     * Subtracts {@link #monthlyFee()} from the balance (the fee is charged even if it overdraws).
     */
    public void monthEnd() {
        this.balance -= this.monthlyFee();
    }

    /**
     * Fee charged by {@link #monthEnd()}. Decided by the account type.
     */
    protected abstract long monthlyFee();

    /**
     * Whether withdrawing {@code amount} right now is allowed. Decided by the account type.
     */
    protected abstract boolean canWithdraw(long amount);
}
