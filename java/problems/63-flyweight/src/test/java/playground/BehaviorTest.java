package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Set;
import java.util.concurrent.CyclicBarrier;
import org.junit.jupiter.api.Test;
import solution.MapLayer;
import solution.Marker;
import solution.MarkerStyle;
import solution.StyleFactory;

class BehaviorTest {

    @Test
    void shouldShareOneStyleObjectPerShapeAndColour() {
        StyleFactory factory = new StyleFactory();

        MarkerStyle a = factory.get("pin", "red");
        MarkerStyle b = factory.get("pin", "red");
        MarkerStyle c = factory.get("pin", "blue");
        MarkerStyle d = factory.get("flag", "red");
        MarkerStyle e = factory.get("pin", "Red");

        assertThat(a).isSameAs(b);
        assertThat(a).isNotSameAs(c).isNotSameAs(d).isNotSameAs(e);
        assertThat(factory.instancesCreated()).isEqualTo(4);
        assertThat(a.shape()).isEqualTo("pin");
        assertThat(a.color()).isEqualTo("red");
        assertThat(a.bitmapSize()).isEqualTo(4096);
    }

    @Test
    void shouldCreateOnlyThreeStylesForTenThousandMarkers() {
        StyleFactory factory = new StyleFactory();
        MapLayer layer = new MapLayer(factory);
        String[][] kinds = {{"pin", "red"}, {"pin", "blue"}, {"flag", "green"}};

        for (int i = 0; i < 10_000; i++) {
            String[] kind = kinds[i % 3];
            layer.addMarker(i % 90, i % 180, "m" + i, kind[0], kind[1]);
        }

        Set<MarkerStyle> distinct = Collections.newSetFromMap(new IdentityHashMap<>());
        for (Marker m : layer.markers()) {
            distinct.add(m.style());
        }
        assertThat(layer.markerCount()).isEqualTo(10_000);
        assertThat(factory.instancesCreated()).isEqualTo(3);
        assertThat(distinct).hasSize(3);
    }

    @Test
    void shouldKeepPerMarkerDataOnTheMarkerAndHeavyDataOnTheStyle() {
        StyleFactory factory = new StyleFactory();
        MapLayer layer = new MapLayer(factory);
        layer.addMarker(12.5, -3.25, "Cafe", "pin", "red");
        layer.addMarker(1.0, 2.0, "Bank", "pin", "red");

        Marker first = layer.markers().get(0);
        Marker second = layer.markers().get(1);

        assertThat(first.lat()).isEqualTo(12.5);
        assertThat(first.lng()).isEqualTo(-3.25);
        assertThat(first.label()).isEqualTo("Cafe");
        assertThat(second.label()).isEqualTo("Bank");
        assertThat(first.style()).isSameAs(second.style()).isSameAs(factory.get("pin", "red"));
        for (Field f : Marker.class.getDeclaredFields()) {
            assertThat(f.getType()).as("Marker field " + f.getName() + " must not be a bitmap").isNotEqualTo(byte[].class);
        }
    }

    @Test
    void shouldCreateEachStyleOnlyOnceEvenWhenManyThreadsAskTogether() throws Exception {
        StyleFactory factory = new StyleFactory();
        int threads = 32;
        CyclicBarrier startTogether = new CyclicBarrier(threads);
        List<MarkerStyle[]> results = Collections.synchronizedList(new ArrayList<>());
        List<Thread> workers = new ArrayList<>();
        for (int t = 0; t < threads; t++) {
            Thread worker = new Thread(() -> {
                try {
                    startTogether.await();
                } catch (Exception e) {
                    throw new IllegalStateException(e);
                }
                MarkerStyle[] seen = new MarkerStyle[5];
                for (int round = 0; round < 200; round++) {
                    for (int k = 0; k < 5; k++) {
                        seen[k] = factory.get("pin", "color-" + k);
                    }
                }
                results.add(seen);
            });
            workers.add(worker);
            worker.start();
        }
        for (Thread worker : workers) {
            worker.join();
        }

        assertThat(factory.instancesCreated()).isEqualTo(5);
        for (int k = 0; k < 5; k++) {
            Set<MarkerStyle> distinct = Collections.newSetFromMap(new IdentityHashMap<>());
            for (MarkerStyle[] seen : results) {
                distinct.add(seen[k]);
            }
            assertThat(distinct).as("styles seen for key " + k).hasSize(1);
        }
    }

    @Test
    void shouldDescribeEachMarkerOnItsOwnLine() {
        MapLayer layer = new MapLayer(new StyleFactory());
        layer.addMarker(12.5, -3.25, "Cafe", "pin", "red");
        layer.addMarker(-40.0, 100.5, "Harbour", "flag", "blue");

        assertThat(layer.render()).containsExactly("pin/red @ 12.5,-3.25 Cafe", "flag/blue @ -40.0,100.5 Harbour");
    }

    @Test
    void shouldOnlyLetTheFactoryCreateStyles() {
        assertThat(MarkerStyle.class.getConstructors()).as("MarkerStyle must have no public constructor").isEmpty();
        assertThat(new StyleFactory().get("pin", "red")).isNotNull();
    }

    @Test
    void shouldRejectInvalidInput() {
        StyleFactory factory = new StyleFactory();
        MapLayer layer = new MapLayer(factory);

        assertThatThrownBy(() -> factory.get(" ", "red")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> factory.get("pin", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> layer.addMarker(91, 0, "x", "pin", "red")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> layer.addMarker(0, -181, "x", "pin", "red")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> layer.addMarker(0, 0, "x", "", "red")).isInstanceOf(IllegalArgumentException.class);
        assertThat(layer.markerCount()).isZero();
        assertThat(factory.instancesCreated()).isZero();
    }

    @Test
    void shouldNotLetCallersChangeTheLayerThroughTheMarkerList() {
        MapLayer layer = new MapLayer(new StyleFactory());
        layer.addMarker(1, 1, "a", "pin", "red");

        assertThatThrownBy(() -> layer.markers().clear()).isInstanceOf(UnsupportedOperationException.class);
        assertThat(layer.markers()).hasSize(1);
        assertThat(Arrays.stream(Marker.class.getDeclaredFields()).count()).isGreaterThanOrEqualTo(4);
    }
}
