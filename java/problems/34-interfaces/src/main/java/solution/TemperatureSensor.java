package solution;

import contract.Sensor;
import java.util.function.DoubleSupplier;

/** Unit "C". Faulty (IllegalStateException on read) when the probe returns less than -273.15. */
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
