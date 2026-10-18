package solution;

import contract.Vehicle;

/** 1000 cents + 300 per axle beyond 2; at peak time 1.5x. Fewer than 2 axles -> IllegalArgumentException. */
public class Truck implements Vehicle {

    public Truck(String plate, int axles) {
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
