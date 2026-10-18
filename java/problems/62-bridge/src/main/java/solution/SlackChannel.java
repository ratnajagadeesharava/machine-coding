package solution;

import contract.Channel;
import java.util.function.Consumer;

/** Writes {@code "SLACK @<recipient> *<title>* <body>"}. Null wire -> IllegalArgumentException. */
public class SlackChannel implements Channel {

    public SlackChannel(Consumer<String> wire) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void deliver(String recipient, String title, String body) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
