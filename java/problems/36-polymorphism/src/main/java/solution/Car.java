package solution;

import contract.Vehicle;

/**
 * 500 cents; at peak time 1.5x (750).
 */
public class Car implements Vehicle {
    private final String plate;

    public Car(String plate) {
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
        if (peak) {
            return 750L;
        } else {
            return 500L;
        }
    }
}
