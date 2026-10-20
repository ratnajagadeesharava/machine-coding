package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.FoodDeliveryApi;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private FoodDeliveryApi app() {
        FoodDeliveryApi a = Candidate.of(Solution::create);
        a.addRestaurant("r1", "pizza", 0, 0);
        a.addRestaurant("r2", "pizza", 5, 5);
        a.addRestaurant("r3", "sushi", 1, 1);
        a.addItem("r1", "margherita", 250, 10);
        a.addItem("r1", "cola", 50, 100);
        return a;
    }

    private static String ready(FoodDeliveryApi a, String customer, String item) {
        String id = a.placeOrder(customer, "r1", Map.of(item, 1));
        a.accept(id);
        a.markReady(id);
        return id;
    }

    @Test
    void shouldSearchRestaurantsByCuisineAndDistance() {
        FoodDeliveryApi a = app();

        assertThat(a.searchRestaurants("pizza", 1, 1, 100)).containsExactly("r1", "r2");
        assertThat(a.searchRestaurants("pizza", 5, 4, 100)).containsExactly("r2", "r1");
        assertThat(a.searchRestaurants("pizza", 1, 1, 3)).containsExactly("r1");
        assertThat(a.searchRestaurants("sushi", 0, 0, 1)).isEmpty();
        assertThat(a.searchRestaurants("thai", 0, 0, 100)).isEmpty();
    }

    @Test
    void shouldPlaceAnOrderReserveStockAndComputeTheTotal() {
        FoodDeliveryApi a = app();

        String id = a.placeOrder("ann", "r1", Map.of("margherita", 2, "cola", 3));

        assertThat(a.totalOf(id)).isEqualTo(2 * 250 + 3 * 50);
        assertThat(a.statusOf(id)).isEqualTo("PLACED");
        assertThat(a.stockOf("r1", "margherita")).isEqualTo(8);
        assertThat(a.stockOf("r1", "cola")).isEqualTo(97);
    }

    @Test
    void shouldRefuseAnOrderWithoutEnoughStockAndReserveNothing() {
        FoodDeliveryApi a = app();

        assertThatThrownBy(() -> a.placeOrder("ann", "r1", Map.of("cola", 5, "margherita", 11))).isInstanceOf(IllegalStateException.class);

        assertThat(a.stockOf("r1", "cola")).isEqualTo(100);
        assertThat(a.stockOf("r1", "margherita")).isEqualTo(10);
    }

    @Test
    void shouldRejectInvalidOrders() {
        FoodDeliveryApi a = app();

        assertThatThrownBy(() -> a.placeOrder("ann", "nope", Map.of("cola", 1))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> a.placeOrder("ann", "r1", Map.of("sushi roll", 1))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> a.placeOrder("ann", "r1", Map.of("cola", 0))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> a.placeOrder("ann", "r1", Map.of())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> a.statusOf("nope")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldWalkThroughTheHappyPath() {
        FoodDeliveryApi a = app();
        a.addPartner("p1", 1, 1);
        String id = a.placeOrder("ann", "r1", Map.of("cola", 1));

        a.accept(id);
        assertThat(a.statusOf(id)).isEqualTo("ACCEPTED");
        a.markReady(id);
        assertThat(a.statusOf(id)).isEqualTo("READY");
        assertThat(a.partnerOf(id)).hasValue("p1");
        a.pickUp(id);
        assertThat(a.statusOf(id)).isEqualTo("PICKED_UP");
        a.deliver(id);
        assertThat(a.statusOf(id)).isEqualTo("DELIVERED");
    }

    @Test
    void shouldRejectIllegalTransitions() {
        FoodDeliveryApi a = app();
        a.addPartner("p1", 0, 0);
        String id = a.placeOrder("ann", "r1", Map.of("cola", 1));

        assertThatThrownBy(() -> a.markReady(id)).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> a.pickUp(id)).isInstanceOf(IllegalStateException.class);
        a.accept(id);
        assertThatThrownBy(() -> a.accept(id)).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> a.deliver(id)).isInstanceOf(IllegalStateException.class);
        a.markReady(id);
        a.pickUp(id);
        assertThatThrownBy(() -> a.cancel(id, "ann")).isInstanceOf(IllegalStateException.class);
        a.deliver(id);
        assertThatThrownBy(() -> a.deliver(id)).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldAssignTheNearestFreePartnerWithTiesById() {
        FoodDeliveryApi a = app();
        a.addPartner("far", 9, 9);
        a.addPartner("pb", 1, 0);
        a.addPartner("pa", 0, 1);

        String first = ready(a, "ann", "cola");
        String second = ready(a, "bob", "cola");
        String third = ready(a, "cy", "cola");

        assertThat(a.partnerOf(first)).hasValue("pa"); // tie with pb at distance 1: lower id wins
        assertThat(a.partnerOf(second)).hasValue("pb");
        assertThat(a.partnerOf(third)).hasValue("far");
    }

    @Test
    void shouldQueueReadyOrdersWhenNoPartnerIsFreeAndAssignThemInOrder() {
        FoodDeliveryApi a = app();
        a.addPartner("p1", 0, 0);
        String o1 = ready(a, "ann", "cola");
        String o2 = ready(a, "bob", "cola");
        String o3 = ready(a, "cy", "cola");

        assertThat(a.partnerOf(o1)).hasValue("p1");
        assertThat(a.partnerOf(o2)).isEmpty();
        assertThatThrownBy(() -> a.pickUp(o2)).isInstanceOf(IllegalStateException.class);

        a.pickUp(o1);
        a.deliver(o1);
        assertThat(a.partnerOf(o2)).hasValue("p1");
        assertThat(a.partnerOf(o3)).isEmpty();
        a.pickUp(o2);
        a.deliver(o2);
        assertThat(a.partnerOf(o3)).hasValue("p1");
    }

    @Test
    void shouldRestoreStockAndFreeThePartnerOnCancel() {
        FoodDeliveryApi a = app();
        a.addPartner("p1", 0, 0);
        String o1 = ready(a, "ann", "margherita");
        String o2 = ready(a, "bob", "cola");
        assertThat(a.stockOf("r1", "margherita")).isEqualTo(9);

        a.cancel(o1, "ann");

        assertThat(a.statusOf(o1)).isEqualTo("CANCELLED");
        assertThat(a.stockOf("r1", "margherita")).isEqualTo(10);
        assertThat(a.partnerOf(o2)).hasValue("p1"); // the waiting order got the freed partner
        assertThatThrownBy(() -> a.cancel(o1, "ann")).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldRemoveACancelledWaitingOrderFromTheQueue() {
        FoodDeliveryApi a = app();
        a.addPartner("p1", 0, 0);
        String o1 = ready(a, "ann", "cola");
        String o2 = ready(a, "bob", "cola");
        String o3 = ready(a, "cy", "cola");

        a.cancel(o2, "bob");
        a.pickUp(o1);
        a.deliver(o1);

        assertThat(a.partnerOf(o3)).hasValue("p1");
        assertThat(a.partnerOf(o2)).isEmpty();
    }

    @Test
    void shouldOnlyLetTheCustomerCancel() {
        FoodDeliveryApi a = app();
        String id = a.placeOrder("ann", "r1", Map.of("cola", 1));

        assertThatThrownBy(() -> a.cancel(id, "bob")).isInstanceOf(SecurityException.class);
    }

    @Test
    void shouldNeverSellMoreThanTheStockUnderContention() {
        FoodDeliveryApi a = app();

        List<Boolean> results = Concurrently.collect(50, t -> {
            try {
                a.placeOrder("user" + t, "r1", Map.of("margherita", 1));
                return true;
            } catch (IllegalStateException e) {
                return false;
            }
        });

        assertThat(results.stream().filter(r -> r).count()).isEqualTo(10);
        assertThat(a.stockOf("r1", "margherita")).isZero();
    }

    @Test
    void shouldGiveEveryPartnerAtMostOneOrderAndDeliverEverything() {
        FoodDeliveryApi a = app();
        int partners = 4;
        a.addItem("r1", "cola", 50, 1_000); // plenty of stock for 300 orders
        for (int i = 0; i < partners; i++) {
            a.addPartner("p" + i, i, i);
        }
        Map<String, AtomicInteger> load = new ConcurrentHashMap<>();
        AtomicInteger violations = new AtomicInteger();
        AtomicInteger delivered = new AtomicInteger();

        Concurrently.run(12, 60_000, t -> {
            for (int i = 0; i < 25; i++) {
                String id = a.placeOrder("user" + t, "r1", Map.of("cola", 1));
                a.accept(id);
                a.markReady(id);
                while (a.partnerOf(id).isEmpty()) {
                    Thread.yield();
                }
                String p = a.partnerOf(id).orElseThrow();
                if (load.computeIfAbsent(p, k -> new AtomicInteger()).incrementAndGet() > 1) {
                    violations.incrementAndGet();
                }
                a.pickUp(id);
                load.get(p).decrementAndGet();
                a.deliver(id);
                delivered.incrementAndGet();
            }
        });

        assertThat(violations).hasValue(0);
        assertThat(delivered).hasValue(300);
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            FoodDeliveryApi a = app();
            a.addPartner("p1", 0, 0);
            a.addPartner("p2", 1, 1);

            List<String> assigned = Concurrently.collect(6, t -> {
                String id = ready(a, "u" + t, "cola");
                return a.partnerOf(id).orElse("none");
            });

            assertThat(assigned.stream().filter(p -> !p.equals("none")).count()).as("run %d", run).isEqualTo(2);
            assertThat(assigned.stream().filter(p -> !p.equals("none")).distinct().count()).as("run %d", run).isEqualTo(2);
        }
    }
}
