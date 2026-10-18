package solution;

import contract.Vehicle;

/** 200 cents, never a peak surcharge. */
public class Motorcycle implements Vehicle {

    public Motorcycle(String plate) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String plate() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public long toll(boolean peak) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
