package solution;

/** Cannot be withdrawn below {@code minBalance}. No monthly fee. */
public class SavingsAccount extends Account {

    public SavingsAccount(long minBalance) {
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
