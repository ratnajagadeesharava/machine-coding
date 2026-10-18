package solution;

import contract.Channel;

/** Delivers immediately with the title {@code "Info"} and the message unchanged. */
public class InfoAlert extends Alert {

    public InfoAlert(Channel channel) {
        super(channel);
    }

    @Override
    public void send(String recipient, String message) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
