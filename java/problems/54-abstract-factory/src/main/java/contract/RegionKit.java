package contract;

/**
 * Provided abstract factory: one kit = one consistent FAMILY of products for a region. A client takes all of its
 * tax, price and address behaviour from the same kit, so it can never mix US tax with EU formatting.
 */
public interface RegionKit {

    /** Region code: "US", "EU" or "IN". */
    String region();

    TaxCalculator taxCalculator();

    PriceFormatter priceFormatter();

    AddressValidator addressValidator();
}
