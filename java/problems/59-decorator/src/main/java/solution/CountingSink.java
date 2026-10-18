package solution;

import contract.MessageSink;

/** Passes messages on and counts those that were delivered (the next sink did not throw). Null next -> IllegalArgumentException. */
public class CountingSink implements MessageSink {

    public CountingSink(MessageSink next) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void send(String message) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Number of successfully delivered messages. */
    public int count() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
