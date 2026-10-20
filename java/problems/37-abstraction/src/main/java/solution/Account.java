package solution;

/** Bank account. */
public abstract class Account {

    protected Account() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public long balance() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public void deposit(long amount) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public void withdraw(long amount) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public void monthEnd() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    protected abstract long monthlyFee();

    protected abstract boolean canWithdraw(long amount);
}
