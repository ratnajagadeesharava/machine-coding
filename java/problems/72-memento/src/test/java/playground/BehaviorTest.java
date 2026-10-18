package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.Arrays;
import org.junit.jupiter.api.Test;
import solution.GameCharacter;
import solution.SaveSlots;
import solution.UndoHistory;

class BehaviorTest {

    private static void assertState(GameCharacter c, int hp, int level, int x, int y, String... items) {
        assertThat(c.hp()).as("hp").isEqualTo(hp);
        assertThat(c.level()).as("level").isEqualTo(level);
        assertThat(c.x()).as("x").isEqualTo(x);
        assertThat(c.y()).as("y").isEqualTo(y);
        assertThat(c.inventory()).as("inventory").containsExactly(items);
    }

    @Test
    void shouldStartFreshAndChangeThroughItsOwnMethods() {
        GameCharacter c = new GameCharacter("Aria");

        assertState(c, 100, 1, 0, 0);
        c.takeDamage(30);
        c.heal(5);
        c.levelUp();
        c.moveTo(3, -4);
        c.pickUp("sword");
        c.pickUp("potion");
        assertThat(c.drop("shield")).isFalse();
        assertThat(c.drop("potion")).isTrue();

        assertState(c, 75, 2, 3, -4, "sword");
        assertThat(c.name()).isEqualTo("Aria");
        c.heal(1000);
        c.takeDamage(1000);
        assertThat(c.hp()).isZero();
    }

    @Test
    void shouldRestoreTheExactSavedState() {
        GameCharacter c = new GameCharacter("Aria");
        c.takeDamage(40);
        c.levelUp();
        c.moveTo(7, 8);
        c.pickUp("sword");
        c.pickUp("map");
        GameCharacter.Snapshot saved = c.save();

        c.takeDamage(50);
        c.levelUp();
        c.moveTo(-1, -1);
        c.drop("sword");
        c.pickUp("gem");
        c.restore(saved);

        assertState(c, 60, 2, 7, 8, "sword", "map");
    }

    @Test
    void shouldKeepASnapshotIndependentOfLaterChangesAndReusable() {
        GameCharacter c = new GameCharacter("Aria");
        c.pickUp("sword");
        GameCharacter.Snapshot saved = c.save();

        c.pickUp("shield");
        c.takeDamage(10);
        c.restore(saved);
        assertState(c, 100, 1, 0, 0, "sword");
        c.pickUp("bow");
        c.restore(saved);

        assertState(c, 100, 1, 0, 0, "sword");
    }

    @Test
    void shouldAllowRestoringIntoAnotherCharacterWithoutChangingItsName() {
        GameCharacter hero = new GameCharacter("Aria");
        hero.levelUp();
        hero.pickUp("sword");
        GameCharacter twin = new GameCharacter("Bryn");

        twin.restore(hero.save());

        assertState(twin, 100, 2, 0, 0, "sword");
        assertThat(twin.name()).isEqualTo("Bryn");
        twin.pickUp("bow");
        assertThat(hero.inventory()).containsExactly("sword");
    }

    @Test
    void shouldStoreNamedSaveSlots() {
        GameCharacter c = new GameCharacter("Aria");
        SaveSlots slots = new SaveSlots();
        slots.save("before-boss", c);
        c.takeDamage(90);
        c.moveTo(5, 5);
        slots.save("after-boss", c);
        c.heal(100);

        assertThat(slots.slotCount()).isEqualTo(2);
        assertThat(slots.load("after-boss", c)).isTrue();
        assertState(c, 10, 1, 5, 5);
        assertThat(slots.load("before-boss", c)).isTrue();
        assertState(c, 100, 1, 0, 0);
        assertThat(slots.load("nope", c)).isFalse();
        assertState(c, 100, 1, 0, 0);

        c.levelUp();
        slots.save("before-boss", c);
        assertThat(slots.slotCount()).isEqualTo(2);
        c.takeDamage(1);
        slots.load("before-boss", c);
        assertState(c, 100, 2, 0, 0);
    }

