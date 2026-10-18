package solution;

import contract.Channel;
import java.util.function.Consumer;

/**
 * Writes {@code "SMS <recipient>: <text>"} where text = {@code title + ": " + body}, cut to 40 characters:
 * a longer text becomes its first 37 characters followed by "...". Null wire -> IllegalArgumentException.
 */
public class SmsChannel implements Channel {

    public SmsChannel(Consumer<String> wire) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void deliver(String recipient, String title, String body) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
