package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.DocStore;
import contract.MergeConflictException;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static final String FIVE = "l1\nl2\nl3\nl4\nl5";

    private DocStore store() {
        return Candidate.of(Solution::create);
    }

    @Test
    void shouldCreateAndReadVersions() {
        DocStore s = store();

        s.create("d", "ann", "hello");

        assertThat(s.latest("d")).isEqualTo(1);
        assertThat(s.content("d", 1)).isEqualTo("hello");
        assertThat(s.history("d")).containsExactly("1:ann");
    }

    @Test
    void shouldRejectDuplicatesAndUnknownDocumentsOrVersions() {
        DocStore s = store();
        s.create("d", "ann", "x");

        assertThatThrownBy(() -> s.create("d", "bob", "y")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.latest("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.content("d", 2)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.content("d", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.edit("d", "bob", 2, "z")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.edit("d", "bob", 0, "z")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldAppendAVersionPerEditAndKeepOldOnes() {
        DocStore s = store();
        s.create("d", "ann", "one");

        int v2 = s.edit("d", "bob", 1, "two");
        int v3 = s.edit("d", "ann", 2, "three");

        assertThat(v2).isEqualTo(2);
        assertThat(v3).isEqualTo(3);
        assertThat(s.content("d", 1)).isEqualTo("one");
        assertThat(s.content("d", 2)).isEqualTo("two");
        assertThat(s.history("d")).containsExactly("1:ann", "2:bob", "3:ann");
    }

    @Test
    void shouldNotCreateAVersionForAnUnchangedEdit() {
        DocStore s = store();
        s.create("d", "ann", "same");

        assertThat(s.edit("d", "bob", 1, "same")).isEqualTo(1);

        assertThat(s.history("d")).hasSize(1);
    }

    @Test
    void shouldRestoreAnOldVersionAsANewVersion() {
        DocStore s = store();
        s.create("d", "ann", "one");
        s.edit("d", "ann", 1, "two");

        int restored = s.restore("d", "bob", 1);

        assertThat(restored).isEqualTo(3);
        assertThat(s.content("d", 3)).isEqualTo("one");
        assertThat(s.history("d")).containsExactly("1:ann", "2:ann", "3:bob");
        assertThat(s.restore("d", "bob", 3)).isEqualTo(3); // already the latest content
    }

    @Test
    void shouldDiffLineByLine() {
        DocStore s = store();
        s.create("d", "ann", "a\nb\nc\nd");
        s.edit("d", "ann", 1, "a\nB\nc\nd\ne");

        assertThat(s.diff("d", 1, 2)).containsExactly("  a", "- b", "+ B", "  c", "  d", "+ e");
        assertThat(s.diff("d", 2, 1)).containsExactly("  a", "- B", "+ b", "  c", "  d", "- e");
        assertThat(s.diff("d", 1, 1)).containsExactly("  a", "  b", "  c", "  d");
    }

    @Test
    void shouldMergeStaleEditsThatTouchDifferentLines() {
        DocStore s = store();
        s.create("d", "ann", FIVE);
        s.edit("d", "ann", 1, "L1\nl2\nl3\nl4\nl5");

        int merged = s.edit("d", "bob", 1, "l1\nl2\nl3\nl4\nL5");

        assertThat(merged).isEqualTo(3);
        assertThat(s.content("d", 3)).isEqualTo("L1\nl2\nl3\nl4\nL5");
        assertThat(s.history("d")).containsExactly("1:ann", "2:ann", "3:bob");
    }

    @Test
    void shouldMergeInsertionsAndDeletionsAtDifferentPlaces() {
        DocStore s = store();
        s.create("d", "ann", FIVE);
        s.edit("d", "ann", 1, "l1\nnew\nl2\nl3\nl4\nl5"); // inserts before l2

        s.edit("d", "bob", 1, "l1\nl2\nl3\nl5"); // deletes l4

        assertThat(s.content("d", s.latest("d"))).isEqualTo("l1\nnew\nl2\nl3\nl5");
    }

    @Test
    void shouldMergeAdjacentChangesWithoutConflict() {
        DocStore s = store();
        s.create("d", "ann", FIVE);
        s.edit("d", "ann", 1, "l1\nl2\nX\nl4\nl5"); // changes l3

        s.edit("d", "bob", 1, "l1\nY\nl3\nl4\nl5"); // changes l2 (adjacent, different line)

        assertThat(s.content("d", s.latest("d"))).isEqualTo("l1\nY\nX\nl4\nl5");
    }

    @Test
    void shouldReportAConflictWhenBothChangeTheSameLine() {
        DocStore s = store();
        s.create("d", "ann", FIVE);
        s.edit("d", "ann", 1, "l1\nl2\nANN\nl4\nl5");

        assertThatThrownBy(() -> s.edit("d", "bob", 1, "l1\nl2\nBOB\nl4\nl5"))
                .isInstanceOf(MergeConflictException.class);

        assertThat(s.latest("d")).isEqualTo(2);
        assertThat(s.content("d", 2)).isEqualTo("l1\nl2\nANN\nl4\nl5");
    }

    @Test
    void shouldReportAConflictForTwoInsertionsAtTheSamePlace() {
        DocStore s = store();
        s.create("d", "ann", "a\nb");
        s.edit("d", "ann", 1, "a\nx\nb");

        assertThatThrownBy(() -> s.edit("d", "bob", 1, "a\ny\nb")).isInstanceOf(MergeConflictException.class);
    }

    @Test
    void shouldAcceptTheSameChangeMadeTwice() {
        DocStore s = store();
        s.create("d", "ann", FIVE);
        s.edit("d", "ann", 1, "l1\nl2\nSAME\nl4\nl5");

        int v = s.edit("d", "bob", 1, "l1\nl2\nSAME\nl4\nl5");

        assertThat(v).isEqualTo(2);
        assertThat(s.history("d")).hasSize(2);
    }

    @Test
    void shouldMergeConcurrentEditsOfDifferentLinesWithoutLosingAny() {
        DocStore s = store();
        s.create("d", "root", "0\n1\n2\n3\n4\n5\n6\n7");

        Concurrently.run(8, t -> {
            String[] lines = "0\n1\n2\n3\n4\n5\n6\n7".split("\n");
            lines[t] = "edited-" + t;
            s.edit("d", "user" + t, 1, String.join("\n", lines));
        });

        assertThat(s.latest("d")).isEqualTo(9);
        assertThat(s.content("d", 9)).isEqualTo("edited-0\nedited-1\nedited-2\nedited-3\nedited-4\nedited-5\nedited-6\nedited-7");
        assertThat(s.history("d")).hasSize(9);
    }

    @Test
    void shouldLetExactlyOneConcurrentEditOfTheSameLineWin() {
        DocStore s = store();
        s.create("d", "root", "a\nb\nc");

        List<Boolean> wins = Concurrently.collect(8, t -> {
            try {
                s.edit("d", "user" + t, 1, "changed-by-" + t + "\nb\nc");
                return true;
            } catch (MergeConflictException e) {
                return false;
            }
        });

        assertThat(wins.stream().filter(w -> w).count()).isEqualTo(1);
        assertThat(s.latest("d")).isEqualTo(2);
    }

    @Test
    void shouldKeepVersionsGaplessUnderConcurrentWritersAndReaders() {
        DocStore s = store();
        s.create("d", "root", "start");
        AtomicInteger conflicts = new AtomicInteger();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 50; i++) {
                int base = s.latest("d");
                try {
                    s.edit("d", "u" + t, base, "text-" + t + "-" + i);
                } catch (MergeConflictException e) {
                    conflicts.incrementAndGet();
                }
                s.content("d", s.latest("d")); // readers must always find the latest version
            }
        });

        int latest = s.latest("d");
        assertThat(s.history("d")).hasSize(latest);
        for (int v = 1; v <= latest; v++) {
            assertThat(s.history("d").get(v - 1)).startsWith(v + ":");
        }
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            DocStore s = store();
            s.create("d", "root", "0\n1\n2\n3");

            Concurrently.run(4, t -> {
                String[] lines = "0\n1\n2\n3".split("\n");
                lines[t] = "e" + t;
                s.edit("d", "u" + t, 1, String.join("\n", lines));
            });

            assertThat(s.content("d", s.latest("d"))).as("run %d", run).isEqualTo("e0\ne1\ne2\ne3");
        }
    }
}
