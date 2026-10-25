package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.DictionaryApp;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private DictionaryApp dict() {
        return Candidate.of(Solution::create);
    }

    private static String name(int n) {
        StringBuilder b = new StringBuilder("w");
        do {
            b.append((char) ('a' + n % 26));
            n /= 26;
        } while (n > 0);
        return b.toString();
    }

    @Test
    void shouldCoverEveryKindOfEditDistanceOne() {
        DictionaryApp d = dict();
        for (String w : new String[] {"stone", "tone", "stoned", "store", "atone", "stove", "scone", "ston", "o'ne"}) {
            d.addWord(w, "noun", "x");
        }

        // insertion, deletion and substitution all count as one edit; a transposition or two edits does not
        assertThat(d.suggest("stne", 20)).containsExactly("stone");
        assertThat(d.suggest("stoe", 20)).containsExactly("ston", "stone", "store", "stove");
        assertThat(d.suggest("tone", 20)).isEmpty();
        assertThat(d.suggest("one", 20)).containsExactly("o'ne", "tone");
        // distance 2 is not a suggestion
        assertThat(d.suggest("stonee", 20)).containsExactly("stone", "stoned");
        assertThat(d.suggest("sstonee", 20)).isEmpty();
    }

    @Test
    void shouldReturnTheFirstKSuggestionsInAlphabeticalOrderNotInsertionOrder() {
        DictionaryApp d = dict();
        for (String w : new String[] {"bat", "cat", "aat", "eat", "dat"}) {
            d.addWord(w, "noun", "x");
        }

        assertThat(d.suggest("xat", 3)).containsExactly("aat", "bat", "cat");
        assertThat(d.suggest("at", 2)).containsExactly("aat", "bat");
    }

    @Test
    void shouldFollowTheWordOfTheDayWhenWordsComeAndGo() {
        DictionaryApp d = dict();
        for (String w : new String[] {"delta", "alpha", "charlie", "bravo"}) {
            d.addWord(w, "noun", "x");
        }
        assertThat(d.wordOfTheDay(Long.MAX_VALUE)).contains("delta"); // MAX mod 4 = 3
        assertThat(d.wordOfTheDay(Long.MIN_VALUE)).contains("alpha"); // floorMod(MIN, 4) = 0
        assertThat(d.wordOfTheDay(-3)).contains("bravo");

        d.removeWord("alpha");
        assertThat(d.wordOfTheDay(0)).contains("bravo");
        assertThat(d.wordOfTheDay(3)).contains("bravo");
        assertThat(d.wordOfTheDay(Long.MIN_VALUE)).contains("charlie"); // floorMod(MIN, 3) = 1

        d.addWord("aardvark", "noun", "x");
        assertThat(d.wordOfTheDay(0)).contains("aardvark");
        d.removeWord("aardvark");
        d.removeWord("bravo");
        d.removeWord("charlie");
        d.removeWord("delta");
        assertThat(d.wordOfTheDay(0)).isEmpty();
    }

    @Test
    void shouldKeepUpdateInPlaceAndRejectOnlyRealDuplicates() {
        DictionaryApp d = dict();
        d.addWord("set", "noun", "a group");
        d.addWord("set", "noun", "a tennis unit");
        d.addWord("set", "verb", "to put down");

        d.updateMeaning("set", 0, "a collection");
        d.updateMeaning("set", 2, "a group");

        assertThat(d.lookup("set")).containsExactly("NOUN: a collection", "NOUN: a tennis unit", "VERB: a group");
        // the old text is free again
        assertThat(d.addWord("set", "noun", "a group")).isTrue();
        // an update must not count as an add: still ignored when equal to an existing pair
        assertThat(d.addWord("set", "noun", "a collection")).isFalse();
        assertThat(d.meaningCount()).isEqualTo(4);
    }

    @Test
    void shouldSearchAcrossAllMeaningsWithoutMatchingAcrossMeaningBoundaries() {
        DictionaryApp d = dict();
        d.addWord("one", "noun", "alpha");
        d.addWord("one", "noun", "beta");
        d.addWord("two", "noun", "ALPHABET soup");
        d.addWord("three", "adverb", "Gamma ray");

        assertThat(d.search("alpha", 10)).containsExactly("one", "two");
        assertThat(d.search("alphabeta", 10)).isEmpty();
        assertThat(d.search("alpha beta", 10)).isEmpty();
        assertThat(d.search("ray", 10)).containsExactly("three");
        assertThat(d.search("gamma ray", 10)).containsExactly("three");
        assertThat(d.search("a", 2)).containsExactly("one", "three");
    }

    @Test
    void shouldStayConsistentWhenMeaningsAreRemovedAndTheWordDisappears() {
        DictionaryApp d = dict();
        d.addWord("a", "noun", "x");
        d.addWord("b", "noun", "y");
        d.addWord("b", "verb", "z");
        d.addSynonym("a", "b");

        d.removeMeaning("b", 0);
        assertThat(d.synonyms("a")).containsExactly("b");
        d.removeMeaning("b", 0);

        assertThat(d.size()).isEqualTo(1);
        assertThat(d.meaningCount()).isEqualTo(1);
        assertThat(d.synonyms("a")).isEmpty();
        assertThat(d.prefix("b", 5)).isEmpty();
        assertThatThrownBy(() -> d.addSynonym("a", "b")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectASynonymLinkToARemovedWord() {
        DictionaryApp d = dict();
        d.addWord("a", "noun", "x");
        d.addWord("b", "noun", "y");
        d.removeWord("b");

        assertThatThrownBy(() -> d.addSynonym("a", "b")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldBeAbleToLinkTheSameTwoWordsInOppositeDirectionsConcurrentlyWithoutDeadlock() {
        DictionaryApp d = dict();
        for (int i = 0; i < 20; i++) {
            d.addWord(name(i), "noun", "x");
        }

        Concurrently.run(8, 20_000, t -> {
            for (int round = 0; round < 500; round++) {
                for (int i = 0; i < 20; i++) {
                    int j = (i + 1 + t) % 20;
                    if (i == j) {
                        continue;
                    }
                    if (t % 2 == 0) {
                        d.addSynonym(name(i), name(j));
                    } else {
                        d.addSynonym(name(j), name(i));
                    }
                }
            }
        });

        for (int i = 0; i < 20; i++) {
            for (String s : d.synonyms(name(i))) {
                assertThat(d.synonyms(s)).contains(name(i));
            }
        }
    }

    @Test
    void shouldNeverShowAHalfAppliedWordRemovalToConcurrentReaders() {
        DictionaryApp d = dict();
        d.addWord("hub", "noun", "x");
        d.addWord("spoke", "noun", "x");

        Concurrently.run(5, 30_000, t -> {
            if (t == 0) {
                for (int i = 0; i < 3_000; i++) {
                    d.removeWord("spoke");
                    d.addWord("spoke", "noun", "x");
                    d.addSynonym("hub", "spoke");
                }
                return;
            }
            for (int i = 0; i < 6_000; i++) {
                List<String> hubSyn = d.synonyms("hub");
                // synonyms never contain a word that is not a word: every listed name looks up to something
                // at the moment of the call, or was removed after it; links to *unknown* names must not persist
                assertThat(hubSyn.size()).isLessThanOrEqualTo(1);
                assertThat(d.size()).isBetween(1, 2);
            }
        });

        assertThat(d.synonyms("hub")).containsExactly("spoke");
        assertThat(d.synonyms("spoke")).containsExactly("hub");
    }

    @Test
    void shouldKeepInsertionOrderVisibleToReadersWhileWritersAppend() {
        DictionaryApp d = dict();
        d.addWord("bank", "noun", "m0");

        Concurrently.run(5, 30_000, t -> {
            if (t == 0) {
                for (int i = 1; i < 2_000; i++) {
                    d.addWord("bank", "noun", "m" + i);
                }
                return;
            }
            for (int i = 0; i < 1_000; i++) {
                List<String> l = d.lookup("bank");
                for (int k = 0; k < l.size(); k++) {
                    assertThat(l.get(k)).isEqualTo("NOUN: m" + k);
                }
            }
        });

        assertThat(d.lookup("bank")).hasSize(2_000);
    }

    @Test
    void shouldCountMeaningsExactlyUnderConcurrentAddsAndWordRemovals() {
        DictionaryApp d = dict();

        Concurrently.run(8, 30_000, t -> {
            for (int i = 0; i < 300; i++) {
                String w = name(i % 30);
                if (t % 4 == 3) {
                    d.removeWord(w);
                } else {
                    d.addWord(w, "noun", "t" + t + "i" + i);
                }
            }
        });

        int total = 0;
        for (String w : d.prefix("w", 1_000)) {
            total += d.lookup(w).size();
        }
        assertThat(d.meaningCount()).isEqualTo(total);
        assertThat(d.size()).isEqualTo(d.prefix("w", 1_000).size());
    }

    @Test
    void shouldAnswerPrefixAndSuggestQuicklyOnALargeDictionary() {
        DictionaryApp d = dict();
        for (int i = 0; i < 30_000; i++) {
            d.addWord(name(i), "noun", "meaning " + i);
        }

        Assertions.assertTimeoutPreemptively(Duration.ofSeconds(20), () -> {
            for (int i = 0; i < 3_000; i++) {
                assertThat(d.prefix("wa", 5)).hasSize(5);
                assertThat(d.lookup(name(i))).hasSize(1);
            }
            for (int i = 0; i < 20; i++) {
                assertThat(d.suggest("wqqqqx", 3)).isNotNull();
            }
        });
        assertThat(d.size()).isEqualTo(30_000);
    }

    @Test
    void shouldTreatDifferentCasesAndPaddingAsTheSameWordEverywhere() {
        DictionaryApp d = dict();
        d.addWord("Apple", "noun", "a fruit");

        assertThat(d.addWord(" APPLE\t", "noun", "a fruit")).isFalse();
        d.addWord("apple", "noun", "a company");
        d.addWord("pear", "noun", "a fruit");
        d.addSynonym(" APPLE ", "Pear");

        assertThat(d.size()).isEqualTo(2);
        assertThat(d.synonyms("pear")).containsExactly("apple");
        assertThat(d.removeWord("ApPlE")).isTrue();
        assertThat(d.synonyms("pear")).isEmpty();
        assertThat(d.search("fruit", 5)).containsExactly("pear");
    }
}
