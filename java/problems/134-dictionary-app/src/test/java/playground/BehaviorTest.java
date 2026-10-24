package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.DictionaryApp;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private DictionaryApp dict() {
        return Candidate.of(Solution::create);
    }

    /** cat, car, cart, dog, hound: the dictionary used in the README examples. */
    private DictionaryApp sample() {
        DictionaryApp d = dict();
        d.addWord("cat", "noun", "small domesticated feline");
        d.addWord("car", "noun", "a road vehicle");
        d.addWord("cart", "noun", "a vehicle pulled by an animal");
        d.addWord("dog", "noun", "a domesticated canine");
        d.addWord("hound", "noun", "a dog used for hunting");
        return d;
    }

    @Test
    void shouldStoreAMeaningAndLookItUp() {
        DictionaryApp d = dict();

        assertThat(d.addWord("bank", "NOUN", "a financial institution")).isTrue();

        assertThat(d.lookup("bank")).containsExactly("NOUN: a financial institution");
        assertThat(d.size()).isEqualTo(1);
        assertThat(d.meaningCount()).isEqualTo(1);
    }

    @Test
    void shouldKeepSeveralMeaningsInInsertionOrder() {
        DictionaryApp d = dict();
        d.addWord("bank", "NOUN", "a financial institution");
        d.addWord("bank", "VERB", "to tilt an aircraft");
        d.addWord("bank", "NOUN", "the land beside a river");

        assertThat(d.lookup("bank"))
                .containsExactly(
                        "NOUN: a financial institution", "VERB: to tilt an aircraft", "NOUN: the land beside a river");
        assertThat(d.size()).isEqualTo(1);
        assertThat(d.meaningCount()).isEqualTo(3);
    }

    @Test
    void shouldIgnoreTheSamePartOfSpeechAndMeaningTwice() {
        DictionaryApp d = dict();
        assertThat(d.addWord("bank", "NOUN", "a financial institution")).isTrue();

        assertThat(d.addWord("bank", "noun", "  a financial institution ")).isFalse();
        assertThat(d.addWord("bank", "VERB", "a financial institution")).isTrue();
        assertThat(d.addWord("bank", "NOUN", "A financial institution")).isTrue();

        assertThat(d.meaningCount()).isEqualTo(3);
    }

    @Test
    void shouldLowerCaseAndTrimWordsOnEveryCall() {
        DictionaryApp d = dict();
        d.addWord("  Bank ", "noun", "a financial institution");

        assertThat(d.lookup("BANK")).containsExactly("NOUN: a financial institution");
        assertThat(d.lookup(" bank")).hasSize(1);
        assertThat(d.prefix("BA", 5)).containsExactly("bank");
        assertThat(d.size()).isEqualTo(1);
    }

    @Test
    void shouldNormalisePartOfSpeechAndTrimTheMeaning() {
        DictionaryApp d = dict();
        d.addWord("run", " verb ", "  to move fast on foot  ");

        assertThat(d.lookup("run")).containsExactly("VERB: to move fast on foot");
    }

    @Test
    void shouldAcceptHyphensAndApostrophesInWords() {
        DictionaryApp d = dict();

        assertThat(d.addWord("Well-known", "adjective", "famous")).isTrue();
        assertThat(d.addWord("o'clock", "adverb", "by the clock")).isTrue();

        assertThat(d.lookup("well-known")).containsExactly("ADJECTIVE: famous");
        assertThat(d.lookup("O'Clock")).containsExactly("ADVERB: by the clock");
    }

    @Test
    void shouldRejectInvalidWordsPartsOfSpeechAndMeanings() {
        DictionaryApp d = dict();

        assertThatThrownBy(() -> d.addWord(null, "NOUN", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addWord("  ", "NOUN", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addWord("two words", "NOUN", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addWord("abc1", "NOUN", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addWord("café", "NOUN", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addWord("under_score", "NOUN", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addWord("word", "PREPOSITION", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addWord("word", null, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addWord("word", "NOUN", " ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addWord("word", "NOUN", null)).isInstanceOf(IllegalArgumentException.class);

        assertThat(d.size()).isZero();
        assertThat(d.meaningCount()).isZero();
        assertThatThrownBy(() -> d.lookup("bad word")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.lookup(" ")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldReturnAnEmptyListForAnUnknownWord() {
        DictionaryApp d = sample();

        assertThat(d.lookup("zebra")).isEmpty();
        assertThat(d.synonyms("zebra")).isEmpty();
    }

    @Test
    void shouldReturnALookupThatLaterChangesDoNotAffect() {
        DictionaryApp d = dict();
        d.addWord("bank", "NOUN", "a financial institution");
        List<String> before = new ArrayList<>(d.lookup("bank"));
        List<String> returned = d.lookup("bank");

        d.addWord("bank", "VERB", "to tilt an aircraft");

        assertThat(returned).containsExactlyElementsOf(before);
    }

    @Test
    void shouldUpdateAMeaningInPlace() {
        DictionaryApp d = dict();
        d.addWord("bank", "NOUN", "a financial institution");
        d.addWord("bank", "VERB", "to tilt an aircraft");

        d.updateMeaning("Bank", 1, "  to lean into a turn ");

        assertThat(d.lookup("bank")).containsExactly("NOUN: a financial institution", "VERB: to lean into a turn");
        assertThat(d.meaningCount()).isEqualTo(2);
    }

    @Test
    void shouldRejectBadUpdates() {
        DictionaryApp d = dict();
        d.addWord("bank", "NOUN", "a financial institution");
        d.addWord("bank", "NOUN", "the land beside a river");
        d.addWord("bank", "VERB", "the land beside a river");

        assertThatThrownBy(() -> d.updateMeaning("nope", 0, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.updateMeaning("bank", 3, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.updateMeaning("bank", -1, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.updateMeaning("bank", 0, "  ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.updateMeaning("bank", 0, "the land beside a river"))
                .isInstanceOf(IllegalArgumentException.class);

        assertThat(d.lookup("bank"))
                .containsExactly(
                        "NOUN: a financial institution", "NOUN: the land beside a river", "VERB: the land beside a river");
    }

    @Test
    void shouldAllowUpdatingAMeaningToItsOwnTextOrToAnotherPartOfSpeechsText() {
        DictionaryApp d = dict();
        d.addWord("bank", "NOUN", "a financial institution");
        d.addWord("bank", "VERB", "to tilt an aircraft");

        d.updateMeaning("bank", 0, "a financial institution");
        d.updateMeaning("bank", 1, "a financial institution");

        assertThat(d.lookup("bank")).containsExactly("NOUN: a financial institution", "VERB: a financial institution");
    }

    @Test
    void shouldShiftLaterMeaningsDownWhenOneIsRemoved() {
        DictionaryApp d = dict();
        d.addWord("bank", "NOUN", "first");
        d.addWord("bank", "NOUN", "second");
        d.addWord("bank", "NOUN", "third");

        d.removeMeaning("bank", 0);

        assertThat(d.lookup("bank")).containsExactly("NOUN: second", "NOUN: third");
        d.removeMeaning("bank", 1);
        assertThat(d.lookup("bank")).containsExactly("NOUN: second");
        assertThat(d.meaningCount()).isEqualTo(1);
    }

    @Test
    void shouldRemoveTheWordWhenItsLastMeaningIsRemoved() {
        DictionaryApp d = sample();

        d.removeMeaning("dog", 0);

        assertThat(d.lookup("dog")).isEmpty();
        assertThat(d.size()).isEqualTo(4);
        assertThat(d.removeWord("dog")).isFalse();
    }

    @Test
    void shouldRejectBadMeaningRemovals() {
        DictionaryApp d = sample();

        assertThatThrownBy(() -> d.removeMeaning("nope", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.removeMeaning("dog", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.removeMeaning("dog", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThat(d.lookup("dog")).hasSize(1);
    }

    @Test
    void shouldRemoveAWordAndReportWhetherItExisted() {
        DictionaryApp d = sample();

        assertThat(d.removeWord(" DOG ")).isTrue();
        assertThat(d.removeWord("dog")).isFalse();
        assertThat(d.removeWord("never-added")).isFalse();

        assertThat(d.lookup("dog")).isEmpty();
        assertThat(d.size()).isEqualTo(4);
        assertThat(d.meaningCount()).isEqualTo(4);
        assertThatThrownBy(() -> d.removeWord("")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldLinkSynonymsBothWaysSortedAndWithoutTransitivity() {
        DictionaryApp d = sample();
        d.addWord("canine", "noun", "a dog or related animal");

        assertThat(d.addSynonym("dog", "hound")).isTrue();
        assertThat(d.addSynonym("hound", "dog")).isFalse();
        assertThat(d.addSynonym("Canine", "hound")).isTrue();
        assertThat(d.addSynonym("dog", "canine")).isTrue();

        assertThat(d.synonyms("dog")).containsExactly("canine", "hound");
        assertThat(d.synonyms("hound")).containsExactly("canine", "dog");
        assertThat(d.synonyms("cat")).isEmpty();
    }

    @Test
    void shouldNotMakeSynonymsTransitive() {
        DictionaryApp d = sample();
        d.addSynonym("cat", "dog");
        d.addSynonym("dog", "hound");

        assertThat(d.synonyms("cat")).containsExactly("dog");
        assertThat(d.synonyms("hound")).containsExactly("dog");
    }

    @Test
    void shouldRejectBadSynonymLinks() {
        DictionaryApp d = sample();

        assertThatThrownBy(() -> d.addSynonym("dog", "dog")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addSynonym("dog", " DOG ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addSynonym("dog", "wolf")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addSynonym("wolf", "dog")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.addSynonym("dog", "")).isInstanceOf(IllegalArgumentException.class);

        assertThat(d.synonyms("dog")).isEmpty();
    }

    @Test
    void shouldDropSynonymLinksWhenAWordIsRemoved() {
        DictionaryApp d = sample();
        d.addSynonym("dog", "hound");
        d.addSynonym("cat", "dog");

        d.removeWord("dog");

        assertThat(d.synonyms("hound")).isEmpty();
        assertThat(d.synonyms("cat")).isEmpty();
        d.addWord("dog", "noun", "a domesticated canine");
        assertThat(d.synonyms("dog")).isEmpty();
        assertThat(d.synonyms("hound")).isEmpty();
    }

    @Test
    void shouldListWordsByPrefixSortedAndLimited() {
        DictionaryApp d = sample();

        assertThat(d.prefix("ca", 10)).containsExactly("car", "cart", "cat");
        assertThat(d.prefix("ca", 2)).containsExactly("car", "cart");
        assertThat(d.prefix("CART", 5)).containsExactly("cart");
        assertThat(d.prefix("z", 5)).isEmpty();
        assertThat(d.prefix("d", 1)).containsExactly("dog");
    }

    @Test
    void shouldRejectBadPrefixQueries() {
        DictionaryApp d = sample();

        assertThatThrownBy(() -> d.prefix("ca", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.prefix("ca", -3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.prefix(" ", 3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.prefix("c4", 3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.prefix(null, 3)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldSearchMeaningsCaseInsensitivelySortedByWord() {
        DictionaryApp d = sample();

        assertThat(d.search("DOMESTICATED", 10)).containsExactly("cat", "dog");
        assertThat(d.search("vehicle", 10)).containsExactly("car", "cart");
        assertThat(d.search("vehicle", 1)).containsExactly("car");
        assertThat(d.search("  an animal ", 10)).containsExactly("cart");
        assertThat(d.search("quantum", 10)).isEmpty();
    }

    @Test
    void shouldSearchOnlyMeaningsAndListEachWordOnce() {
        DictionaryApp d = sample();
        d.addWord("dog", "verb", "to follow closely");
        d.addWord("dog", "verb", "to pursue like a dog");

        assertThat(d.search("dog", 10)).containsExactly("dog", "hound");
        assertThat(d.search("hound", 10)).isEmpty();
        assertThat(d.search("NOUN", 10)).isEmpty();
    }

    @Test
    void shouldRejectBadSearchQueries() {
        DictionaryApp d = sample();

        assertThatThrownBy(() -> d.search("dog", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.search("  ", 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.search(null, 5)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldSuggestWordsAtEditDistanceOneForAnUnknownWord() {
        DictionaryApp d = sample();

        assertThat(d.suggest("cag", 5)).containsExactly("car", "cat");
        assertThat(d.suggest("do", 5)).containsExactly("dog");
        assertThat(d.suggest("cats", 5)).containsExactly("cat");
        assertThat(d.suggest("hund", 5)).containsExactly("hound");
        assertThat(d.suggest("CAG", 1)).containsExactly("car");
        assertThat(d.suggest("zzz", 5)).isEmpty();
    }

    @Test
    void shouldNotSuggestForAKnownWordAndRejectBadSuggestInput() {
        DictionaryApp d = sample();

        assertThat(d.suggest("cat", 5)).isEmpty();
        assertThatThrownBy(() -> d.suggest("cag", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.suggest(" ", 3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.suggest("c4g", 3)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldPickTheWordOfTheDayFromTheSortedWords() {
        DictionaryApp d = sample();

        assertThat(d.wordOfTheDay(0)).contains("car");
        assertThat(d.wordOfTheDay(2)).contains("cat");
        assertThat(d.wordOfTheDay(7)).contains("cat");
        assertThat(d.wordOfTheDay(5)).contains("car");
        assertThat(d.wordOfTheDay(-1)).contains("hound");
        assertThat(d.wordOfTheDay(-5)).contains("car");
    }

    @Test
    void shouldHaveNoWordOfTheDayInAnEmptyDictionary() {
        DictionaryApp d = dict();

        assertThat(d.wordOfTheDay(0)).isEmpty();
        assertThat(d.wordOfTheDay(-12)).isEmpty();
        assertThat(d.size()).isZero();
        assertThat(d.meaningCount()).isZero();
    }

    @Test
    void shouldSortWordsByPlainCharacterCode() {
        DictionaryApp d = dict();
        d.addWord("a-b", "noun", "x");
        d.addWord("a'b", "noun", "x");
        d.addWord("ab", "noun", "x");
        d.addWord("a", "noun", "x");

        assertThat(d.prefix("a", 10)).containsExactly("a", "a'b", "a-b", "ab");
        assertThat(d.wordOfTheDay(1)).contains("a'b");
    }

    @Test
    void shouldLoseNoMeaningWhenManyThreadsAddDifferentMeaningsToTheSameWord() {
        DictionaryApp d = dict();

        Concurrently.run(16, t -> {
            for (int i = 0; i < 100; i++) {
                d.addWord("bank", "NOUN", "meaning " + t + "-" + i);
            }
        });

        assertThat(d.lookup("bank")).hasSize(1_600);
        assertThat(d.meaningCount()).isEqualTo(1_600);
        assertThat(d.size()).isEqualTo(1);
    }

    @Test
    void shouldAcceptTheSameMeaningFromManyThreadsExactlyOnce() {
        DictionaryApp d = dict();

        List<Boolean> results = Concurrently.collect(16, t -> d.addWord("bank", "noun", "a financial institution"));

        assertThat(results).filteredOn(r -> r).hasSize(1);
        assertThat(d.lookup("bank")).containsExactly("NOUN: a financial institution");
    }

    @Test
    void shouldCreateEachWordOnceWhenThreadsAddDifferentWordsAndMeanings() {
        DictionaryApp d = dict();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 200; i++) {
                d.addWord("w" .concat(letters(i)), "verb", "by thread " + t);
            }
        });

        assertThat(d.size()).isEqualTo(200);
        assertThat(d.meaningCount()).isEqualTo(1_600);
    }

    private static String letters(int n) {
        StringBuilder b = new StringBuilder();
        do {
            b.append((char) ('a' + n % 26));
            n /= 26;
        } while (n > 0);
        return b.toString();
    }
}
