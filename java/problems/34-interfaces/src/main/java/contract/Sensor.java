package contract;

/** Provided contract. Anything that can report a measurement is a Sensor. */
public interface Sensor {

    String id();

    /** Current reading. Throws IllegalStateException when the sensor is faulty. */
    double read();

    /** Unit of the reading, e.g. "C" or "%". */
    String unit();
}
