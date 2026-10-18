package solution;

import contract.MessageSink;

/**
 * Replaces every run of 4 or more digits in the message with {@code ****} (card numbers, PINs) and passes it on.
 * Runs of 1 to 3 digits are left alone. Null next -> IllegalArgumentException.
 */
public class RedactingSink implements MessageSink {

    public RedactingSink(MessageSink next) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void send(String message) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
