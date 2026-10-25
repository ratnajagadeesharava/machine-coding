package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.PlaceSearch;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private PlaceSearch search() {
        return Candidate.of(Solution::create);
    }

    /** Everything is allowed through: no category, no rating floor, price cap 4. */
    private static List<String> all(PlaceSearch s, String query, int ux, int uy, int radius, int k) {
        return s.search(query, ux, uy, radius, null, 0, 4, k);
    }

    private static PlaceSearch coffeeDistrict(PlaceSearch s) {
        s.addPlace("c1", "Coffee", "cafe", 3, 0, 40, 2);
        s.addPlace("c2", "Coffee House", "cafe", 1, 0, 40, 2);
        s.addPlace("c3", "Iced Coffee Bar", "cafe", 0, 2, 45, 3);
        s.addPlace("c4", "Bean Cart", "coffee", 5, 0, 30, 1);
        s.addPlace("p1", "Pizza", "pizza", 1, 1, 50, 2);
        return s;
    }

    @Test
    void shouldReproduceTheReadmeCoffeeExamples() {
        PlaceSearch s = coffeeDistrict(search());

        assertThat(s.search("coffee", 0, 0, 100, null, 40, 2, 10)).containsExactly("c1", "c2");
        assertThat(s.search(null, 0, 0, 100, "CAFE", 0, 4, 2)).containsExactly("c3", "c2");
        assertThat(s.search("coffee", 0, 0, 3, null, 0, 4, 10)).containsExactly("c3", "c1", "c2");
        assertThat(s.nearest(0, 0, 3)).containsExactly("c2", "c3", "p1");
        assertThat(s.categories()).containsExactly("cafe:3", "coffee:1", "pizza:1");
    }

    @Test
    void shouldRankByRatingTimesTenMinusThreeTimesDistance() {
        PlaceSearch s = search();
        s.addPlace("near", "Alpha", "cafe", 1, 0, 30, 2); // 300 - 3   = 297
        s.addPlace("good", "Beta", "cafe", 5, 0, 40, 2); // 400 - 15  = 385
        s.addPlace("far", "Gamma", "cafe", 40, 40, 50, 2); // 500 - 240 = 260

        assertThat(all(s, null, 0, 0, 1000, 10)).containsExactly("good", "near", "far");
    }

    @Test
    void shouldAddPrefixAndStackedEqualityBonusesForTheName() {
        PlaceSearch s = coffeeDistrict(search());

        // c3 444 (substring only), c1 441 (equals: +50), c2 422 (prefix: +25), c4 285 (category match, no bonus)
        assertThat(all(s, "coffee", 0, 0, 100, 10)).containsExactly("c3", "c1", "c2", "c4");
    }

    @Test
    void shouldMatchTheQueryAsACaseInsensitiveSubstringOfTheName() {
        PlaceSearch s = search();
        s.addPlace("a", "The Golden DRAGON", "chinese", 1, 0, 40, 2);
        s.addPlace("b", "Dragonfly", "bar", 2, 0, 40, 2);
        s.addPlace("c", "Noodle Bar", "chinese", 3, 0, 40, 2);

        assertThat(all(s, "dragon", 0, 0, 10, 10)).containsExactly("b", "a");
        assertThat(all(s, "GOLDEN dra", 0, 0, 10, 10)).containsExactly("a");
        assertThat(all(s, "xyz", 0, 0, 10, 10)).isEmpty();
    }

    @Test
    void shouldMatchTheQueryWhenItEqualsTheCategoryIgnoringCaseButNotWhenItIsOnlyPartOfIt() {
        PlaceSearch s = search();
        s.addPlace("a", "Spice Route", "Indian", 1, 0, 40, 2);
        s.addPlace("b", "Curry Leaf", "indian-fusion", 2, 0, 40, 2);

        assertThat(all(s, "INDIAN", 0, 0, 10, 10)).containsExactly("a");
        assertThat(all(s, "india", 0, 0, 10, 10)).isEmpty();
        assertThat(all(s, "fusion", 0, 0, 10, 10)).isEmpty();
    }

    @Test
    void shouldTreatNullAndBlankQueriesAsNoTextFilterAndAddNoBonus() {
        PlaceSearch s = search();
        s.addPlace("a", "Coffee", "cafe", 0, 0, 40, 2);
        s.addPlace("b", "Tea", "cafe", 0, 1, 41, 2);

        assertThat(all(s, null, 0, 0, 10, 10)).containsExactly("b", "a"); // 407 vs 400: no bonus for "a"
        assertThat(all(s, "", 0, 0, 10, 10)).containsExactly("b", "a");
        assertThat(all(s, "   ", 0, 0, 10, 10)).containsExactly("b", "a");
    }

    @Test
    void shouldIncludePlacesExactlyAtTheRadiusAndExcludeThoseBeyond() {
        PlaceSearch s = search();
        s.addPlace("in", "In", "cafe", 3, -2, 40, 2); // distance 5
        s.addPlace("out", "Out", "cafe", 3, -3, 40, 2); // distance 6

        assertThat(all(s, null, 0, 0, 5, 10)).containsExactly("in");
        assertThat(all(s, null, 0, 0, 6, 10)).containsExactly("in", "out");
        assertThat(all(s, null, 0, 0, 0, 10)).isEmpty();
    }

    @Test
    void shouldIncludeAPlaceAtDistanceZeroWhenTheRadiusIsZero() {
        PlaceSearch s = search();
        s.addPlace("here", "Here", "cafe", 7, 7, 20, 1);

        assertThat(all(s, null, 7, 7, 0, 5)).containsExactly("here");
    }

    @Test
    void shouldFilterByCategoryIgnoringCase() {
        PlaceSearch s = coffeeDistrict(search());

        assertThat(s.search(null, 0, 0, 100, "CAFE", 0, 4, 10)).containsExactlyInAnyOrder("c1", "c2", "c3");
        assertThat(s.search(null, 0, 0, 100, "Pizza", 0, 4, 10)).containsExactly("p1");
        assertThat(s.search(null, 0, 0, 100, "sushi", 0, 4, 10)).isEmpty();
        assertThat(s.search(null, 0, 0, 100, "", 0, 4, 10)).hasSize(5); // blank category means no category filter
    }

    @Test
    void shouldFilterByMinimumRatingInclusive() {
        PlaceSearch s = coffeeDistrict(search());

        assertThat(s.search("coffee", 0, 0, 100, null, 45, 4, 10)).containsExactly("c3");
        assertThat(s.search("coffee", 0, 0, 100, null, 40, 4, 10)).containsExactly("c3", "c1", "c2");
        assertThat(s.search("coffee", 0, 0, 100, null, 46, 4, 10)).isEmpty();
    }

    @Test
    void shouldFilterByMaximumPriceLevelInclusive() {
        PlaceSearch s = coffeeDistrict(search());

        assertThat(s.search("coffee", 0, 0, 100, null, 0, 3, 10)).containsExactly("c3", "c1", "c2", "c4");
        assertThat(s.search("coffee", 0, 0, 100, null, 0, 2, 10)).containsExactly("c1", "c2", "c4");
        assertThat(s.search("coffee", 0, 0, 100, null, 0, 1, 10)).containsExactly("c4");
        assertThat(s.search("coffee", 0, 0, 100, null, 0, 0, 10)).isEmpty();
    }

    @Test
    void shouldBreakEqualScoresByDistanceThenPlaceId() {
        PlaceSearch s = search();
        s.addPlace("far", "Far", "cafe", 10, 0, 43, 2); // 430 - 30 = 400, distance 10
        s.addPlace("close", "Close", "cafe", 0, 0, 40, 2); // 400, distance 0
        s.addPlace("b", "Twin B", "cafe", 0, 5, 41, 2); // 410 - 15 = 395, distance 5
        s.addPlace("a", "Twin A", "cafe", 5, 0, 41, 2); // 395, distance 5

        assertThat(all(s, null, 0, 0, 100, 10)).containsExactly("close", "far", "a", "b");
    }

    @Test
    void shouldReturnAtMostKResultsKeepingTheBestOnes() {
        PlaceSearch s = search();
        for (int i = 0; i < 20; i++) {
            s.addPlace(String.format("p%02d", i), "Place " + i, "cafe", i, 0, 20, 2); // closer is better
        }

        assertThat(all(s, null, 0, 0, 100, 3)).containsExactly("p00", "p01", "p02");
        assertThat(all(s, null, 0, 0, 100, 1000)).hasSize(20);
    }

    @Test
    void shouldReturnAnEmptyListWhenNothingMatchesOrNothingExists() {
        PlaceSearch s = search();
        assertThat(all(s, "coffee", 0, 0, 10, 5)).isEmpty();
        assertThat(s.nearest(0, 0, 5)).isEmpty();
        assertThat(s.categories()).isEmpty();
    }

    @Test
    void shouldRankNegativeScoresCorrectly() {
        PlaceSearch s = search();
        s.addPlace("a", "A", "cafe", 1000, 0, 0, 1); // -3000
        s.addPlace("b", "B", "cafe", 500, 0, 0, 1); // -1500
        s.addPlace("c", "C", "cafe", 2000, 0, 50, 1); // 500 - 6000 = -5500

        assertThat(all(s, null, 0, 0, 5000, 10)).containsExactly("b", "a", "c");
    }

    @Test
    void shouldHandleNegativeCoordinates() {
        PlaceSearch s = search();
        s.addPlace("a", "A", "cafe", -3, -4, 40, 2); // distance 7 from origin
        s.addPlace("b", "B", "cafe", 3, 4, 40, 2); // distance 7 from origin

        assertThat(all(s, null, 0, 0, 7, 10)).containsExactly("a", "b");
        assertThat(all(s, null, -3, -4, 0, 10)).containsExactly("a");
    }

    @Test
    void shouldRejectInvalidSearchArguments() {
        PlaceSearch s = coffeeDistrict(search());

        assertThatThrownBy(() -> all(s, null, 0, 0, 10, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> all(s, null, 0, 0, 10, -3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> all(s, null, 0, 0, -1, 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.nearest(0, 0, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.nearest(0, 0, -1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectDuplicateAndInvalidPlaces() {
        PlaceSearch s = search();
        s.addPlace("a", "A", "cafe", 0, 0, 40, 2);

        assertThatThrownBy(() -> s.addPlace("a", "Other", "bar", 1, 1, 10, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addPlace(null, "N", "cafe", 0, 0, 10, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addPlace(" ", "N", "cafe", 0, 0, 10, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addPlace("x", "", "cafe", 0, 0, 10, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addPlace("x", "N", " ", 0, 0, 10, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addPlace("x", "N", "cafe", 0, 0, 51, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addPlace("x", "N", "cafe", 0, 0, -1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addPlace("x", "N", "cafe", 0, 0, 10, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addPlace("x", "N", "cafe", 0, 0, 10, 5)).isInstanceOf(IllegalArgumentException.class);

        // rejected adds leave no trace, and the original place is untouched
        assertThat(all(s, null, 0, 0, 100, 10)).containsExactly("a");
        assertThat(s.categories()).containsExactly("cafe:1");
    }

    @Test
    void shouldAcceptTheBoundaryRatingsAndPriceLevels() {
        PlaceSearch s = search();
        s.addPlace("lo", "Lo", "cafe", 0, 0, 0, 1);
        s.addPlace("hi", "Hi", "cafe", 1, 0, 50, 4);

        assertThat(all(s, null, 0, 0, 5, 10)).containsExactly("hi", "lo"); // 497 vs 0
    }

    @Test
    void shouldUpdateAPlaceByReplacingAllItsAttributes() {
        PlaceSearch s = search();
        s.addPlace("a", "Old Name", "cafe", 50, 50, 10, 1);
        s.addPlace("b", "Rival", "cafe", 1, 0, 30, 2);
        assertThat(all(s, null, 0, 0, 200, 10)).containsExactly("b", "a");

        s.updatePlace("a", "New Diner", "diner", 0, 1, 50, 3);

        assertThat(all(s, null, 0, 0, 200, 10)).containsExactly("a", "b");
        assertThat(all(s, "old", 0, 0, 200, 10)).isEmpty();
        assertThat(all(s, "new", 0, 0, 200, 10)).containsExactly("a");
        assertThat(s.search(null, 0, 0, 200, "cafe", 0, 4, 10)).containsExactly("b");
        assertThat(s.categories()).containsExactly("cafe:1", "diner:1");
    }

    @Test
    void shouldRejectUpdatesAndRemovalsOfUnknownPlacesAndKeepStateOnFailedUpdate() {
        PlaceSearch s = search();
        s.addPlace("a", "Alpha", "cafe", 2, 0, 40, 2);

        assertThatThrownBy(() -> s.updatePlace("zzz", "Z", "cafe", 0, 0, 10, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.removePlace("zzz")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.updatePlace("a", "Beta", "bar", 9, 9, 60, 2)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.updatePlace("a", "Beta", "bar", 9, 9, 10, 9)).isInstanceOf(IllegalArgumentException.class);

        assertThat(all(s, "alpha", 2, 0, 0, 5)).containsExactly("a");
        assertThat(s.categories()).containsExactly("cafe:1");
    }

    @Test
    void shouldRemovePlacesAndAllowTheIdToBeUsedAgain() {
        PlaceSearch s = coffeeDistrict(search());

        s.removePlace("c3");

        assertThat(all(s, "coffee", 0, 0, 100, 10)).containsExactly("c1", "c2", "c4");
        assertThatThrownBy(() -> s.removePlace("c3")).isInstanceOf(IllegalArgumentException.class);
        s.addPlace("c3", "Fresh Coffee", "cafe", 0, 1, 50, 1);
        assertThat(all(s, "coffee", 0, 0, 100, 10)).startsWith("c3");
    }

    @Test
    void shouldReturnTheNearestPlacesIgnoringEveryOtherFilterWithIdTieBreak() {
        PlaceSearch s = search();
        s.addPlace("z", "Z", "cafe", 0, 2, 0, 4);
        s.addPlace("m", "M", "bar", 2, 0, 50, 1);
        s.addPlace("a", "A", "bar", 1, 1, 10, 3);
        s.addPlace("far", "Far", "cafe", 9, 9, 50, 1);
        s.addPlace("here", "Here", "cafe", 0, 0, 0, 4);

        assertThat(s.nearest(0, 0, 4)).containsExactly("here", "a", "m", "z");
        assertThat(s.nearest(0, 0, 100)).containsExactly("here", "a", "m", "z", "far");
        assertThat(s.nearest(9, 9, 1)).containsExactly("far");
    }

    @Test
    void shouldListCategoriesWithCountsSortedAndMergedIgnoringCase() {
        PlaceSearch s = search();
        s.addPlace("1", "A", "Cafe", 0, 0, 10, 1);
        s.addPlace("2", "B", "cafe", 0, 1, 10, 1);
        s.addPlace("3", "C", "BAR", 0, 2, 10, 1);
        s.addPlace("4", "D", "pizza", 0, 3, 10, 1);
        assertThat(s.categories()).containsExactly("bar:1", "cafe:2", "pizza:1");

        s.updatePlace("2", "B", "pizza", 0, 1, 10, 1);
        s.removePlace("3");
        assertThat(s.categories()).containsExactly("cafe:1", "pizza:2");
    }

    @Test
    void shouldNeverShowAHalfUpdatedPlaceToConcurrentSearches() {
        PlaceSearch s = search();
        s.addPlace("p", "Alpha", "cafe", 0, 0, 50, 1);
        AtomicBoolean stop = new AtomicBoolean(false);
        AtomicInteger torn = new AtomicInteger();

        Concurrently.run(5, 60_000, t -> {
            if (t == 0) {
                for (int i = 0; i < 20_000; i++) {
                    if (i % 2 == 0) {
                        s.updatePlace("p", "Beta", "bar", 100, 100, 0, 4);
                    } else {
                        s.updatePlace("p", "Alpha", "cafe", 0, 0, 50, 1);
                    }
                }
                stop.set(true);
                return;
            }
            while (!stop.get()) {
                // each of these asks for a mix of old and new values: it must never match
                if (!s.search("alpha", 0, 0, 1000, "bar", 0, 4, 5).isEmpty()
                        || !s.search("beta", 0, 0, 1000, "cafe", 0, 4, 5).isEmpty()
                        || !s.search("alpha", 100, 100, 0, null, 0, 4, 5).isEmpty()
                        || !s.search("beta", 0, 0, 0, null, 0, 4, 5).isEmpty()
                        || !s.search("beta", 0, 0, 1000, null, 1, 4, 5).isEmpty()
                        || !s.search("beta", 0, 0, 1000, null, 0, 3, 5).isEmpty()) {
                    torn.incrementAndGet();
                }
            }
        });

        assertThat(torn.get()).isZero();
        assertThat(s.nearest(0, 0, 5)).containsExactly("p");
    }

    @Test
    void shouldLoseNoPlaceWhenManyThreadsAddAtOnce() {
        PlaceSearch s = search();

        Concurrently.run(16, t -> {
            for (int i = 0; i < 250; i++) {
                s.addPlace("t" + t + "-" + i, "Place " + i, "cafe", t, i, 20, 2);
                s.search("place", 0, 0, 10_000, null, 0, 4, 5);
            }
        });

        assertThat(s.nearest(0, 0, 10_000)).hasSize(4_000);
        assertThat(s.categories()).containsExactly("cafe:4000");
    }

    @Test
    void shouldAnswerTwoThousandSearchesOverFiftyThousandPlacesWithinThirtySeconds() {
        PlaceSearch s = search();
        String[] cats = {"cafe", "pizza", "bar", "sushi", "bakery", "gym", "books", "pharmacy"};
        String[] words = {"Golden", "Blue", "Happy", "Urban", "Old", "Royal", "Green", "Spicy"};
        long seed = 12345;
        for (int i = 0; i < 50_000; i++) {
            seed = (seed * 6364136223846793005L + 1442695040888963407L);
            int a = (int) ((seed >>> 33) % 2_000);
            seed = (seed * 6364136223846793005L + 1442695040888963407L);
            int b = (int) ((seed >>> 33) % 2_000);
            s.addPlace("p" + i, words[i % 8] + " " + cats[(i / 8) % 8] + " " + i, cats[(i / 8) % 8], a, b, (i * 7) % 51, 1 + i % 4);
        }

        long start = System.nanoTime();
        int nonEmpty = 0;
        for (int q = 0; q < 2_000; q++) {
            List<String> r = s.search(q % 3 == 0 ? "golden" : q % 3 == 1 ? "pizza" : null, (q * 37) % 2_000, (q * 91) % 2_000,
                    300 + q % 400, q % 5 == 0 ? "cafe" : null, 10, 4, 10);
            if (!r.isEmpty()) {
                nonEmpty++;
            }
            assertThat(r).hasSizeLessThanOrEqualTo(10);
        }
        long elapsedMs = (System.nanoTime() - start) / 1_000_000;

        assertThat(nonEmpty).isGreaterThan(1_500);
        assertThat(elapsedMs).as("2,000 searches over 50,000 places").isLessThan(30_000);
        assertThat(s.categories()).hasSize(8);
    }
}
