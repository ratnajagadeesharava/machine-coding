package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.FoodDeliveryApi;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldRestoreTheStockExactlyWhenManyOrdersAreCancelledConcurrently() {
        FoodDeliveryApi a = Candidate.of(Solution::create);
        a.addRestaurant("r", "x", 0, 0);
        a.addItem("r", "dish", 10, 1_000);

        Concurrently.run(10, 60_000, t -> {
            for (int i = 0; i < 100; i++) {
                String id = a.placeOrder("u" + t, "r", Map.of("dish", 1));
                if (i % 2 == 0) {
                    a.cancel(id, "u" + t);
                }
            }
        });

        assertThat(a.stockOf("r", "dish")).isEqualTo(1_000 - 500);
    }

    @Test
    void shouldHandleABurstOfReadyOrdersWithFewPartners() {
        FoodDeliveryApi a = Candidate.of(Solution::create);
        a.addRestaurant("r", "x", 0, 0);
        a.addItem("r", "dish", 10, 10_000);
        for (int i = 0; i < 3; i++) {
            a.addPartner("p" + i, i, 0);
        }
        AtomicInteger delivered = new AtomicInteger();

        Concurrently.run(16, 60_000, t -> {
            for (int i = 0; i < 100; i++) {
                String id = a.placeOrder("u" + t, "r", Map.of("dish", 1));
                a.accept(id);
                a.markReady(id);
                while (a.partnerOf(id).isEmpty()) {
                    Thread.yield();
                }
                a.pickUp(id);
                a.deliver(id);
                delivered.incrementAndGet();
            }
        });

        assertThat(delivered).hasValue(1_600);
    }
}
