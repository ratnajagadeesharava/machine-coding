package solution;

import contract.AddressValidator;
import contract.PriceFormatter;
import contract.RegionKit;
import contract.TaxCalculator;

/** Region kit for the US. */
public class UsKit implements RegionKit {

    @Override
    public String region() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public TaxCalculator taxCalculator() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public PriceFormatter priceFormatter() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public AddressValidator addressValidator() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
