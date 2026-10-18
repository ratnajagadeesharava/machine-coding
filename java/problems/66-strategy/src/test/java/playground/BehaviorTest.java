package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Parcel;
import contract.ShippingStrategy;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.ByWeight;
import solution.DistanceBased;
import solution.FlatRate;
import solution.FreeOverThreshold;
import solution.ShippingAdvisor;
import solution.ShippingCalculator;

class BehaviorTest {

    private static final Parcel LIGHT_NEAR = new Parcel(500, 10, 2000);
    private static final Parcel HEAVY_FAR = new Parcel(2500, 300, 20_000);

    /** A strategy written in the test: customer collects the parcel themselves. */
    private static final class Pickup implements ShippingStrategy {
        public String name() { return "pickup"; }
        public long cost(Parcel p) { return 0; }
    }

    @Test
    void shouldChargeAFlatRateRegardlessOfTheParcel() {
        ShippingStrategy flat = new FlatRate(499);

        assertThat(flat.cost(LIGHT_NEAR)).isEqualTo(499);
        assertThat(flat.cost(HEAVY_FAR)).isEqualTo(499);
        assertThat(flat.name()).isEqualTo("flat");
    }

    @Test
    void shouldChargeByTheStartedKilogram() {
        ShippingStrategy byWeight = new ByWeight(200);

        assertThat(byWeight.cost(new Parcel(1, 0, 0))).isEqualTo(200);
        assertThat(byWeight.cost(new Parcel(1000, 0, 0))).isEqualTo(200);
        assertThat(byWeight.cost(new Parcel(1001, 0, 0))).isEqualTo(400);
        assertThat(byWeight.cost(HEAVY_FAR)).isEqualTo(600);
        assertThat(byWeight.name()).isEqualTo("weight");
    }

    @Test
    void shouldChargeByDistance() {
        ShippingStrategy byDistance = new DistanceBased(3);

        assertThat(byDistance.cost(LIGHT_NEAR)).isEqualTo(30);
        assertThat(byDistance.cost(HEAVY_FAR)).isEqualTo(900);
        assertThat(byDistance.cost(new Parcel(10, 0, 0))).isZero();
        assertThat(byDistance.name()).isEqualTo("distance");
    }

    @Test
    void shouldShipFreeAboveTheThresholdAndDelegateBelowIt() {
        ShippingStrategy strategy = new FreeOverThreshold(10_000, new FlatRate(499));

        assertThat(strategy.cost(new Parcel(100, 5, 10_000))).isZero();
        assertThat(strategy.cost(HEAVY_FAR)).isZero();
        assertThat(strategy.cost(new Parcel(100, 5, 9_999))).isEqualTo(499);
        assertThat(strategy.name()).isEqualTo("free>=10000/flat");
    }

    @Test
    void shouldLetStrategiesWrapOtherStrategies() {
        ShippingStrategy nested = new FreeOverThreshold(50_000, new FreeOverThreshold(10_000, new ByWeight(100)));

        assertThat(nested.cost(new Parcel(2500, 1, 60_000))).isZero();
        assertThat(nested.cost(new Parcel(2500, 1, 20_000))).isZero();
        assertThat(nested.cost(new Parcel(2500, 1, 5_000))).isEqualTo(300);
        assertThat(nested.name()).isEqualTo("free>=50000/free>=10000/weight");
    }

    @Test
    void shouldPriceWithWhicheverStrategyIsCurrentAndSwapAtRuntime() {
        ShippingCalculator calc = new ShippingCalculator(new FlatRate(499));
        assertThat(calc.quote(HEAVY_FAR)).isEqualTo(499);

        calc.setStrategy(new DistanceBased(3));
        assertThat(calc.quote(HEAVY_FAR)).isEqualTo(900);
        assertThat(calc.strategy().name()).isEqualTo("distance");

        calc.setStrategy(new Pickup());
        assertThat(calc.quote(HEAVY_FAR)).isZero();
        assertThat(calc.strategy().name()).isEqualTo("pickup");
    }

    @Test
    void shouldPickTheCheapestStrategyAndPreferTheEarlierOneOnATie() {
        List<ShippingStrategy> options = List.of(new FlatRate(499), new ByWeight(200), new DistanceBased(3), new Pickup());

        assertThat(ShippingAdvisor.cheapest(options, HEAVY_FAR).name()).isEqualTo("pickup");
        assertThat(ShippingAdvisor.cheapest(options.subList(0, 3), LIGHT_NEAR).name()).isEqualTo("distance");
        assertThat(ShippingAdvisor.cheapest(List.of(new FlatRate(100), new ByWeight(100)), new Parcel(1000, 1, 1)).name())
                .isEqualTo("flat");
    }

    @Test
    void shouldRejectInvalidParcelsAndMissingStrategies() {
        ShippingCalculator calc = new ShippingCalculator(new FlatRate(1));

        assertThatThrownBy(() -> calc.quote(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> calc.quote(new Parcel(0, 1, 1))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> calc.quote(new Parcel(1, -1, 1))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> calc.quote(new Parcel(1, 1, -1))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new ShippingCalculator(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> calc.setStrategy(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> ShippingAdvisor.cheapest(List.of(), LIGHT_NEAR)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new FlatRate(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new FreeOverThreshold(100, null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(calc.quote(LIGHT_NEAR)).isEqualTo(1);
    }
}
