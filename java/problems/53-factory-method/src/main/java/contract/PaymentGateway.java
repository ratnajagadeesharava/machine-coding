package contract;

/** Provided contract: the "product" the factory method creates. */
public interface PaymentGateway {

    /** Short name, e.g. "card" or "bank". */
    String name();

    /** @return true if the charge was accepted. */
    boolean charge(String customerId, long amountCents);
}
