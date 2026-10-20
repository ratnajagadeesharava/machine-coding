package solution;

import contract.Channel;

/** A message to be sent through a channel. */
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
