package solution;

import contract.Channel;

/**
 * The abstraction side of the bridge: WHAT is being said. It holds a {@link Channel} (the bridge) and never
 * knows which concrete channel it is. Subclasses decide how the message is shaped.
 */
public abstract class Alert {

    /** Null channel -> IllegalArgumentException. */
    protected Alert(Channel channel) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Switches the delivery channel at runtime. Null -> IllegalArgumentException. */
    public void setChannel(Channel channel) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Channel channel() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Sends (or, for digests, queues) a message for the recipient. */
    public abstract void send(String recipient, String message);
}
