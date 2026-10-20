package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.DocStore;
import contract.MergeConflictException;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    @Test
    void shouldHandleALargeDocumentWithManyVersions() {
        DocStore s = Candidate.of(Solution::create);
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < 2_000; i++) {
            sb.append("line ").append(i).append('\n');
        }
        s.create("big", "ann", sb.toString());

        for (int v = 1; v <= 200; v++) {
            s.edit("big", "ann", v, sb + "extra " + v);
        }

        assertThat(s.latest("big")).isEqualTo(201);
        assertThat(s.diff("big", 1, 201)).contains("+ extra 200");
    }

    @Test
    void shouldMergeTwoStaleEditsAgainstAMovedBase() {
        DocStore s = Candidate.of(Solution::create);
        s.create("d", "ann", "1\n2\n3\n4\n5\n6");
        s.edit("d", "ann", 1, "1\n2\n3\n4\n5\nsix");   // v2
        s.edit("d", "bob", 1, "one\n2\n3\n4\n5\n6");    // stale base 1 -> merged v3
        s.edit("d", "cy", 2, "1\n2\nthree\n4\n5\nsix"); // stale base 2 -> merged v4

        assertThat(s.content("d", 4)).isEqualTo("one\n2\nthree\n4\n5\nsix");
        try {
            s.edit("d", "dee", 1, "1\n2\n3\n4\n5\nSIX");
            throw new AssertionError("expected a conflict on the last line");
        } catch (MergeConflictException expected) {
            // fine
        }
    }
}
