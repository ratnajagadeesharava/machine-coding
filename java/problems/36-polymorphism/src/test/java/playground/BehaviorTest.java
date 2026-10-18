package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Vehicle;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Bus;
import solution.Car;
import solution.Motorcycle;
import solution.TollPlaza;
import solution.Truck;

class BehaviorTest {

    /** Types that exist only in this test file: the plaza must handle them without any change. */
    private record Ambulance(String plate) implements Vehicle {
        public long toll(boolean peak) { return 0; }
    }

    private record Limo(String plate) implements Vehicle {
        public long toll(boolean peak) { return peak ? 5000 : 2500; }
    }

    @Test
    void shouldChargeACarMoreAtPeakTime() {
        Vehicle car = new Car("C-1");

        assertThat(car.toll(false)).isEqualTo(500);
        assertThat(car.toll(true)).isEqualTo(750);
        assertThat(car.plate()).isEqualTo("C-1");
    }

    @Test
    void shouldNeverSurchargeMotorcycles() {
        Vehicle bike = new Motorcycle("M-1");

        assertThat(bike.toll(false)).isEqualTo(200);
        assertThat(bike.toll(true)).isEqualTo(200);
    }

    @Test
    void shouldChargeTrucksPerAxle() {
        assertThat(new Truck("T", 2).toll(false)).isEqualTo(1000);
        assertThat(new Truck("T", 5).toll(false)).isEqualTo(1900);
        assertThat(new Truck("T", 4).toll(true)).isEqualTo(2400);
        assertThatThrownBy(() -> new Truck("T", 1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldChargeBusesAFlatRatePlusTwentyPercentAtPeak() {
        Vehicle bus = new Bus("B-1");

        assertThat(bus.toll(false)).isEqualTo(800);
        assertThat(bus.toll(true)).isEqualTo(960);
    }

    @Test
    void shouldSumTheRightTollForEachDifferentVehicleThroughOneCall() {
        TollPlaza plaza = new TollPlaza();
        List<Vehicle> queue = List.of(new Car("C"), new Motorcycle("M"), new Truck("T", 3), new Bus("B"));

        long charged = 0;
        for (Vehicle v : queue) {
            charged += plaza.charge(v, false);
        }

        assertThat(charged).isEqualTo(500 + 200 + 1300 + 800);
        assertThat(plaza.total()).isEqualTo(charged);
    }

    @Test
    void shouldHandleVehicleTypesItHasNeverSeen() {
        TollPlaza plaza = new TollPlaza();

        assertThat(plaza.charge(new Ambulance("AMB"), true)).isZero();
        assertThat(plaza.charge(new Limo("LIMO"), true)).isEqualTo(5000);
        assertThat(plaza.charge(new Car("C"), true)).isEqualTo(750);
        assertThat(plaza.total()).isEqualTo(5750);
    }

    @Test
    void shouldKeepReceiptsInOrderAndReadOnly() {
        TollPlaza plaza = new TollPlaza();
        plaza.charge(new Car("AB-12"), false);
        plaza.charge(new Ambulance("AMB"), false);
        plaza.charge(new Bus("BUS-9"), true);

        assertThat(plaza.receipts()).containsExactly("AB-12:500", "AMB:0", "BUS-9:960");
        assertThatThrownBy(() -> plaza.receipts().add("x")).isInstanceOf(UnsupportedOperationException.class);
        assertThat(plaza.receipts()).hasSize(3);
    }

    @Test
    void shouldRejectANullVehicleWithoutChangingTotals() {
        TollPlaza plaza = new TollPlaza();
        plaza.charge(new Car("C"), false);

        assertThatThrownBy(() -> plaza.charge(null, false)).isInstanceOf(IllegalArgumentException.class);
        assertThat(plaza.total()).isEqualTo(500);
    }
}
