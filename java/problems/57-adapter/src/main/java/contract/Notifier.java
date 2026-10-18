package contract;

/** Provided target interface: what the application code wants to talk to. */
public interface Notifier {

    Delivery send(String phone, String message);
}
