package solution;

import contract.Aircraft;
import contract.Tower;
import java.util.List;
import java.util.Optional;

/** Coordinates the aircraft landing on one runway. */
public class ControlTower implements Tower {

    @Override
    public void requestLanding(Aircraft aircraft) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void landed(Aircraft aircraft) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public boolean cancel(Aircraft aircraft) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void declareEmergency(Aircraft aircraft) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public Optional<String> onRunway() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public List<String> queue() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
