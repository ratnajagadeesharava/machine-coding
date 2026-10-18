package solution;

/**
 * May be overdrawn down to {@code -overdraftLimit}. Monthly fee 200, waived while the balance is at least 100000.
 */
public class CheckingAccount extends Account {
    private final long overDraftLimit;

    public CheckingAccount(long overdraftLimit) {
        this.overDraftLimit = overdraftLimit;
    }

    @Override
    protected long monthlyFee() {
        if (balance() >= 100000) {
            return 0;
        }
        return 200;
    }

    @Override
    protected boolean canWithdraw(long amount) {
        if (this.balance() - amount >= -overDraftLimit) {
            return true;
        } else {
            return false;
        }
    }
}
