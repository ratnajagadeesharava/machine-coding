package solution;

import contract.Delivery;
import contract.LegacySmsGateway;
import contract.Notifier;

/**
 * Makes the legacy gateway usable as a {@link Notifier}. It wraps (has-a) a gateway; it does not extend anything.
 */
public class LegacySmsAdapter implements Notifier {

    public LegacySmsAdapter(LegacySmsGateway legacy) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Blank phone -> Delivery(false, "invalid number") and blank message -> Delivery(false, "empty message"), in
     * that order of checking, WITHOUT calling the legacy gateway. Otherwise calls {@code legacy.submitXml} exactly
     * once with {@code <sms><to>PHONE</to><body>MESSAGE</body></sms>} where both texts have & < > " ' escaped as
     * &amp; &lt; &gt; &quot; &apos; (the ampersand first). Result: code 0 -> Delivery(true, ""); 1 -> "invalid number";
     * 2 -> "message too long"; 3 -> "unauthorized"; any other code N -> "unknown error (code N)".
     */
    @Override
    public Delivery send(String phone, String message) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
