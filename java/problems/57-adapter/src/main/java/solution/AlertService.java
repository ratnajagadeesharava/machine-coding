package solution;

import contract.Notifier;
import java.util.List;

/** Application code: it only knows the {@link Notifier} interface. */
public class AlertService {

    public AlertService(Notifier notifier) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Sends the message to every phone, in order. Returns one entry {@code "<phone>: <error>"} per failed delivery
     * (empty list when everything went out). A null phone list means nobody to alert.
     */
    public List<String> alertAll(List<String> phones, String message) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
