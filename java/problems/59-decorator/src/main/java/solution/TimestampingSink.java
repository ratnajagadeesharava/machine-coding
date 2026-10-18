package solution;

import contract.MessageSink;
import java.time.Clock;

/** Prefixes every message with {@code "[t=<clock.millis()>] "} and passes it on. Null next or clock -> IllegalArgumentException. */
public class TimestampingSink implements MessageSink {

    public TimestampingSink(MessageSink next, Clock clock) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void send(String message) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
