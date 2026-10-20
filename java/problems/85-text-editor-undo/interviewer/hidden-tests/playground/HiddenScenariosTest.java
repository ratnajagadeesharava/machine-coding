package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Editor;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    @Test
    void shouldMatchAnUndoRedoModelOverAThousandRandomOperations() {
        Editor e = Candidate.of(Solution::create);
        java.util.Random r = new java.util.Random(42);
        java.util.ArrayDeque<String> texts = new java.util.ArrayDeque<>();
        int undoable = 0;
        for (int i = 0; i < 1_000; i++) {
            String before = e.text();
            e.moveCursor(r.nextInt(e.text().length() + 1)); // ends any group so every edit below is a new step
            if (r.nextInt(4) == 0 && !e.text().isEmpty()) {
                int from = r.nextInt(e.text().length());
                e.select(from, from + 1);
                e.backspace();
            } else {
                e.type("" + (char) ('a' + r.nextInt(26)));
            }
            texts.push(before);
            undoable++;
        }
        for (int i = 0; i < undoable; i++) {
            assertThat(e.undo()).isTrue();
            assertThat(e.text()).isEqualTo(texts.pop());
        }
        assertThat(e.undo()).isFalse();
    }

    @Test
    void shouldHandleATwentyThousandCharactersOfTyping() {
        Editor e = Candidate.of(Solution::create);

        for (int i = 0; i < 20_000; i++) {
            e.type("x");
        }

        assertThat(e.text()).hasSize(20_000);
        assertThat(e.undo()).isTrue();
        assertThat(e.text()).isEmpty();
    }
}
