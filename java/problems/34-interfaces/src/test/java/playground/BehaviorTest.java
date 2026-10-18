package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Sensor;
import org.junit.jupiter.api.Test;
import solution.HumiditySensor;
import solution.SensorHub;
import solution.TemperatureSensor;

class BehaviorTest {

    /** A sensor written in the test: the hub must work with ANY implementation of the interface. */
    private static Sensor custom(String id, String unit, double value) {
        return new Sensor() {
            public String id() { return id; }
            public double read() { return value; }
            public String unit() { return unit; }
        };
    }

    @Test
    void shouldReadATemperatureSensorThroughTheInterface() {
        Sensor s = new TemperatureSensor("t1", () -> 21.5);

        assertThat(s.id()).isEqualTo("t1");
        assertThat(s.unit()).isEqualTo("C");
        assertThat(s.read()).isEqualTo(21.5);
    }

    @Test
    void shouldReportPercentForHumidity() {
        Sensor s = new HumiditySensor("h1", () -> 40.0);

        assertThat(s.unit()).isEqualTo("%");
        assertThat(s.read()).isEqualTo(40.0);
    }

    @Test
    void shouldTreatImpossibleReadingsAsFaults() {
        Sensor cold = new TemperatureSensor("t", () -> -300.0);
        Sensor wet = new HumiditySensor("h", () -> 120.0);
        Sensor edge = new HumiditySensor("e", () -> 100.0);

        assertThatThrownBy(cold::read).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(wet::read).isInstanceOf(IllegalStateException.class);
        assertThat(edge.read()).isEqualTo(100.0);
    }

    @Test
    void shouldReadAllInRegistrationOrderAndSkipFaultySensors() {
        SensorHub hub = new SensorHub();
        hub.register(new TemperatureSensor("b", () -> 10.0));
        hub.register(new HumiditySensor("broken", () -> -5.0));
        hub.register(new HumiditySensor("a", () -> 55.0));

        assertThat(hub.readAll()).containsExactly(
                java.util.Map.entry("b", 10.0), java.util.Map.entry("a", 55.0));
        assertThat(hub.lastFaults()).containsExactly("broken");
    }

    @Test
    void shouldWorkWithASensorImplementationItHasNeverSeen() {
        SensorHub hub = new SensorHub();
        hub.register(custom("lux1", "lx", 300.0));
        hub.register(new TemperatureSensor("t1", () -> 20.0));

        assertThat(hub.readAll()).containsEntry("lux1", 300.0).containsEntry("t1", 20.0);
        assertThat(hub.sensors()).hasSize(2);
    }

    @Test
    void shouldRejectDuplicateIds() {
        SensorHub hub = new SensorHub();
        hub.register(new TemperatureSensor("same", () -> 1.0));

        assertThatThrownBy(() -> hub.register(custom("same", "C", 2.0)))
                .isInstanceOf(IllegalArgumentException.class);
        assertThat(hub.sensors()).hasSize(1);
    }

    @Test
    void shouldAverageHealthySensorsOfOneUnit() {
        SensorHub hub = new SensorHub();
        hub.register(new TemperatureSensor("t1", () -> 20.0));
        hub.register(new TemperatureSensor("t2", () -> 30.0));
        hub.register(new TemperatureSensor("t3", () -> -500.0));
        hub.register(custom("t4", "C", 10.0));
        hub.register(new HumiditySensor("h1", () -> 80.0));

        assertThat(hub.average("C")).hasValue(20.0);
        assertThat(hub.average("%")).hasValue(80.0);
        assertThat(hub.average("lx")).isEmpty();
    }

    @Test
    void shouldUnregisterAndNotLetCallersEditTheList() {
        SensorHub hub = new SensorHub();
        hub.register(new TemperatureSensor("t1", () -> 20.0));
        hub.register(new TemperatureSensor("t2", () -> 21.0));

        assertThat(hub.unregister("t1")).isTrue();
        assertThat(hub.unregister("t1")).isFalse();
        assertThat(hub.sensors()).extracting(Sensor::id).containsExactly("t2");
        assertThatThrownBy(() -> hub.sensors().clear()).isInstanceOf(UnsupportedOperationException.class);
        assertThat(hub.sensors()).hasSize(1);
    }
}