    @Test
    void shouldUndoAndRedoAcrossCheckpoints() {
        GameCharacter c = new GameCharacter("Aria");
        UndoHistory history = new UndoHistory(c, 10);
        history.checkpoint();
        c.takeDamage(30);
        history.checkpoint();
        c.levelUp();
        c.takeDamage(20);
        assertState(c, 50, 2, 0, 0);

        assertThat(history.undo()).isTrue();
        assertState(c, 70, 1, 0, 0);
        assertThat(history.undo()).isTrue();
        assertState(c, 100, 1, 0, 0);
        assertThat(history.undo()).isFalse();
        assertThat(history.redo()).isTrue();
        assertState(c, 70, 1, 0, 0);
        assertThat(history.redo()).isTrue();
        assertState(c, 50, 2, 0, 0);
        assertThat(history.redo()).isFalse();
        assertThat(history.undoDepth()).isEqualTo(2);
        assertThat(history.redoDepth()).isZero();
    }

    @Test
    void shouldForgetTheRedoFutureWhenANewCheckpointIsTaken() {
        GameCharacter c = new GameCharacter("Aria");
        UndoHistory history = new UndoHistory(c, 10);
        history.checkpoint();
        c.takeDamage(10);
        history.undo();
        assertThat(history.redoDepth()).isEqualTo(1);

        history.checkpoint();

        assertThat(history.redoDepth()).isZero();
        assertThat(history.redo()).isFalse();
        assertThat(history.undoDepth()).isEqualTo(1);
    }

    @Test
    void shouldDropTheOldestCheckpointWhenTheLimitIsExceeded() {
        GameCharacter c = new GameCharacter("Aria");
        UndoHistory history = new UndoHistory(c, 2);
        for (int i = 0; i < 3; i++) {
            history.checkpoint();
            c.takeDamage(10);
        }
        assertState(c, 70, 1, 0, 0);

        assertThat(history.undoDepth()).isEqualTo(2);
        assertThat(history.undo()).isTrue();
        assertState(c, 80, 1, 0, 0);
        assertThat(history.undo()).isTrue();
        assertState(c, 90, 1, 0, 0);
        assertThat(history.undo()).isFalse();
    }

    @Test
    void shouldKeepTheMementoOpaqueAndTheOriginatorsFieldsPrivate() {
        GameCharacter.Snapshot saved = new GameCharacter("Aria").save();
        assertThat(saved).isNotNull();

        Class<?> memento = GameCharacter.Snapshot.class;
        assertThat(memento.getConstructors()).as("no public constructor").isEmpty();
        assertThat(Arrays.stream(memento.getDeclaredMethods()).filter(m -> Modifier.isPublic(m.getModifiers()) && !m.isSynthetic()))
                .as("no public methods").isEmpty();
        assertThat(Arrays.stream(memento.getDeclaredFields()).filter(f -> !Modifier.isPrivate(f.getModifiers()) && !f.isSynthetic()))
                .as("no non-private fields").isEmpty();
        assertThat(memento.getDeclaredFields()).as("the memento really stores the state").isNotEmpty();
        for (Field f : GameCharacter.class.getDeclaredFields()) {
            if (!Modifier.isStatic(f.getModifiers())) {
                assertThat(Modifier.isPrivate(f.getModifiers())).as("GameCharacter field " + f.getName()).isTrue();
            }
        }
        assertThat(Arrays.stream(GameCharacter.class.getDeclaredMethods()).map(Method::getName)).noneMatch(n -> n.startsWith("set"));
    }

    @Test
    void shouldRejectInvalidInput() {
        GameCharacter c = new GameCharacter("Aria");
        c.pickUp("a");
        c.pickUp("b");
        c.pickUp("c");

        assertThatThrownBy(() -> new GameCharacter(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.takeDamage(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.heal(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.pickUp("d")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> c.pickUp(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.restore(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new SaveSlots().save(" ", c)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new SaveSlots().save("x", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new UndoHistory(c, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new UndoHistory(null, 3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.inventory().add("hack")).isInstanceOf(UnsupportedOperationException.class);
        assertThat(c.inventory()).containsExactly("a", "b", "c");
    }
}
