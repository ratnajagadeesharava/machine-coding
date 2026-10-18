package solution;

import contract.MessageSink;

/**
 * Tries the next sink up to {@code maxAttempts} times. A RuntimeException from the next sink triggers another
 * attempt; after the last failed attempt the LAST exception is rethrown. maxAttempts < 1 or a null next
 * -> IllegalArgumentException.
 */
public class RetryingSink implements MessageSink {

    public RetryingSink(MessageSink next, int maxAttempts) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void send(String message) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
