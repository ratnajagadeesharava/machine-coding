package solution;

import contract.Vehicle;

/**
 * 200 cents, never a peak surcharge.
 */
public class Motorcycle implements Vehicle {
    private final String plate;

    public Motorcycle(String plate) {
        if (plate == null || plate.isEmpty() || plate.isBlank()) {
            throw new IllegalArgumentException("plate is not valid");
        }
        this.plate = plate;
    }

    @Override
    public String plate() {

        return this.plate;
    }

    @Override
    public long toll(boolean peak) {
        return 200;
    }
}
