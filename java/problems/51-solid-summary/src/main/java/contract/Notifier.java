package contract;

/** Provided abstraction: how the customer is told. */
public interface Notifier {

    void notify(String orderId, String message);
}
