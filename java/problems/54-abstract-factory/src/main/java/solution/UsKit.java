package solution;

import contract.AddressValidator;
import contract.PriceFormatter;
import contract.RegionKit;
import contract.TaxCalculator;

/**
 * Region "US". Tax 8% (800 basis points, half up). Prices like "$12.50". Postal codes: 5 digits, optionally
 * followed by "-" and 4 more digits ("12345", "12345-6789").
 */
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
