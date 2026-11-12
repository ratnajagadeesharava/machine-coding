package solution;

import contract.Vehicle;

/**
 * 1000 cents + 300 per axle beyond 2; at peak time 1.5x. Fewer than 2 axles -> IllegalArgumentException.
 */
public class Truck implements Vehicle {
    private final String plate;
    private final int axles;

    public Truck(String plate, int axles) {
        if (plate == null || plate.isBlank()) {
            throw new IllegalArgumentException("plate is not valid");
        }
        if (axles < 2) {
            throw new IllegalArgumentException("Axels cant be less than 2");

        }
        this.plate = plate;
        this.axles = axles;
    }

    @Override
    public String plate() {

        return this.plate;
    }

    @Override
    public long toll(boolean peak) {
        if (peak) {
            return (long) (1500L + 300L * (this.axles - 2) * 1.5);
        } else {
            return (long) (1000L + 300L * (this.axles - 2));
        }
    }
}
