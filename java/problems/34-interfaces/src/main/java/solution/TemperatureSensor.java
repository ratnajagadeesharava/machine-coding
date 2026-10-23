package solution;

import contract.Sensor;
import java.util.function.DoubleSupplier;

/** Temperature sensor. */
public class TemperatureSensor implements Sensor {

    public TemperatureSensor(String id, DoubleSupplier probe) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String id() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public double read() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String unit() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
