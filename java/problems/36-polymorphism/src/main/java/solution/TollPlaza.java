package solution;

import contract.Vehicle;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;

/**
 * Charges any Vehicle. It must not know which concrete vehicle types exist.
 */
public class TollPlaza {

    private long totalCharge;
    private final List<String> receipts;

    public TollPlaza() {
        this.totalCharge = 0;
        this.receipts = new ArrayList<>();
    }

    /**
     * Charges the vehicle's own toll, records a receipt and returns the amount. Null -> IllegalArgumentException.
     */
    public long charge(Vehicle vehicle, boolean peak) {
        if (vehicle == null) {
            throw new IllegalArgumentException("vehicle cant be null");
        }
        long charge = vehicle.toll(peak);
        this.totalCharge += charge;
        this.receipts.add(vehicle.plate() + ":" + charge);
        return charge;
    }

    /**
     * Sum of everything charged so far.
     */
    public long total() {
        return this.totalCharge;
    }

    /**
     * One entry per charge, in order, formatted "PLATE:amount" (e.g. "AB-12:500"). Cannot be modified.
     */
    public List<String> receipts() {
        return List.copyOf(receipts);
    }
}
