package contract;

/** Provided contract. */
public interface Order {

    Customer getCustomer();

    String shippingCity();

    /** Value of the goods in cents. */
    long itemTotal();
}
