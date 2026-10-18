package solution;

import contract.Customer;
import contract.Wallet;

/** A customer that pays out of its own wallet. canAfford / pay are answered by the customer itself. */
public class StandardCustomer implements Customer {

    public StandardCustomer(String name, Wallet wallet) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String name() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public boolean canAfford(long amount) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void pay(long amount) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public Wallet getWallet() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
