package solution;

import contract.Channel;
import java.util.function.Consumer;

/**
 * Writes one line to the "wire" per delivery: {@code "EMAIL to:<recipient> subject:<title> | <body>"}.
 * Null wire -> IllegalArgumentException.
 */
public class EmailChannel implements Channel {

    public EmailChannel(Consumer<String> wire) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void deliver(String recipient, String title, String body) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
