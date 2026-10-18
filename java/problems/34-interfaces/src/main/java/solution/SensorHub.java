package solution;

import contract.Sensor;
import java.util.List;
import java.util.Map;
import java.util.OptionalDouble;

/** Talks to sensors only through the {@link Sensor} interface. */
public class SensorHub {

    /** Throws IllegalArgumentException if a sensor with the same id is already registered. */
    public void register(Sensor sensor) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** @return true if a sensor with that id was registered. */
    public boolean unregister(String id) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Registered sensors in registration order. The returned list cannot be modified. */
    public List<Sensor> sensors() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** id -> reading, in registration order. Faulty sensors are left out. */
    public Map<String, Double> readAll() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Ids of the sensors that were faulty during the last {@link #readAll()} (empty before the first call). */
    public List<String> lastFaults() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Mean of the healthy sensors that use this unit; empty when there are none. */
    public OptionalDouble average(String unit) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
