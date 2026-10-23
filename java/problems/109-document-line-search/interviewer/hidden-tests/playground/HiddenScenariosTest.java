package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.DocumentSearch;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static DocumentSearch of(List<String> lines) {
        return Candidate.of(() -> Solution.create(lines));
    }

    private static List<String> big(int n) {
        List<String> lines = new ArrayList<>(n);
        for (int i = 1; i <= n; i++) {
            StringBuilder sb = new StringBuilder("row").append(i).append(" common shared");
            sb.append(" bucket").append(i % 50);
            if (i % 10_000 == 0) {
                sb.append(" rare");
            }
            lines.add(sb.toString());
        }
        return lines;
    }

    @Test
    void shouldReturnSortedIdsWhenARareWordIsIntersectedWithACommonOne() {
        DocumentSearch d = of(big(100_000));

        assertThat(d.search("common rare")).containsExactly(10_000, 20_000, 30_000, 40_000, 50_000, 60_000, 70_000, 80_000, 90_000, 100_000);
        assertThat(d.search("rare common shared")).hasSize(10).isSorted();
        assertThat(d.search("rare nosuchword common")).isEmpty();
    }

    @Test
    void shouldAnswerFiveThousandSearchesOnAHundredThousandLines() {
        DocumentSearch d = of(big(100_000));

        long start = System.nanoTime();
        long total = 0;
        for (int i = 0; i < 5_000; i++) {
            int row = 1 + (i * 37) % 100_000;
            total += d.search("bucket" + (row % 50) + " row" + row + " common").size();
        }
        long elapsedMs = (System.nanoTime() - start) / 1_000_000;

        assertThat(total).isGreaterThan(0);
        assertThat(elapsedMs).as("5,000 searches over 100,000 lines").isLessThan(15_000);
    }

    @Test
    void shouldKeepIdsStableWhileDeletingAndSearchingInterleaved() {
        DocumentSearch d = of(big(5_000));

        int live = 5_000;
        for (int id = 1; id <= 5_000; id += 3) {
            assertThat(d.deleteLine(id)).isTrue();
            live--;
            if (id % 300 == 1) {
                assertThat(d.search("common")).hasSize(live).isSorted().doesNotContain(id);
            }
        }
        assertThat(d.search("row2")).containsExactly(2);
        assertThat(d.search("row1")).isEmpty();
        assertThat(d.search("row5000")).containsExactly(5_000);
        assertThat(d.deleteLine(5_001)).isFalse();
    }

    @Test
    void shouldReturnNothingAfterEveryLineIsDeleted() {
        DocumentSearch d = of(List.of("a b", "b c", "c a"));

        assertThat(d.deleteLine(3)).isTrue();
        assertThat(d.deleteLine(1)).isTrue();
        assertThat(d.deleteLine(2)).isTrue();

        assertThat(d.search("a")).isEmpty();
        assertThat(d.search("b c")).isEmpty();
        assertThat(d.deleteLine(1)).isFalse();
        assertThat(d.deleteLine(2)).isFalse();
        assertThat(d.deleteLine(3)).isFalse();
        assertThat(d.deleteLine(4)).isFalse();
    }

    @Test
    void shouldFoldCaseOfNonAsciiLettersAndSplitOnAnyWhitespace() {
        DocumentSearch d = of(List.of("ÉCOLE normale", "café\tcrème\nbrûlée", "Straße 7", "tab separated"));

        assertThat(d.search("école")).containsExactly(1);
        assertThat(d.search("ÉCOLE")).containsExactly(1);
        assertThat(d.search("CAFÉ BRÛLÉE")).containsExactly(2);
        assertThat(d.search("crème")).containsExactly(2);
        assertThat(d.search("tab separated")).containsExactly(4);
        assertThat(d.search("7")).containsExactly(3);
    }

    @Test
    void shouldHandleAVeryLongLineAndAQueryWithManyWords() {
        StringBuilder longLine = new StringBuilder();
        StringBuilder query = new StringBuilder();
        for (int i = 0; i < 50_000; i++) {
            longLine.append("w").append(i).append(i % 2 == 0 ? ", " : " - ");
        }
        for (int i = 0; i < 200; i++) {
            query.append(" w").append(i * 7);
        }
        DocumentSearch d = of(List.of("w1 w2", longLine.toString(), "w7 w14"));

        assertThat(d.search(query.toString())).containsExactly(2);
        assertThat(d.search(query + " w50000")).isEmpty();
        assertThat(d.search("w7 w14")).containsExactly(2, 3);
    }

    @Test
    void shouldTellExactlyOneThreadForEveryRacedDeletion() {
        int lines = 200;
        List<String> text = new ArrayList<>();
        for (int i = 0; i < lines; i++) {
            text.add("shared line" + i);
        }
        DocumentSearch d = of(text);
        AtomicInteger[] wins = new AtomicInteger[lines + 1];
        for (int i = 0; i <= lines; i++) {
            wins[i] = new AtomicInteger();
        }

        Concurrently.run(16, t -> {
            for (int id = 1; id <= lines; id++) {
                if (d.deleteLine(id)) {
                    wins[id].incrementAndGet();
                }
            }
        });

        for (int id = 1; id <= lines; id++) {
            assertThat(wins[id].get()).as("winners for id " + id).isEqualTo(1);
        }
        assertThat(d.search("shared")).isEmpty();
    }

    @Test
    void shouldNeverShowADeletedLineToASearchThatStartsAfterTheDeletionReturned() {
        DocumentSearch d = of(big(20_000));
        AtomicInteger deletedUpTo = new AtomicInteger(0);
        AtomicReference<String> problem = new AtomicReference<>();

        Concurrently.run(9, 60_000, t -> {
            if (t == 0) {
                for (int id = 1; id <= 20_000; id++) {
                    d.deleteLine(id);
                    deletedUpTo.set(id);
                }
                return;
            }
            for (int i = 0; i < 100; i++) {
                int floor = deletedUpTo.get();
                List<Integer> ids = d.search("shared bucket" + (i % 50));
                int prev = 0;
                for (int id : ids) {
                    if (id <= floor) {
                        problem.compareAndSet(null, "deleted id " + id + " visible, floor " + floor);
                    }
                    if (id <= prev) {
                        problem.compareAndSet(null, "not strictly increasing");
                    }
                    prev = id;
                }
            }
        });

        assertThat(problem.get()).isNull();
        assertThat(d.search("shared")).isEmpty();
    }
}
