package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.CommentService;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.TreeSet;
import java.util.concurrent.atomic.AtomicBoolean;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private CommentService service(String... docs) {
        return Candidate.of(() -> Solution.create(new HashSet<>(List.of(docs))));
    }

    @Test
    void shouldGiveReadersAPrefixOfTheFinalHistoryWhileWritersRun() {
        CommentService s = service("doc-1");
        int writes = 5_000;
        AtomicBoolean writerDone = new AtomicBoolean();

        List<List<List<String>>> snapshots = Concurrently.collect(5, 60_000, t -> {
            List<List<String>> seen = new ArrayList<>();
            if (t == 0) {
                for (int i = 0; i < writes; i++) {
                    s.addComment("doc-1", "w", "c" + i);
                }
                writerDone.set(true);
            } else {
                while (!writerDone.get()) {
                    seen.add(s.comments("doc-1"));
                }
            }
            return seen;
        });

        List<String> finalList = s.comments("doc-1");
        assertThat(finalList).hasSize(writes);
        for (List<List<String>> perReader : snapshots) {
            for (List<String> snap : perReader) {
                assertThat(snap).isEqualTo(finalList.subList(0, snap.size()));
            }
        }
    }

    @Test
    void shouldLeaveNoHolesInTheIdSequenceWhenRejectedAddsRaceWithGoodOnes() {
        CommentService s = service("doc-1", "doc-2");

        List<List<String>> accepted = Concurrently.collect(8, 60_000, t -> {
            List<String> mine = new ArrayList<>();
            for (int i = 0; i < 300; i++) {
                switch (i % 4) {
                    case 0 -> mine.add(s.addComment("doc-1", "r" + t, "ok" + i));
                    case 1 -> {
                        assertThatThrownBy(() -> s.addComment("nope", "r", "x"))
                                .isInstanceOf(IllegalArgumentException.class);
                    }
                    case 2 -> mine.add(s.addComment("doc-2", "r" + t, "ok" + i));
                    default -> {
                        assertThatThrownBy(() -> s.addComment("doc-1", " ", "x"))
                                .isInstanceOf(IllegalArgumentException.class);
                    }
                }
            }
            return mine;
        });

        Set<Integer> ids = new TreeSet<>();
        accepted.forEach(l -> l.forEach(id -> ids.add(Integer.parseInt(id))));
        assertThat(ids).hasSize(8 * 150);
        assertThat(ids.iterator().next()).isEqualTo(1);
        assertThat(ids).last().isEqualTo(8 * 150);
        assertThat(s.comments("doc-1").size() + s.comments("doc-2").size()).isEqualTo(8 * 150);
    }

    @Test
    void shouldNotChangeASnapshotTakenBeforeConcurrentAdds() {
        CommentService s = service("doc-1");
        s.addComment("doc-1", "alice", "seed");
        List<String> snapshot = s.comments("doc-1");

        Concurrently.run(8, t -> {
            for (int i = 0; i < 200; i++) {
                s.addComment("doc-1", "w" + t, "c" + i);
            }
        });

        assertThat(snapshot).containsExactly("alice: seed");
        assertThat(s.comments("doc-1")).hasSize(1_601).startsWith("alice: seed");
    }

    @Test
    void shouldKeepEachThreadsOwnOrderAcrossManyDocuments() {
        String[] docs = new String[50];
        for (int i = 0; i < docs.length; i++) {
            docs[i] = "doc-" + i;
        }
        CommentService s = service(docs);

        Concurrently.run(8, 60_000, t -> {
            for (int i = 0; i < 500; i++) {
                s.addComment(docs[(t * 7 + i) % docs.length], "r" + t, Integer.toString(i));
            }
        });

        int total = 0;
        for (String d : docs) {
            List<String> list = s.comments(d);
            total += list.size();
            for (int t = 0; t < 8; t++) {
                int last = -1;
                for (String c : list) {
                    if (c.startsWith("r" + t + ": ")) {
                        int n = Integer.parseInt(c.substring(("r" + t + ": ").length()));
                        assertThat(n).isGreaterThan(last);
                        last = n;
                    }
                }
            }
        }
        assertThat(total).isEqualTo(8 * 500);
    }

    @Test
    void shouldHandleAHugeHistoryInOneDocument() {
        CommentService s = service("doc-1", "doc-2");
        for (int i = 0; i < 50_000; i++) {
            s.addComment("doc-1", "r" + (i % 10), "c" + i);
        }

        List<String> list = s.comments("doc-1");
        assertThat(list).hasSize(50_000);
        assertThat(list.get(0)).isEqualTo("r0: c0");
        assertThat(list.get(49_999)).isEqualTo("r9: c49999");
        assertThat(s.comments("doc-2")).isEmpty();
        assertThat(s.addComment("doc-2", "x", "y")).isEqualTo("50001");
    }

    @Test
    void shouldNotCreateADocumentByFailingToAddToIt() {
        CommentService s = service("doc-1");

        assertThatThrownBy(() -> s.addComment("ghost", "alice", "boo")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.comments("ghost")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addComment("ghost", "alice", "boo")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNotParseTheReviewerOrTheTextWhenTheyContainTheSeparator() {
        CommentService s = service("doc-1");
        s.addComment("doc-1", "a: b", "c: d");
        s.addComment("doc-1", "x", ": ");

        assertThat(s.comments("doc-1")).containsExactly("a: b: c: d", "x: : ");
    }
}
