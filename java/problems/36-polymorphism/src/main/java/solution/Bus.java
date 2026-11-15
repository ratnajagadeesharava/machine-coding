package solution;

import contract.Vehicle;

/**
 * 800 cents; at peak time 1.2x (960).
 */
public class Bus implements Vehicle {
    private final String plate;

    public Bus(String plate) {
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
        return peak ? 960 : 800;
    }
}
