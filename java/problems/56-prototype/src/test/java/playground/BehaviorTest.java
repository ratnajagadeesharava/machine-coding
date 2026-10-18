package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Blueprint;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.BlueprintRegistry;
import solution.DbServer;
import solution.WebServer;

class BehaviorTest {

    /** A blueprint type written in the test: the registry must clone it through copy(), nothing else. */
    private static final class GpuServer implements Blueprint {
        static int copies;
        private String name;
        private final List<String> tags = new ArrayList<>();
        private final List<Integer> disks = new ArrayList<>(List.of(1000));

        GpuServer(String name) { this.name = name; }

        public String name() { return name; }
        public void rename(String n) { name = n; }
        public String kind() { return "gpu"; }
        public void addTag(String t) { if (!t.isBlank() && !tags.contains(t)) tags.add(t); }
        public List<String> tags() { return List.copyOf(tags); }
        public List<Integer> diskSizesGb() { return List.copyOf(disks); }
        public void resizeDisk(int i, int gb) { disks.set(i, gb); }
        public Blueprint copy() {
            copies++;
            GpuServer c = new GpuServer(name);
            c.tags.addAll(tags);
            c.disks.clear();
            c.disks.addAll(disks);
            return c;
        }
    }

    @Test
    void shouldStartAWebServerSmall() {
        Blueprint web = new WebServer("w1");

        assertThat(web.name()).isEqualTo("w1");
        assertThat(web.kind()).isEqualTo("web");
        assertThat(web.diskSizesGb()).containsExactly(20);
        assertThat(web.tags()).isEmpty();
    }

    @Test
    void shouldStartADatabaseServerWithTwoDisks() {
        Blueprint db = new DbServer("d1");

        assertThat(db.kind()).isEqualTo("db");
        assertThat(db.diskSizesGb()).containsExactly(100, 500);
    }

    @Test
    void shouldCopyTheStateAndTheConcreteType() {
        Blueprint db = new DbServer("d1");
        db.addTag("prod");
        db.addTag("eu");
        db.resizeDisk(1, 750);

        Blueprint clone = db.copy();

        assertThat(clone).isNotSameAs(db).isInstanceOf(DbServer.class);
        assertThat(clone.name()).isEqualTo("d1");
        assertThat(clone.kind()).isEqualTo("db");
        assertThat(clone.tags()).containsExactly("prod", "eu");
        assertThat(clone.diskSizesGb()).containsExactly(100, 750);
    }

    @Test
    void shouldMakeTheCopyCompletelyIndependentOfTheOriginal() {
        Blueprint original = new WebServer("w1");
        original.addTag("blue");
        Blueprint clone = original.copy();

        clone.addTag("canary");
        clone.rename("w1-clone");
        clone.resizeDisk(0, 99);
        original.addTag("green");
        original.resizeDisk(0, 40);

        assertThat(original.name()).isEqualTo("w1");
        assertThat(original.tags()).containsExactly("blue", "green");
        assertThat(original.diskSizesGb()).containsExactly(40);
        assertThat(clone.name()).isEqualTo("w1-clone");
        assertThat(clone.tags()).containsExactly("blue", "canary");
        assertThat(clone.diskSizesGb()).containsExactly(99);
    }

    @Test
    void shouldHandOutViewsThatCannotChangeTheBlueprint() {
        Blueprint web = new WebServer("w1");
        web.addTag("a");

        try {
            web.tags().add("hacked");
        } catch (UnsupportedOperationException expectedForReadOnlyViews) {
            // fine: either a read-only view or a throw-away copy
        }
        try {
            web.diskSizesGb().set(0, 1);
        } catch (UnsupportedOperationException expectedForReadOnlyViews) {
            // fine
        }

        assertThat(web.tags()).containsExactly("a");
        assertThat(web.diskSizesGb()).containsExactly(20);
    }

    @Test
    void shouldStampOutIndependentServersFromARegisteredPrototype() {
        BlueprintRegistry registry = new BlueprintRegistry();
        Blueprint prototype = new DbServer("template");
        prototype.addTag("prod");
        registry.register("db-prod", prototype);

        Blueprint one = registry.create("db-prod", "db-1");
        Blueprint two = registry.create("db-prod", "db-2");
        one.addTag("only-one");
        one.resizeDisk(0, 1);

        assertThat(one.name()).isEqualTo("db-1");
        assertThat(two.name()).isEqualTo("db-2");
        assertThat(two.tags()).containsExactly("prod");
        assertThat(two.diskSizesGb()).containsExactly(100, 500);
        assertThat(registry.create("db-prod", "db-3").tags()).containsExactly("prod");
        assertThat(one).isInstanceOf(DbServer.class).isNotSameAs(two);
    }

    @Test
    void shouldKeepTheCatalogueSafeFromLaterChangesToWhatWasRegistered() {
        BlueprintRegistry registry = new BlueprintRegistry();
        Blueprint prototype = new WebServer("template");
        registry.register("web", prototype);

        prototype.addTag("late");
        prototype.resizeDisk(0, 999);

        Blueprint created = registry.create("web", "w1");
        assertThat(created.tags()).isEmpty();
        assertThat(created.diskSizesGb()).containsExactly(20);
        assertThat(prototype.name()).isEqualTo("template");
    }

    @Test
    void shouldCloneThroughCopyForAnyBlueprintType() {
        GpuServer.copies = 0;
        BlueprintRegistry registry = new BlueprintRegistry();
        GpuServer gpu = new GpuServer("template");
        gpu.addTag("ml");

        registry.register("gpu", gpu);
        Blueprint made = registry.create("gpu", "g1");

        assertThat(GpuServer.copies).isEqualTo(2);
        assertThat(made).isInstanceOf(GpuServer.class).isNotSameAs(gpu);
        assertThat(made.name()).isEqualTo("g1");
        assertThat(made.tags()).containsExactly("ml");
        assertThat(gpu.name()).isEqualTo("template");
    }

    @Test
    void shouldReplaceAnEarlierPrototypeWithTheSameKey() {
        BlueprintRegistry registry = new BlueprintRegistry();
        registry.register("x", new WebServer("a"));
        registry.register("x", new DbServer("b"));

        assertThat(registry.create("x", "n").kind()).isEqualTo("db");
    }

    @Test
    void shouldRejectBadInput() {
        BlueprintRegistry registry = new BlueprintRegistry();
        registry.register("web", new WebServer("t"));
        Blueprint web = new WebServer("w");

        assertThatThrownBy(() -> registry.create("nope", "n")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> registry.create("web", " ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> registry.register(" ", web)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> registry.register("k", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> web.resizeDisk(5, 10)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> web.resizeDisk(0, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new WebServer(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThat(web.diskSizesGb()).containsExactly(20);
    }
}
