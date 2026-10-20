package contract;

/** Provided: region-specific tax, price and address behaviour. */
public interface RegionKit {

    /** Region code: "US", "EU" or "IN". */
    String region();

    TaxCalculator taxCalculator();

    PriceFormatter priceFormatter();

    AddressValidator addressValidator();
}
