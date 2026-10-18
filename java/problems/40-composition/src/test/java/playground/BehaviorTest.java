package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.lang.ref.WeakReference;
import java.lang.reflect.Method;
import java.util.Arrays;
import org.junit.jupiter.api.Test;
import solution.LineItem;
import solution.Order;

class BehaviorTest {

    /** Creates an order, takes a weak reference to its first line and lets the order go out of scope. */
    private static WeakReference<LineItem> lineOfAnOrderThatIsDropped() {
        Order order = new Order("temporary");
        order.addLine("A", 1, 100);
        return new WeakReference<>(order.lineAt(0));
    }

    private static WeakReference<LineItem> weakLine(Order order, int index) {
        return new WeakReference<>(order.lineAt(index));
    }

    private static void assertCollected(WeakReference<?> ref) {
        for (int i = 0; i < 100 && ref.get() != null; i++) {
            System.gc();
            try {
                Thread.sleep(10);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
        }
        assertThat(ref.get()).as("the part should have been destroyed with / by its owner").isNull();
    }

    @Test
    void shouldCreateLinesAndComputeTheTotal() {
        Order order = new Order("o-1");

        order.addLine("A", 2, 500);
        order.addLine("B", 1, 250);

        assertThat(order.id()).isEqualTo("o-1");
        assertThat(order.lineCount()).isEqualTo(2);
        assertThat(order.total()).isEqualTo(1250);
        assertThat(order.lineAt(0).sku()).isEqualTo("A");
        assertThat(order.lineAt(0).subtotal()).isEqualTo(1000);
    }

    @Test
    void shouldMergeTheSameSkuIntoOneLine() {
        Order order = new Order("o");
        order.addLine("A", 2, 500);

        order.addLine("A", 3, 500);

        assertThat(order.lineCount()).isEqualTo(1);
        assertThat(order.lineAt(0).quantity()).isEqualTo(5);
        assertThatThrownBy(() -> order.addLine("A", 1, 999)).isInstanceOf(IllegalArgumentException.class);
        assertThat(order.lineAt(0).quantity()).isEqualTo(5);
    }

    @Test
    void shouldOwnItsLinesSoNoOneElseCanCreateOrInjectOne() {
        assertThat(LineItem.class.getConstructors()).as("LineItem must have no public constructor").isEmpty();
        assertThat(Arrays.stream(Order.class.getMethods())
                .filter(m -> m.getDeclaringClass() == Order.class)
                .map(Method::getParameterTypes)
                .flatMap(Arrays::stream))
                .doesNotContain(LineItem.class);
        Order order = new Order("o");
        order.addLine("A", 1, 10);
        assertThat(order.lineAt(0)).isSameAs(order.lineAt(0));
    }

    @Test
    void shouldDestroyTheLinesWhenTheOrderIsGone() {
        assertCollected(lineOfAnOrderThatIsDropped());
    }

    @Test
    void shouldDestroyALineWhenItIsRemovedFromTheOrder() {
        Order order = new Order("o");
        order.addLine("A", 1, 100);
        order.addLine("B", 1, 100);
        WeakReference<LineItem> lineA = weakLine(order, 0);

        assertThat(order.removeLine("A")).isTrue();
        assertThat(order.removeLine("A")).isFalse();

        assertCollected(lineA);
        assertThat(order.lineCount()).isEqualTo(1);
        assertThat(order.total()).isEqualTo(100);
    }

    @Test
    void shouldDestroyAllLinesOnCancelAndRefuseNewOnes() {
        Order order = new Order("o");
        order.addLine("A", 1, 100);
        order.addLine("B", 2, 100);
        WeakReference<LineItem> first = weakLine(order, 0);
        WeakReference<LineItem> second = weakLine(order, 1);

        order.cancel();

        assertThat(order.isCancelled()).isTrue();
        assertThat(order.lineCount()).isZero();
        assertThat(order.total()).isZero();
        assertCollected(first);
        assertCollected(second);
        assertThatThrownBy(() -> order.addLine("C", 1, 1)).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldRejectInvalidLines() {
        Order order = new Order("o");

        assertThatThrownBy(() -> order.addLine("", 1, 100)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> order.addLine("A", 0, 100)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> order.addLine("A", 1, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThat(order.lineCount()).isZero();
    }
}
