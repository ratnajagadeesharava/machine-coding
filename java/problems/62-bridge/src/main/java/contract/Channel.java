package contract;

/** Provided implementor interface: HOW a message is delivered. */
public interface Channel {

    void deliver(String recipient, String title, String body);
}
