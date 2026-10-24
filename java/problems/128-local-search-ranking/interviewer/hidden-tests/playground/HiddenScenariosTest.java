package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.PlaceSearch;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Locale;
import java.util.Random;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private record P(String id, String name, String cat, int x, int y, int rating, int price) {}

    private static List<String> oracle(List<P> ps, String q, int ux, int uy, int radius, String cat, int minR, int maxP, int k) {
        String query = q == null || q.isBlank() ? null : q.toLowerCase(Locale.ROOT);
        record R(String id, long score, long dist) {}
        List<R> rs = new ArrayList<>();
        for (P p : ps) {
            long d = Math.abs((long) p.x - ux) + Math.abs((long) p.y - uy);
            String name = p.name.toLowerCase(Locale.ROOT);
            String c = p.cat.toLowerCase(Locale.ROOT);
            if (d > radius || p.rating < minR || p.price > maxP) continue;
            if (cat != null && !cat.isBlank() && !c.equals(cat.toLowerCase(Locale.ROOT))) continue;
            long score = p.rating * 10L - 3 * d;
            if (query != null) {
                if (!name.contains(query) && !c.equals(query)) continue;
                if (name.startsWith(query)) score += 25;
                if (name.equals(query)) score += 25;
            }
            rs.add(new R(p.id, score, d));
        }
        rs.sort(Comparator.comparingLong(R::score).reversed().thenComparingLong(R::dist).thenComparing(R::id));
        return rs.stream().limit(k).map(R::id).toList();
    }

    @Test
    void shouldAgreeWithABruteForceOracleOnRandomSearches() {
        PlaceSearch s = Candidate.of(Solution::create);
        Random rnd = new Random(42);
        String[] words = {"coffee", "Coffee", "tea", "bar", "Bar", "pizza", "grill", "cafe"};
        String[] cats = {"cafe", "Bar", "pizza", "tea", "coffee"};
        List<P> ps = new ArrayList<>();
        for (int i = 0; i < 3_000; i++) {
            P p = new P("id" + rnd.nextInt(1_000_000) + "-" + i, words[rnd.nextInt(8)] + (rnd.nextBoolean() ? "" : " " + words[rnd.nextInt(8)]),
                    cats[rnd.nextInt(5)], rnd.nextInt(121) - 60, rnd.nextInt(121) - 60, rnd.nextInt(51), 1 + rnd.nextInt(4));
            ps.add(p);
            s.addPlace(p.id, p.name, p.cat, p.x, p.y, p.rating, p.price);
        }
        String[] queries = {null, "", "coffee", "COFFEE", "bar", "Tea", "pizza g", "cafe", "gril", "zzz", "coffee coffee"};
        for (int q = 0; q < 400; q++) {
            String query = queries[rnd.nextInt(queries.length)];
            String cat = rnd.nextInt(3) == 0 ? cats[rnd.nextInt(5)].toUpperCase(Locale.ROOT) : null;
            int ux = rnd.nextInt(161) - 80, uy = rnd.nextInt(161) - 80, radius = rnd.nextInt(150);
            int minR = rnd.nextInt(30), maxP = 1 + rnd.nextInt(4), k = 1 + rnd.nextInt(40);
            assertThat(s.search(query, ux, uy, radius, cat, minR, maxP, k))
                    .as("query=%s cat=%s u=(%d,%d) r=%d minR=%d maxP=%d k=%d", query, cat, ux, uy, radius, minR, maxP, k)
                    .containsExactlyElementsOf(oracle(ps, query, ux, uy, radius, cat, minR, maxP, k));
        }
    }

    @Test
    void shouldNotOverflowWithExtremeCoordinatesAndRadii() {
        PlaceSearch s = Candidate.of(Solution::create);
        s.addPlace("west", "West", "cafe", Integer.MIN_VALUE, 0, 50, 1);
        s.addPlace("east", "East", "cafe", Integer.MAX_VALUE, 0, 50, 1);
        s.addPlace("origin", "Origin", "cafe", 0, 0, 0, 1);

        // distances here exceed the range of an int
        assertThat(s.search(null, Integer.MIN_VALUE, 0, Integer.MAX_VALUE, null, 0, 4, 10)).containsExactly("west"); // origin is 2^31 away: just beyond the largest radius
        assertThat(s.search(null, Integer.MAX_VALUE, 0, Integer.MAX_VALUE, null, 0, 4, 10)).containsExactly("east", "origin");
        assertThat(s.nearest(Integer.MIN_VALUE, Integer.MIN_VALUE, 3)).containsExactly("west", "origin", "east");
    }

    @Test
    void shouldAcceptAnEnormousKWithoutPreallocatingForIt() {
        PlaceSearch s = Candidate.of(Solution::create);
        s.addPlace("a", "A", "cafe", 1, 0, 10, 1);
        s.addPlace("b", "B", "cafe", 2, 0, 10, 1);

        assertThat(s.search(null, 0, 0, 10, null, 0, 4, Integer.MAX_VALUE)).containsExactly("a", "b");
        assertThat(s.nearest(0, 0, Integer.MAX_VALUE)).containsExactly("a", "b");
    }

    @Test
    void shouldApplyBonusesOnNameOnlyAndIgnoreCase() {
        PlaceSearch s = Candidate.of(Solution::create);
        s.addPlace("cart", "Coffee Cart", "coffee", 0, 0, 40, 1); // name prefix and category equals the query: 400 + 25
        s.addPlace("exact", "COFFEE", "bakery", 5, 0, 40, 1); // 400 - 15 + 50 = 435
        s.addPlace("sub", "Best coffee", "bakery", 0, 0, 43, 1); // 430, substring only
        s.addPlace("cat", "Beans", "COFFEE", 0, 0, 42, 1); // 420, matches the category only

        assertThat(s.search("Coffee", 0, 0, 50, null, 0, 4, 10)).containsExactly("exact", "sub", "cart", "cat");
    }

    @Test
    void shouldKeepResultsSortedWhenPlacesAreUpdatedAndRemovedBetweenSearches() {
        PlaceSearch s = Candidate.of(Solution::create);
        for (int i = 0; i < 200; i++) {
            s.addPlace(String.format("p%03d", i), "Spot " + i, i % 2 == 0 ? "cafe" : "bar", i, 0, 25, 2);
        }
        for (int i = 0; i < 200; i += 3) {
            s.removePlace(String.format("p%03d", i));
        }
        for (int i = 1; i < 200; i += 3) {
            s.updatePlace(String.format("p%03d", i), "Spot " + i, "cafe", 0, i, 25, 2);
        }

        List<String> top = s.search(null, 0, 0, 1_000, null, 0, 4, 5);
        assertThat(top).containsExactly("p001", "p002", "p004", "p005", "p007");
        assertThat(s.categories()).containsExactly("bar:" + 33, "cafe:" + 100);
        assertThat(s.nearest(0, 0, 3)).containsExactly("p001", "p002", "p004");
    }

    @Test
    void shouldStayConsistentWhileSearchesRunDuringAddsUpdatesAndRemoves() {
        PlaceSearch s = Candidate.of(Solution::create);
        for (int i = 0; i < 500; i++) {
            s.addPlace("base" + i, "Base " + i, "cafe", i, i, 25, 2);
        }

        Concurrently.run(8, 90_000, t -> {
            if (t < 3) {
                for (int i = 0; i < 1_500; i++) {
                    String id = "w" + t + "-" + i;
                    s.addPlace(id, "Writer " + t, "bar", i % 100, t, 30, 1);
                    s.updatePlace(id, "Writer " + t, "bar", i % 100, t + 1, 31, 1);
                    if (i % 2 == 0) {
                        s.removePlace(id);
                    }
                }
                return;
            }
            for (int i = 0; i < 3_000; i++) {
                List<String> r = s.search("base", 0, 0, 10_000, "cafe", 0, 4, 10);
                assertThat(r).hasSize(10).startsWith("base0", "base1", "base2");
                assertThat(s.nearest(0, 0, 3)).hasSize(3);
            }
        });

        assertThat(s.categories()).containsExactly("bar:" + 3 * 750, "cafe:500");
        assertThat(s.search("writer 1", 1, 2, 0, "bar", 0, 4, 1_000)).hasSize(15); // i = 1, 101, ..., 1401
    }

    @Test
    void shouldHandleTenThousandSearchesOverOneHundredThousandPlaces() {
        PlaceSearch s = Candidate.of(Solution::create);
        Random rnd = new Random(7);
        for (int i = 0; i < 100_000; i++) {
            s.addPlace("p" + i, "Place " + (i % 977), "cat" + (i % 13), rnd.nextInt(4_000), rnd.nextInt(4_000), rnd.nextInt(51), 1 + i % 4);
        }
        long start = System.nanoTime();
        for (int q = 0; q < 10_000; q++) {
            s.search(q % 2 == 0 ? "place 9" : null, rnd.nextInt(4_000), rnd.nextInt(4_000), 200 + q % 300, null, 20, 3, 20);
        }
        assertThat((System.nanoTime() - start) / 1_000_000).isLessThan(120_000);
    }
}
