package contract;

/** Provided: how a text message gets out. */
public interface MessageSender {

    /** @return true if the message was accepted for delivery. */
    boolean send(String phone, String text);
}
