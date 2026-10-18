package contract;

/**
 * Provided contract. {@link #getCustomer()} exists, but a caller that needs the shipping city should ask the
 * ORDER ({@link #shippingCity()}) instead of walking order -> customer -> address -> city.
 */
public interface Order {

    Customer getCustomer();

    String shippingCity();

    /** Value of the goods in cents. */
    long itemTotal();
}
