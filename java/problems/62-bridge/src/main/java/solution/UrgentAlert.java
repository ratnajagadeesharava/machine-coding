package solution;

import contract.Channel;

/** Delivers immediately with the title {@code "URGENT"} and the message in UPPER CASE. */
public class UrgentAlert extends Alert {

    public UrgentAlert(Channel channel) {
        super(channel);
    }

    @Override
    public void send(String recipient, String message) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
