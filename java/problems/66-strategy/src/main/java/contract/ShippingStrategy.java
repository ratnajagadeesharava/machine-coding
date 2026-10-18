package contract;

/** Provided strategy interface: one way of pricing delivery. Amounts are in cents. */
public interface ShippingStrategy {

    /** Short label, e.g. "flat". */
    String name();

    long cost(Parcel parcel);
}
