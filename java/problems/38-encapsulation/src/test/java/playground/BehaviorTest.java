package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.StockItem;

class BehaviorTest {

    @Test
    void shouldStartEmptyAndTrackAddsAndRemoves() {
        StockItem item = new StockItem("SKU-1", 10);

        item.add(7);
        item.remove(2);

        assertThat(item.sku()).isEqualTo("SKU-1");
        assertThat(item.capacity()).isEqualTo(10);
        assertThat(item.quantity()).isEqualTo(5);
    }

    @Test
    void shouldNeverAllowQuantityAboveCapacityOrBelowZero() {
        StockItem item = new StockItem("S", 10);
        item.add(8);

        assertThatThrownBy(() -> item.add(3)).isInstanceOf(IllegalStateException.class);
        assertThat(item.quantity()).isEqualTo(8);
        assertThatThrownBy(() -> item.remove(9)).isInstanceOf(IllegalStateException.class);
        assertThat(item.quantity()).isEqualTo(8);
        item.add(2);
        assertThat(item.quantity()).isEqualTo(10);
    }

    @Test
    void shouldRejectNonPositiveAmountsAndBadConstruction() {
        StockItem item = new StockItem("S", 10);

        assertThatThrownBy(() -> item.add(0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> item.add(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> item.remove(0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new StockItem(" ", 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new StockItem("S", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThat(item.quantity()).isZero();
    }

    @Test
    void shouldTransferAtomicallyOrNotAtAll() {
        StockItem from = new StockItem("A", 10);
        StockItem to = new StockItem("B", 5);
        from.add(8);
        to.add(3);

        assertThatThrownBy(() -> from.transferTo(to, 4)).isInstanceOf(IllegalStateException.class);
        assertThat(from.quantity()).isEqualTo(8);
        assertThat(to.quantity()).isEqualTo(3);
        assertThatThrownBy(() -> from.transferTo(to, 9)).isInstanceOf(IllegalStateException.class);

        from.transferTo(to, 2);

        assertThat(from.quantity()).isEqualTo(6);
        assertThat(to.quantity()).isEqualTo(5);
    }

    @Test
    void shouldHandOutACopyOfTheTagsNotTheInternalList() {
        StockItem item = new StockItem("S", 10);
        item.addTag("Fragile");
        item.addTag("  fragile ");
        item.addTag("");
        item.addTag("Heavy");

        List<String> tags = item.tags();
        try {
            tags.clear();
            tags.add("hacked");
        } catch (UnsupportedOperationException readOnlyCopyIsFine) {
            // either an immutable list or a mutable copy satisfies the rule
        }

        assertThat(item.tags()).containsExactly("fragile", "heavy");
    }

    @Test
    void shouldKeepAReadOnlyHistoryOfSuccessfulChangesOnly() {
        StockItem a = new StockItem("A", 10);
        StockItem b = new StockItem("B", 10);
        a.add(5);
        a.remove(2);
        assertThatThrownBy(() -> a.remove(50)).isInstanceOf(IllegalStateException.class);
        a.transferTo(b, 1);

        assertThat(a.history()).containsExactly("+5", "-2", "-1 (transfer)");
        assertThat(b.history()).containsExactly("+1 (transfer)");
        assertThatThrownBy(() -> a.history().add("x")).isInstanceOf(UnsupportedOperationException.class);
        assertThat(a.history()).hasSize(3);
    }

    @Test
    void shouldHideItsStateBehindPrivateFieldsAndNoSetters() {
        assertThat(Arrays.stream(StockItem.class.getDeclaredFields()).filter(f -> !f.isSynthetic()))
                .isNotEmpty()
                .allSatisfy(f -> assertThat(Modifier.isPrivate(f.getModifiers()))
                        .as("field " + f.getName() + " must be private").isTrue());
        assertThat(Arrays.stream(StockItem.class.getDeclaredMethods()).map(Method::getName))
                .noneMatch(n -> n.startsWith("set"));
        assertThat(new StockItem("S", 1).quantity()).isZero();
    }
}
