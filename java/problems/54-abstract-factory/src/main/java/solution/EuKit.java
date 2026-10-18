package solution;

import contract.AddressValidator;
import contract.PriceFormatter;
import contract.RegionKit;
import contract.TaxCalculator;

/** Region "EU". Tax 19% (1900 bps, half up). Prices like "12,50 EUR" (decimal comma). Postal codes: exactly 5 digits. */
public class EuKit implements RegionKit {

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
