package contract;

/** Provided abstraction: how a text message gets out. Could be an SMS gateway, a test fake, anything. */
public interface MessageSender {

    /** @return true if the message was accepted for delivery. */
    boolean send(String phone, String text);
}
