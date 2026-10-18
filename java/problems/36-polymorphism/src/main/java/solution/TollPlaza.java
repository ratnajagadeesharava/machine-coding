package solution;

import contract.Vehicle;
import java.util.List;

/** Charges any Vehicle. It must not know which concrete vehicle types exist. */
public class TollPlaza {

    /** Charges the vehicle's own toll, records a receipt and returns the amount. Null -> IllegalArgumentException. */
    public long charge(Vehicle vehicle, boolean peak) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Sum of everything charged so far. */
    public long total() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** One entry per charge, in order, formatted "PLATE:amount" (e.g. "AB-12:500"). Cannot be modified. */
    public List<String> receipts() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
