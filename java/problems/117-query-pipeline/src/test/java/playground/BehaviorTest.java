package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.QueryPipeline;
import contract.QueryPipeline.Result;
import contract.QueryPipeline.Step;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.UnaryOperator;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static final List<String> DICTIONARY = List.of(
            "phone", "case", "cheap", "charger", "cable", "wireless", "usb", "deals", "laptop", "car", "cat");
    private static final Map<String, List<String>> SYNONYMS = Map.of(
            "phone", List.of("mobile", "cell"),
            "cheap", List.of("budget"),
            "laptop", List.of("notebook", "laptop", "notebook"),
            "mobile", List.of("handset"));
    private static final List<String> BANNED = List.of("free", "spam");
    private static final Map<String, String> CORPUS = Map.of(
            "d1", "cheap phone case",
            "d2", "wireless phone charger",
            "d3", "usb charger cable",
            "d4", "Budget mobile deals",
            "d5", "laptop usb cable");

    private static QueryPipeline pipeline() {
        return Candidate.of(() -> Solution.create(DICTIONARY, SYNONYMS, BANNED, CORPUS));
    }

    /** A test-defined step. */
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

    private static Step appending(String name, String word) {
        return step(name, q -> q + " " + word);
    }

    @Test
    void shouldTrimCollapseAndLowerCaseInParse() {
        Result r = pipeline().process("  CASE \t  Charger  ");

        assertThat(r.query()).isEqualTo("case charger");
        assertThat(r.trace()).startsWith("parse");
    }

    @Test
    void shouldNotTraceParseWhenTheQueryIsAlreadyClean() {
        Result r = pipeline().process("usb");

        assertThat(r.query()).isEqualTo("usb");
        assertThat(r.trace()).doesNotContain("parse");
    }

    @Test
    void shouldRejectNullAndBlankQueries() {
        QueryPipeline p = pipeline();

        assertThatThrownBy(() -> p.process(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.process("")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.process(" \t ")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldCorrectWordsWithinOneEdit() {
        QueryPipeline p = pipeline();

        assertThat(p.process("chargr").query()).isEqualTo("charger"); // insertion
        assertThat(p.process("usbb").query()).isEqualTo("usb"); // deletion
        assertThat(p.process("wirelass").query()).isEqualTo("wireless"); // substitution
        assertThat(p.process("chargr").trace()).contains("spell");
    }

    @Test
    void shouldLeaveDictionaryWordsAndFarWordsAlone() {
        QueryPipeline p = pipeline();

        Result inDict = p.process("car"); // one edit from "cat" but it is a dictionary word itself
        assertThat(inDict.query()).isEqualTo("car");
        assertThat(inDict.trace()).doesNotContain("spell");
        assertThat(p.process("zzzz").query()).isEqualTo("zzzz");
        assertThat(p.process("cabel").query()).isEqualTo("cabel"); // a swap is two edits
    }

    @Test
    void shouldLeaveAmbiguousMisspellingsUnchanged() {
        Result r = pipeline().process("cap"); // one edit from both "car" and "cat"

        assertThat(r.query()).isEqualTo("cap");
        assertThat(r.trace()).doesNotContain("spell");
    }

    @Test
    void shouldExpandSynonymsIntoAnOrGroupKeepingTheOriginalFirst() {
        QueryPipeline p = pipeline();

        assertThat(p.process("phone").query()).isEqualTo("(phone OR mobile OR cell)");
        assertThat(p.process("cheap case").query()).isEqualTo("(cheap OR budget) case");
        assertThat(p.process("phone").trace()).contains("synonyms");
    }

    @Test
    void shouldDropDuplicateSynonymsAndNotExpandSynonymsOfSynonyms() {
        QueryPipeline p = pipeline();

        assertThat(p.process("laptop").query()).isEqualTo("(laptop OR notebook)");
        assertThat(p.process("phone").query()).doesNotContain("handset");
        assertThat(p.process("mobile").query()).isEqualTo("(mobile OR handset)");
    }

    @Test
    void shouldCorrectSpellingBeforeExpandingSynonyms() {
        assertThat(pipeline().process("PHON").query()).isEqualTo("(phone OR mobile OR cell)");
    }

    @Test
    void shouldRemoveBannedWordsInTheFilterStep() {
        Result r = pipeline().process("free case spam");

        assertThat(r.query()).isEqualTo("case");
        assertThat(r.trace()).contains("filter");
    }

    @Test
    void shouldReturnAnEmptyResultWhenEverythingIsFiltered() {
        Result r = pipeline().process("Free  SPAM");

        assertThat(r.query()).isEmpty();
        assertThat(r.results()).isEmpty();
        assertThat(r.trace()).containsExactly("parse", "filter");
    }

    @Test
    void shouldRankByDistinctTermsMatchedThenByDocId() {
        Result r = pipeline().process("phone charger");

        assertThat(r.results()).containsExactly(
                "d2: wireless [phone] [charger]",
                "d1: cheap [phone] case",
                "d3: usb [charger] cable",
                "d4: Budget [mobile] deals");
    }

    @Test
    void shouldCountARepeatedTermOnlyOnce() {
        Result r = pipeline().process("cable cable charger");

        assertThat(r.results()).containsExactly(
                "d3: usb [charger] [cable]",
                "d2: wireless phone [charger]",
                "d5: laptop usb [cable]");
    }

    @Test
    void shouldCountAnOrGroupAsOneTermMatchedByAnyAlternative() {
        Result r = pipeline().process("cheap");

        assertThat(r.results()).containsExactly("d1: [cheap] phone case", "d4: [Budget] mobile deals");
    }

    @Test
    void shouldReturnNoResultsAndNoRankTraceWhenNothingMatches() {
        Result r = pipeline().process("zzzz");

        assertThat(r.results()).isEmpty();
        assertThat(r.trace()).doesNotContain("rank", "highlight");
    }

    @Test
    void shouldTraceEveryStepThatChangedSomethingInOrder() {
        Result r = pipeline().process("  Cheep   PHONE  ");

        assertThat(r.query()).isEqualTo("(cheap OR budget) (phone OR mobile OR cell)");
        assertThat(r.trace()).containsExactly("parse", "spell", "synonyms", "rank", "highlight");
        assertThat(r.results()).containsExactly(
                "d1: [cheap] [phone] case",
                "d4: [Budget] [mobile] deals",
                "d2: wireless [phone] charger");
    }

    @Test
    void shouldRunARegisteredStepAtTheEndOfTheQueryPhase() {
        QueryPipeline p = pipeline();
        p.register(appending("add-usb", "usb"));

        Result r = p.process("free case");

        assertThat(r.query()).isEqualTo("case usb");
        assertThat(r.trace()).containsExactly("filter", "add-usb", "rank", "highlight");
    }

    @Test
    void shouldRunAStepRegisteredBeforeOrAfterABuiltInAtThatPosition() {
        QueryPipeline p = pipeline();
        p.registerBefore("synonyms", step("alias", q -> q.replace("handset", "phone")));
        p.registerAfter("filter", step("trailing", q -> q + " case"));

        Result r = p.process("handset");

        assertThat(r.query()).isEqualTo("(phone OR mobile OR cell) case");
        assertThat(r.trace()).containsExactly("alias", "synonyms", "trailing", "rank", "highlight");
    }

    @Test
    void shouldHandTheRawQueryToAStepRegisteredBeforeParse() {
        QueryPipeline p = pipeline();
        List<String> seen = Collections.synchronizedList(new ArrayList<>());
        p.registerBefore("parse", step("spy", q -> {
            seen.add(q);
            return q;
        }));

        Result r = p.process("  Case ");

        assertThat(seen).containsExactly("  Case ");
        assertThat(r.query()).isEqualTo("case");
        assertThat(r.trace()).doesNotContain("spy");
    }

    @Test
    void shouldKeepSeparatePipelineInstancesIndependent() {
        QueryPipeline a = pipeline();
        QueryPipeline b = pipeline();
        a.register(appending("add-usb", "usb"));
        a.removeStep("spell");

        assertThat(b.process("chargr").query()).isEqualTo("charger");
        assertThat(a.process("chargr").query()).isEqualTo("chargr usb");
    }

    @Test
    void shouldNotRunARemovedStepAndShouldAllowReusingItsName() {
        QueryPipeline p = pipeline();
        p.removeStep("spell");
        assertThat(p.process("chargr").query()).isEqualTo("chargr");

        p.register(appending("tag", "usb"));
        p.removeStep("tag");
        assertThat(p.process("case").query()).isEqualTo("case");
        p.register(appending("tag", "cable"));
        assertThat(p.process("case").query()).isEqualTo("case cable");
    }

    @Test
    void shouldSkipADisabledStepUntilItIsEnabledAgain() {
        QueryPipeline p = pipeline();
        p.disableStep("synonyms");

        Result off = p.process("phone");
        assertThat(off.query()).isEqualTo("phone");
        assertThat(off.trace()).doesNotContain("synonyms");

        p.enableStep("synonyms");
        p.enableStep("synonyms");
        assertThat(p.process("phone").query()).isEqualTo("(phone OR mobile OR cell)");
    }

    @Test
    void shouldLeaveResultsEmptyWithoutRankAndUnhighlightedWithoutHighlight() {
        QueryPipeline p = pipeline();
        p.disableStep("highlight");
        Result plain = p.process("cheap");
        assertThat(plain.results()).containsExactly("d1: cheap phone case", "d4: Budget mobile deals");
        assertThat(plain.trace()).containsExactly("synonyms", "rank");

        p.removeStep("rank");
        Result none = p.process("cheap");
        assertThat(none.results()).isEmpty();
        assertThat(none.query()).isEqualTo("(cheap OR budget)");
    }

    @Test
    void shouldSkipAThrowingStepRecordItAsFailedAndContinue() {
        QueryPipeline p = pipeline();
        p.registerAfter("parse", step("boom", q -> {
            throw new IllegalStateException("kaput");
        }));
        p.registerAfter("boom", step("returns-null", q -> null));

        Result r = p.process("CHEEP");

        assertThat(r.query()).isEqualTo("(cheap OR budget)");
        assertThat(r.trace()).containsExactly(
                "parse", "boom:failed", "returns-null:failed", "spell", "synonyms", "rank", "highlight");
    }

    @Test
    void shouldStopTransformingAndReturnNothingWhenAStepEmptiesTheQuery() {
        QueryPipeline p = pipeline();
        p.registerAfter("parse", step("wipe", q -> ""));
        p.register(appending("never", "usb"));

        Result r = p.process("CASE");

        assertThat(r.query()).isEmpty();
        assertThat(r.results()).isEmpty();
        assertThat(r.trace()).containsExactly("parse", "wipe");
    }

    @Test
    void shouldRejectBadRegistrationsAndLeaveThePipelineUnchanged() {
        QueryPipeline p = pipeline();
        p.register(appending("tag", "usb"));

        assertThatThrownBy(() -> p.register(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.register(appending(" ", "x"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.register(appending("tag", "x"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.register(appending("spell", "x"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.registerBefore("nope", appending("a", "x"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.registerAfter("rank", appending("a", "x"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.registerBefore("highlight", appending("a", "x"))).isInstanceOf(IllegalArgumentException.class);

        assertThat(p.process("case").query()).isEqualTo("case usb");
    }

    @Test
    void shouldRejectUnknownStepNamesWhenRemovingDisablingOrEnabling() {
        QueryPipeline p = pipeline();

        assertThatThrownBy(() -> p.removeStep("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.disableStep("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.enableStep("nope")).isInstanceOf(IllegalArgumentException.class);
        p.removeStep("filter");
        assertThatThrownBy(() -> p.removeStep("filter")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.registerAfter("filter", appending("a", "x"))).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNotBeAffectedByLaterChangesToTheCallersConfiguration() {
        List<String> dictionary = new ArrayList<>(DICTIONARY);
        Map<String, String> corpus = new java.util.HashMap<>(CORPUS);
        QueryPipeline p = Candidate.of(() -> Solution.create(dictionary, SYNONYMS, BANNED, corpus));
        dictionary.clear();
        corpus.clear();

        assertThat(p.process("chargr").query()).isEqualTo("charger");
        assertThat(p.process("charger").results()).hasSize(2);
    }

    @Test
    void shouldReturnIndependentResultsPerCall() {
        QueryPipeline p = pipeline();
        Result first = p.process("cheap");
        List<String> snapshot = List.copyOf(first.results());

        p.process("usb");
        p.register(appending("tag", "usb"));
        p.process("case");

        assertThat(first.results()).isEqualTo(snapshot);
        assertThat(first.trace()).containsExactly("synonyms", "rank", "highlight");
    }

    @Test
    void shouldProcessConcurrentlyWhileStepsAreRegisteredAndRemoved() {
        QueryPipeline p = pipeline();
        AtomicInteger bad = new AtomicInteger();
        List<String> expected = p.process("cheep phone").results();

        Concurrently.run(9, 60_000, t -> {
            if (t == 8) {
                for (int i = 0; i < 300; i++) {
                    String n = "noop-" + i;
                    p.register(step(n, q -> q));
                    p.disableStep(n);
                    p.enableStep(n);
                    p.removeStep(n);
                }
                return;
            }
            for (int i = 0; i < 2_000; i++) {
                Result r = p.process("cheep phone");
                if (!r.results().equals(expected) || !r.query().equals("(cheap OR budget) (phone OR mobile OR cell)")) {
                    bad.incrementAndGet();
                }
            }
        });

        assertThat(bad).hasValue(0);
        assertThat(p.process("cheep phone").trace()).containsExactly("spell", "synonyms", "rank", "highlight");
    }
}
