package solution;

import contract.Delivery;
import contract.LegacySmsGateway;
import contract.Notifier;

/** A {@link Notifier} backed by the legacy gateway. */
public class LegacySmsAdapter implements Notifier {

    public LegacySmsAdapter(LegacySmsGateway legacy) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** See README for the spec. */
    @Override
    public Delivery send(String phone, String message) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
