package solution;

/**
 * Cannot be withdrawn below {@code minBalance}. No monthly fee.
 */
public class SavingsAccount extends Account {
    private final long minBalance;

    public SavingsAccount(long minBalance) {
        this.minBalance = minBalance;
    }

    @Override
    protected long monthlyFee() {
        return 0;
    }

    @Override
    protected boolean canWithdraw(long amount) {
        if (this.balance() - amount >= minBalance) {
            return true;
        }
        return false;
    }
}
