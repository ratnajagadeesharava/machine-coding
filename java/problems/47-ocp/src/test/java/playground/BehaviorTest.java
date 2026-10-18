package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.FeePolicy;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.CarPolicy;
import solution.FeeCalculator;
import solution.MotorcyclePolicy;

class BehaviorTest {

    /** New requirement, written in the test: electric vehicles park free for an hour, then 50 per started 30 min. */
    private static final class ElectricPolicy implements FeePolicy {
        public boolean supports(String type) { return "electric".equalsIgnoreCase(type); }
        public long fee(int minutes) { return minutes <= 60 ? 0 : 50L * ((minutes - 60 + 29) / 30); }
    }

    private static FeePolicy flat(String type, long fee) {
        return new FeePolicy() {
            public boolean supports(String t) { return type.equalsIgnoreCase(t); }
            public long fee(int minutes) { return fee; }
        };
    }

    private static FeeCalculator standard() {
        return new FeeCalculator(List.of(new CarPolicy(), new MotorcyclePolicy()));
    }

    @Test
    void shouldPriceCarsByTheHourThenByHalfHour() {
        FeeCalculator calc = standard();

        assertThat(calc.fee("car", 0)).isZero();
        assertThat(calc.fee("car", 30)).isEqualTo(200);
        assertThat(calc.fee("car", 60)).isEqualTo(200);
        assertThat(calc.fee("car", 61)).isEqualTo(300);
        assertThat(calc.fee("CAR", 90)).isEqualTo(300);
        assertThat(calc.fee("Car", 91)).isEqualTo(400);
        assertThat(calc.fee("car", 1000)).isEqualTo(1500);
    }

    @Test
    void shouldChargeMotorcyclesHalf() {
        FeeCalculator calc = standard();

        assertThat(calc.fee("motorcycle", 30)).isEqualTo(100);
        assertThat(calc.fee("Motorcycle", 91)).isEqualTo(200);
        assertThat(calc.fee("motorcycle", 1000)).isEqualTo(750);
    }

    @Test
    void shouldSupportANewVehicleTypeWithoutChangingTheCalculator() {
        FeeCalculator calc = new FeeCalculator(List.of(new CarPolicy(), new MotorcyclePolicy(), new ElectricPolicy()));

        assertThat(calc.fee("electric", 45)).isZero();
        assertThat(calc.fee("electric", 61)).isEqualTo(50);
        assertThat(calc.fee("electric", 120)).isEqualTo(100);
        assertThat(calc.fee("car", 61)).isEqualTo(300);
    }

    @Test
    void shouldAcceptNewPoliciesAtRuntime() {
        FeeCalculator calc = standard();
        assertThatThrownBy(() -> calc.fee("boat", 10)).isInstanceOf(IllegalArgumentException.class);

        calc.register(flat("boat", 999));

        assertThat(calc.fee("boat", 10)).isEqualTo(999);
        assertThat(calc.fee("car", 30)).isEqualTo(200);
    }

    @Test
    void shouldLetTheLatestRegisteredPolicyOverrideAnEarlierOne() {
        FeeCalculator calc = standard();

        calc.register(flat("car", 0));

        assertThat(calc.fee("car", 500)).isZero();
        assertThat(calc.fee("motorcycle", 30)).isEqualTo(100);
    }

    @Test
    void shouldWorkWithNoBuiltInVehiclesAtAll() {
        FeeCalculator calc = new FeeCalculator(new ArrayList<>(List.of(flat("boat", 42))));

        assertThat(calc.fee("boat", 5)).isEqualTo(42);
        assertThatThrownBy(() -> calc.fee("car", 5)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectBadInput() {
        FeeCalculator calc = standard();

        assertThatThrownBy(() -> calc.fee("car", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> calc.fee(null, 10)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> calc.fee("spaceship", 10)).isInstanceOf(IllegalArgumentException.class);
        assertThat(calc.fee("car", 30)).isEqualTo(200);
    }

    @Test
    void shouldKeepVehicleKnowledgeOutOfTheCalculatorSource() throws IOException {
        assertThat(standard().fee("car", 30)).isEqualTo(200);

        String source = Files.readString(Path.of("src/main/java/solution/FeeCalculator.java")).toLowerCase();
        assertThat(source).as("FeeCalculator must not mention vehicle types").doesNotContain("\"car\"", "\"motorcycle\"", "switch");
    }
}
