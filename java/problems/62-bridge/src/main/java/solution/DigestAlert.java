package solution;

import contract.Channel;

/** Queues messages per recipient and delivers them together as one message on flush. */
public class DigestAlert extends Alert {

    public DigestAlert(Channel channel) {
        super(channel);
    }

    /** Queues the message for the recipient (nothing is delivered yet). */
    @Override
    public void send(String recipient, String message) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Number of messages waiting for that recipient. */
    public int pending(String recipient) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Delivers one message to the recipient: title {@code "Digest (<n>)"}, body = the queued messages joined with
     * {@code "; "} in order; then empties that recipient's queue and returns true. Nothing queued -> false and
     * nothing is delivered.
     */
    public boolean flush(String recipient) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
