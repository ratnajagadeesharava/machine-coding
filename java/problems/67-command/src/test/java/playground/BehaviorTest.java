package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Command;
import contract.TextBuffer;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.CommandHistory;
import solution.DeleteCommand;
import solution.InsertCommand;
import solution.MacroCommand;

class BehaviorTest {

    /** A command written in the test: upper-cases the whole buffer, remembering the old text. */
    private static final class UppercaseCommand implements Command {
        private final TextBuffer buffer;
        private String before;

        UppercaseCommand(TextBuffer buffer) { this.buffer = buffer; }

        public void execute() {
            before = buffer.text();
            buffer.delete(0, buffer.length());
            buffer.insert(0, before.toUpperCase());
        }

        public void undo() {
            buffer.delete(0, buffer.length());
            buffer.insert(0, before);
        }
    }

    private final TextBuffer buffer = new TextBuffer();

    private void type(String s) {
        buffer.insert(buffer.length(), s);
    }

    @Test
    void shouldInsertAndUndoTheInsert() {
        type("Hello");
        Command insert = new InsertCommand(buffer, 5, ", world");

        insert.execute();
        assertThat(buffer.text()).isEqualTo("Hello, world");
        insert.undo();

        assertThat(buffer.text()).isEqualTo("Hello");
    }

    @Test
    void shouldRememberWhatItDeletedSoItCanPutItBack() {
        type("Hello, world");
        Command delete = new DeleteCommand(buffer, 5, 12);

        delete.execute();
        assertThat(buffer.text()).isEqualTo("Hello");
        delete.undo();

        assertThat(buffer.text()).isEqualTo("Hello, world");
    }

    @Test
    void shouldRunAMacroAsOneUnitAndUndoItInReverse() {
        type("abc");
        Command macro = new MacroCommand(List.of(
                new InsertCommand(buffer, 3, "def"), new DeleteCommand(buffer, 0, 1), new InsertCommand(buffer, 0, "X")));

        macro.execute();
        assertThat(buffer.text()).isEqualTo("Xbcdef");
        macro.undo();

        assertThat(buffer.text()).isEqualTo("abc");
    }

    @Test
    void shouldRollBackAMacroThatFailsHalfWay() {
        type("abc");
        Command macro = new MacroCommand(List.of(
                new InsertCommand(buffer, 3, "def"), new DeleteCommand(buffer, 0, 99), new InsertCommand(buffer, 0, "X")));

        assertThatThrownBy(macro::execute).isInstanceOf(IllegalArgumentException.class);

        assertThat(buffer.text()).isEqualTo("abc");
    }

    @Test
    void shouldUndoAndRedoThroughTheHistoryInOrder() {
        CommandHistory history = new CommandHistory();
        history.execute(new InsertCommand(buffer, 0, "one"));
        history.execute(new InsertCommand(buffer, 3, " two"));
        history.execute(new InsertCommand(buffer, 7, " three"));
        assertThat(buffer.text()).isEqualTo("one two three");

        assertThat(history.undo()).isTrue();
        assertThat(history.undo()).isTrue();
        assertThat(buffer.text()).isEqualTo("one");
        assertThat(history.redo()).isTrue();
        assertThat(buffer.text()).isEqualTo("one two");
        assertThat(history.canUndo()).isTrue();
        assertThat(history.canRedo()).isTrue();
    }

    @Test
    void shouldClearTheRedoListWhenANewCommandIsExecuted() {
        CommandHistory history = new CommandHistory();
        history.execute(new InsertCommand(buffer, 0, "a"));
        history.execute(new InsertCommand(buffer, 1, "b"));
        history.undo();
        assertThat(history.canRedo()).isTrue();

        history.execute(new InsertCommand(buffer, 1, "c"));

        assertThat(history.canRedo()).isFalse();
        assertThat(history.redo()).isFalse();
        assertThat(buffer.text()).isEqualTo("ac");
    }

    @Test
    void shouldReturnFalseWhenThereIsNothingToUndoOrRedoAndNotRecordFailedCommands() {
        CommandHistory history = new CommandHistory();
        assertThat(history.undo()).isFalse();
        assertThat(history.redo()).isFalse();
        assertThat(history.canUndo()).isFalse();

        assertThatThrownBy(() -> history.execute(new DeleteCommand(buffer, 0, 5))).isInstanceOf(IllegalArgumentException.class);

        assertThat(history.canUndo()).isFalse();
        assertThat(buffer.text()).isEmpty();
        assertThatThrownBy(() -> history.execute(null)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldHandleACommandItHasNeverSeen() {
        type("quiet");
        CommandHistory history = new CommandHistory();

        history.execute(new UppercaseCommand(buffer));
        history.execute(new InsertCommand(buffer, 5, "!"));
        assertThat(buffer.text()).isEqualTo("QUIET!");
        history.undo();
        history.undo();

        assertThat(buffer.text()).isEqualTo("quiet");
        history.redo();
        assertThat(buffer.text()).isEqualTo("QUIET");
    }

    @Test
    void shouldLogCommandsWithoutAnythingKnowingWhatTheyDo() {
        List<String> log = new ArrayList<>();
        Command logged = new Command() {
            public void execute() { log.add("run"); }
            public void undo() { log.add("undo"); }
        };
        CommandHistory history = new CommandHistory();

        history.execute(logged);
        history.undo();
        history.redo();

        assertThat(log).containsExactly("run", "undo", "run");
    }

    @Test
    void shouldRejectBadConstruction() {
        assertThatThrownBy(() -> new InsertCommand(null, 0, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new InsertCommand(buffer, 0, null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new DeleteCommand(null, 0, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new MacroCommand(List.of())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new MacroCommand(null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(new InsertCommand(buffer, 0, "ok")).isNotNull();
    }
}
