package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.CommentService;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.TreeSet;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private CommentService service(String... docs) {
        return Candidate.of(() -> Solution.create(new HashSet<>(List.of(docs))));
    }

    private static void assertRejected(Runnable call) {
        assertThatThrownBy(call::run).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldReturnAnEmptyListForADocumentWithoutComments() {
        CommentService s = service("doc-1");

        assertThat(s.comments("doc-1")).isEmpty();
    }

    @Test
    void shouldReturnCommentIdsFromOneSequenceAcrossDocuments() {
        CommentService s = service("doc-1", "doc-2");

        assertThat(s.addComment("doc-1", "alice", "a")).isEqualTo("1");
        assertThat(s.addComment("doc-2", "bob", "b")).isEqualTo("2");
        assertThat(s.addComment("doc-1", "carol", "c")).isEqualTo("3");
    }

    @Test
    void shouldReturnCommentsInInsertionOrder() {
        CommentService s = service("doc-1");
        s.addComment("doc-1", "alice", "Fix the typo");
        s.addComment("doc-1", "bob", "Add a diagram");
        s.addComment("doc-1", "carol", "Looks good");

        assertThat(s.comments("doc-1"))
                .containsExactly("alice: Fix the typo", "bob: Add a diagram", "carol: Looks good");
    }

    @Test
    void shouldFormatEachCommentAsReviewerColonSpaceText() {
        CommentService s = service("doc-1");
        s.addComment("doc-1", "alice", "hello");

        assertThat(s.comments("doc-1")).containsExactly("alice: hello");
    }

    @Test
    void shouldKeepCommentsOfDifferentDocumentsSeparate() {
        CommentService s = service("doc-1", "doc-2");
        s.addComment("doc-1", "alice", "one");
        s.addComment("doc-2", "bob", "two");
        s.addComment("doc-1", "carol", "three");

        assertThat(s.comments("doc-1")).containsExactly("alice: one", "carol: three");
        assertThat(s.comments("doc-2")).containsExactly("bob: two");
    }

    @Test
    void shouldAllowTheSameReviewerToCommentRepeatedly() {
        CommentService s = service("doc-1");
        s.addComment("doc-1", "alice", "first");
        s.addComment("doc-1", "alice", "second");

        assertThat(s.comments("doc-1")).containsExactly("alice: first", "alice: second");
    }

    @Test
    void shouldKeepIdenticalCommentsAsSeparateEntries() {
        CommentService s = service("doc-1");
        String first = s.addComment("doc-1", "alice", "same");
        String second = s.addComment("doc-1", "alice", "same");

        assertThat(first).isNotEqualTo(second);
        assertThat(s.comments("doc-1")).containsExactly("alice: same", "alice: same");
    }

    @Test
    void shouldStoreTextVerbatim() {
        CommentService s = service("doc-1");
        s.addComment("doc-1", "alice", "  see: line 3\nand üé 中  ");

        assertThat(s.comments("doc-1")).containsExactly("alice:   see: line 3\nand üé 中  ");
    }

    @Test
    void shouldRejectAddingToAnUnknownDocument() {
        CommentService s = service("doc-1");

        assertRejected(() -> s.addComment("doc-9", "alice", "hi"));
    }

    @Test
    void shouldRejectReadingAnUnknownDocument() {
        CommentService s = service("doc-1");

        assertRejected(() -> s.comments("doc-9"));
    }

    @Test
    void shouldTreatDocumentIdsAsExactStrings() {
        CommentService s = service("doc-1");

        assertRejected(() -> s.comments("DOC-1"));
        assertRejected(() -> s.addComment(" doc-1", "alice", "hi"));
    }

    @Test
    void shouldRejectBlankReviewerAndBlankText() {
        CommentService s = service("doc-1");

        for (String blank : new String[] {null, "", " ", "\t\n"}) {
            assertRejected(() -> s.addComment("doc-1", blank, "text"));
            assertRejected(() -> s.addComment("doc-1", "alice", blank));
        }
        assertThat(s.comments("doc-1")).isEmpty();
    }

    @Test
    void shouldRejectBlankDocumentIds() {
        CommentService s = service("doc-1");

        for (String blank : new String[] {null, "", "  "}) {
            assertRejected(() -> s.addComment(blank, "alice", "hi"));
            assertRejected(() -> s.comments(blank));
        }
    }

    @Test
    void shouldNotConsumeAnIdOrLeaveATraceWhenACallIsRejected() {
        CommentService s = service("doc-1");

        assertRejected(() -> s.addComment("doc-9", "alice", "hi"));
        assertRejected(() -> s.addComment("doc-1", " ", "hi"));
        assertRejected(() -> s.addComment("doc-1", "alice", ""));

        assertThat(s.comments("doc-1")).isEmpty();
        assertThat(s.addComment("doc-1", "alice", "ok")).isEqualTo("1");
        assertRejected(() -> s.comments("doc-9"));
    }

    @Test
    void shouldReportBlankArgumentsEvenForAnUnknownDocument() {
        CommentService s = service("doc-1");

        assertRejected(() -> s.addComment("nope", " ", "x"));
        assertRejected(() -> s.addComment("nope", "alice", " "));
    }

    @Test
    void shouldNotChangeAListThatWasAlreadyReturned() {
        CommentService s = service("doc-1");
        s.addComment("doc-1", "alice", "first");
        List<String> before = s.comments("doc-1");

        s.addComment("doc-1", "bob", "second");

        assertThat(before).containsExactly("alice: first");
        assertThat(s.comments("doc-1")).containsExactly("alice: first", "bob: second");
    }

    @Test
    void shouldNotLetCallersChangeTheServiceThroughAReturnedList() {
        CommentService s = service("doc-1");
        s.addComment("doc-1", "alice", "first");
        List<String> returned = s.comments("doc-1");

        try {
            returned.clear();
            returned.add("mallory: injected");
        } catch (UnsupportedOperationException ignored) {
            // an immutable copy is fine too
        }

        assertThat(s.comments("doc-1")).containsExactly("alice: first");
    }

    @Test
    void shouldCopyTheDocumentSetPassedToCreate() {
        Set<String> docs = new HashSet<>(Set.of("doc-1"));
        CommentService s = Candidate.of(() -> Solution.create(docs));

        docs.add("doc-2");
        docs.remove("doc-1");

        assertThat(s.addComment("doc-1", "alice", "hi")).isEqualTo("1");
        assertRejected(() -> s.addComment("doc-2", "alice", "hi"));
    }

    @Test
    void shouldRefuseEverythingWhenCreatedWithoutDocuments() {
        CommentService s = Candidate.of(() -> Solution.create(Set.of()));

        assertRejected(() -> s.addComment("doc-1", "alice", "hi"));
        assertRejected(() -> s.comments("doc-1"));
    }

    @Test
    void shouldRejectANullOrBlankDocumentIdWhenCreating() {
        Candidate.of(() -> Solution.create(Set.of("doc-1")));
        Set<String> withNull = new HashSet<>();
        withNull.add(null);

        assertRejected(() -> Solution.create(null));
        assertRejected(() -> Solution.create(withNull));
        assertRejected(() -> Solution.create(Set.of("doc-1", " ")));
    }

    @Test
    void shouldLoseNoCommentWhenManyThreadsAddToOneDocument() {
        CommentService s = service("doc-1");
        int threads = 16;
        int perThread = 100;

        List<List<String>> ids = Concurrently.collect(threads, t -> {
            List<String> mine = new ArrayList<>();
            for (int i = 0; i < perThread; i++) {
                mine.add(s.addComment("doc-1", "r" + t, "c" + i));
            }
            return mine;
        });

        Set<String> all = new TreeSet<>();
        ids.forEach(all::addAll);
        assertThat(all).hasSize(threads * perThread);
        for (int n = 1; n <= threads * perThread; n++) {
            assertThat(all).contains(Integer.toString(n));
        }
        assertThat(s.comments("doc-1")).hasSize(threads * perThread).doesNotHaveDuplicates();
    }

    @Test
    void shouldListConcurrentCommentsInIdOrderAndKeepEachThreadsOwnOrder() {
        CommentService s = service("doc-1");
        int threads = 8;
        int perThread = 200;

        List<List<String>> ids = Concurrently.collect(threads, t -> {
            List<String> mine = new ArrayList<>();
            for (int i = 0; i < perThread; i++) {
                mine.add(s.addComment("doc-1", "r" + t, "c" + i));
            }
            return mine;
        });

        // expected listing: sort what each call returned by its numeric id
        String[] byId = new String[threads * perThread + 1];
        for (int t = 0; t < threads; t++) {
            for (int i = 0; i < perThread; i++) {
                byId[Integer.parseInt(ids.get(t).get(i))] = "r" + t + ": c" + i;
            }
        }
        List<String> expected = new ArrayList<>(java.util.Arrays.asList(byId).subList(1, byId.length));
        assertThat(s.comments("doc-1")).containsExactlyElementsOf(expected);
    }

    @Test
    void shouldKeepDocumentsIndependentUnderConcurrentWrites() {
        CommentService s = service("a", "b", "c", "d");
        String[] docs = {"a", "b", "c", "d"};

        Concurrently.run(8, t -> {
            for (int i = 0; i < 100; i++) {
                s.addComment(docs[t % 4], "r" + t, "c" + i);
            }
        });

        for (String d : docs) {
            assertThat(s.comments(d)).hasSize(200);
        }
        assertThat(s.addComment("a", "x", "y")).isEqualTo("801");
    }

    @Test
    void shouldAllowReadersWhileWritersAdd() {
        CommentService s = service("doc-1");

        Concurrently.run(8, t -> {
            if (t < 4) {
                for (int i = 0; i < 250; i++) {
                    s.addComment("doc-1", "w" + t, "c" + i);
                }
            } else {
                for (int i = 0; i < 250; i++) {
                    List<String> seen = s.comments("doc-1");
                    assertThat(seen).doesNotContainNull().doesNotHaveDuplicates();
                }
            }
        });

        assertThat(s.comments("doc-1")).hasSize(1_000);
    }
}
