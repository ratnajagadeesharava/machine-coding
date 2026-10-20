package solution;

import contract.Customer;

/** Takes payments. */
public class Cashier {

    /**
     * Returns "APPROVED" after calling customer.pay(amount), or "DECLINED" (without calling pay) when
     * customer.canAfford(amount) is false. A null customer or an amount <= 0 -> IllegalArgumentException.
     */
    public String checkout(Customer customer, long amount) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Sum of all approved amounts. */
    public long revenue() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
