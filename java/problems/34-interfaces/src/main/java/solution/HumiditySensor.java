package solution;

import contract.Sensor;
import java.util.function.DoubleSupplier;

/** Humidity sensor. */
public class HumiditySensor implements Sensor {

    public HumiditySensor(String id, DoubleSupplier probe) {
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
