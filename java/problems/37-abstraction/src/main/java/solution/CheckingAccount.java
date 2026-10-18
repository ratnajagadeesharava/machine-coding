package solution;

/**
 * May be overdrawn down to {@code -overdraftLimit}. Monthly fee 200, waived while the balance is at least 100000.
 */
public class CheckingAccount extends Account {

    public CheckingAccount(long overdraftLimit) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    protected long monthlyFee() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    protected boolean canWithdraw(long amount) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
