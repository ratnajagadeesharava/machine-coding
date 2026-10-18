package solution;

import contract.PricingRules;

/**
 * discount(net): if net >= bulkThresholdCents then net * bulkDiscountBps / 10000 else 0.
 * tax(taxable): taxable * taxBps / 10000.
 * Both round HALF UP to whole cents (1005 * 1800 bps = 180.9 -> 181).
 */
public class StandardRules implements PricingRules {

    public StandardRules(int taxBps, long bulkThresholdCents, int bulkDiscountBps) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public long discount(long net) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public long tax(long taxable) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
