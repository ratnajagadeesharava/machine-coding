package solution;

import contract.AddressValidator;
import contract.PriceFormatter;
import contract.RegionKit;
import contract.TaxCalculator;

/** Region "IN". Tax 18% (1800 bps, half up). Prices like "Rs 12.50". Postal codes: exactly 6 digits. */
public class IndiaKit implements RegionKit {

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
