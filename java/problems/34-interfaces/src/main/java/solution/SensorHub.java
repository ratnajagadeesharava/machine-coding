package solution;

import contract.Sensor;
import java.util.List;
import java.util.Map;
import java.util.OptionalDouble;

/** Collects readings from registered sensors. */
public class SensorHub {

    public void register(Sensor sensor) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public boolean unregister(String id) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public List<Sensor> sensors() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Map<String, Double> readAll() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public List<String> lastFaults() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public OptionalDouble average(String unit) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
