package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.QueryPipeline;
import contract.QueryPipeline.Result;
import contract.QueryPipeline.Step;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.UnaryOperator;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static final List<String> DICTIONARY = List.of("phone", "case", "cheap", "charger", "usb", "car", "cat");
    private static final Map<String, List<String>> SYNONYMS = Map.of("phone", List.of("mobile", "cell"));
    private static final Map<String, String> CORPUS = Map.of(
            "d10", "usb usb cable",
            "d2", "usb cable",
            "d1", "Phone CASE",
            "d3", "mobile phone");

    private static QueryPipeline pipeline(Collection<String> banned) {
        return Candidate.of(() -> Solution.create(DICTIONARY, SYNONYMS, banned, CORPUS));
    }

    private static Step step(String name, UnaryOperator<String> f) {
        return new Step() {
            @Override
            public String name() {
                return name;
            }

            @Override
            public String transform(String query) {
                return f.apply(query);
            }
        };
    }

    @Test
    void shouldFilterBannedWordsInsideSynonymGroupsAndCollapseThem() {
        assertThat(pipeline(List.of("cell")).process("phone").query()).isEqualTo("(phone OR mobile)");
        assertThat(pipeline(List.of("mobile", "cell")).process("phone").query()).isEqualTo("phone");
        assertThat(pipeline(List.of("phone")).process("phone").query()).isEqualTo("(mobile OR cell)");
        assertThat(pipeline(List.of("phone", "mobile", "cell")).process("phone case").query()).isEqualTo("case");
    }

    @Test
    void shouldBreakScoreTiesByDocIdAsText() {
        Result r = pipeline(List.of()).process("usb");

        assertThat(r.results()).containsExactly("d10: [usb] [usb] cable", "d2: [usb] cable");
    }

    @Test
    void shouldHighlightEveryOccurrenceKeepingTheDocumentsOwnCase() {
        Result r = pipeline(List.of()).process("phone case");

        assertThat(r.results()).containsExactly("d1: [Phone] [CASE]", "d3: [mobile] [phone]");
    }

    @Test
    void shouldNotCorrectACorrectionAgainOrTouchWordsThatAreDictionaryWords() {
        QueryPipeline p = pipeline(List.of());

        assertThat(p.process("phon").query()).isEqualTo("(phone OR mobile OR cell)");
        assertThat(p.process("car cat cap").query()).isEqualTo("car cat cap");
        assertThat(p.process("car cat cap").trace()).doesNotContain("spell");
    }

    @Test
    void shouldPlaceStepsExactlyWhereTheyWereRegistered() {
        QueryPipeline p = pipeline(List.of());
        p.registerAfter("parse", step("a", q -> q + " a"));
        p.registerAfter("parse", step("b", q -> q + " b"));
        p.register(step("c", q -> q + " c"));
        p.register(step("d", q -> q + " d"));
        p.registerBefore("filter", step("e", q -> q + " e"));
        p.registerAfter("synonyms", step("f", q -> q + " f"));
        p.registerBefore("a", step("g", q -> q + " g"));

        Result r = p.process("zzzz");

        assertThat(r.query()).isEqualTo("zzzz b g a f e c d");
        assertThat(r.trace()).containsExactly("b", "g", "a", "f", "e", "c", "d");
    }

    @Test
    void shouldFilterWhatEarlierStepsAddButNotWhatLaterStepsAdd() {
        QueryPipeline p = pipeline(List.of("spam"));
        p.registerBefore("filter", step("early", q -> q + " spam"));
        p.register(step("late", q -> q + " spam"));

        Result r = p.process("usb");

        assertThat(r.query()).isEqualTo("usb spam");
        assertThat(r.trace()).containsExactly("early", "filter", "late", "rank", "highlight");
    }

    @Test
    void shouldKeepGoingAfterAStepFailsAndLeaveTheQueryAsItWas() {
        QueryPipeline p = pipeline(List.of());
        List<String> seen = new ArrayList<>();
        p.registerAfter("synonyms", step("explode", q -> {
            seen.add(q);
            throw new UnsupportedOperationException();
        }));
        p.register(step("after", q -> {
            seen.add(q);
            return q;
        }));

        Result r = p.process("phon");

        assertThat(seen).containsExactly("(phone OR mobile OR cell)", "(phone OR mobile OR cell)");
        assertThat(r.trace()).containsExactly("spell", "synonyms", "explode:failed", "rank", "highlight");
        assertThat(r.query()).isEqualTo("(phone OR mobile OR cell)");
    }

    @Test
    void shouldTreatEveryStepAsOptionalIncludingParse() {
        QueryPipeline p = pipeline(List.of());
        p.removeStep("parse");
        p.disableStep("spell");

        Result r = p.process("USB");

        assertThat(r.query()).isEqualTo("USB");
        assertThat(r.trace()).doesNotContain("parse", "spell");
        assertThat(p.process("usb").results()).hasSize(2);
    }

    @Test
    void shouldGiveEveryConcurrentCallAConsistentStepOrderWhileStepsChange() {
        QueryPipeline p = pipeline(List.of("spam"));
        AtomicInteger bad = new AtomicInteger();

        Concurrently.run(10, 60_000, t -> {
            if (t >= 8) {
                for (int i = 0; i < 400; i++) {
                    String n = "t" + t + "-" + i;
                    p.registerBefore("filter", step(n, q -> q + " spam"));
                    p.disableStep(n);
                    p.enableStep(n);
                    p.removeStep(n);
                }
                return;
            }
            for (int i = 0; i < 3_000; i++) {
                Result r = p.process("  USB ");
                List<String> trace = r.trace();
                int parse = trace.indexOf("parse");
                int rank = trace.indexOf("rank");
                // whatever mix of extra steps ran, the built-in order and the end result are intact
                if (parse != 0 || rank < 0 || trace.indexOf("highlight") != rank + 1 || !r.query().startsWith("usb")
                        || r.query().contains("spam")) {
                    bad.incrementAndGet();
                }
            }
        });

        assertThat(bad).hasValue(0);
    }
}
