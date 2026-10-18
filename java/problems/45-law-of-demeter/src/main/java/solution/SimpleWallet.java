package solution;

import contract.Wallet;

/** initial must be >= 0 (IllegalArgumentException otherwise). */
public class SimpleWallet implements Wallet {

    public SimpleWallet(long initial) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public long balance() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void debit(long amount) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
