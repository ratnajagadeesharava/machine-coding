package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.CityMap;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static String cell(int r, int c) {
        return String.format("r%03dc%03d", r, c);
    }

    private static CityMap grid(int n, long w) {
        CityMap m = Candidate.of(Solution::create);
        for (int r = 0; r < n; r++) {
            for (int c = 0; c < n; c++) {
                m.addLocation(cell(r, c));
            }
        }
        for (int r = 0; r < n; r++) {
            for (int c = 0; c < n; c++) {
                if (c + 1 < n) {
                    m.addRoad(cell(r, c), cell(r, c + 1), w);
                }
                if (r + 1 < n) {
                    m.addRoad(cell(r, c), cell(r + 1, c), w);
                }
            }
        }
        return m;
    }

    @Test
    void shouldPickTheLexicographicallySmallestOfAHugeNumberOfEqualRoutes() {
        int n = 150;
        CityMap m = grid(n, 1);
        List<String> expected = new ArrayList<>();
        for (int c = 0; c < n; c++) {
            expected.add(cell(0, c)); // "r000c001" < "r001c000": go right first, then down
        }
        for (int r = 1; r < n; r++) {
            expected.add(cell(r, n - 1));
        }
        assertThat(m.shortestPath(cell(0, 0), cell(n - 1, n - 1))).containsExactlyElementsOf(expected);
    }

    @Test
    void shouldMixAllQueryKindsOnALargeGridWithUpdatesBetweenThem() {
        int n = 150;
        CityMap m = grid(n, 3);
        for (int i = 0; i < n; i += 15) {
            m.tagLandmark(cell(i, (i * 7) % n), "Cafe");
        }
        long start = System.nanoTime();
        long seed = 99;
        for (int q = 0; q < 3_000; q++) {
            seed = seed * 6364136223846793005L + 1442695040888963407L;
            int r1 = (int) ((seed >>> 33) % n);
            int c1 = (int) ((seed >>> 20) % n);
            int r2 = (int) ((seed >>> 45) % n);
            int c2 = (int) ((seed >>> 11) % n);
            switch (q % 4) {
                case 0 -> assertThat(m.shortestDistance(cell(r1, c1), cell(r2, c2)))
                        .isEqualTo(3L * (Math.abs(r1 - r2) + Math.abs(c1 - c2)));
                case 1 -> {
                    List<String> p = m.shortestPath(cell(r1, c1), cell(r2, c2));
                    assertThat(p).hasSize(Math.abs(r1 - r2) + Math.abs(c1 - c2) + 1);
                }
                case 2 -> assertThat(m.nearestLandmarks(cell(r1, c1), "cafe", 3)).hasSize(3);
                default -> assertThat(m.withinRadius(cell(r1, c1), 6)).isNotEmpty();
            }
        }
        assertThat((System.nanoTime() - start) / 1_000_000).as("elapsed ms").isLessThan(120_000);
    }

    @Test
    void shouldReflectEveryUpdateOnAGridWithNoStaleAnswers() {
        int n = 40;
        CityMap m = grid(n, 1);
        String from = cell(0, 0);
        String to = cell(0, n - 1);
        assertThat(m.shortestDistance(from, to)).isEqualTo(n - 1);

        // Cut the road along row 0 in the middle: a detour through row 1 costs two extra.
        m.removeRoad(cell(0, 20), cell(0, 21));
        assertThat(m.shortestDistance(from, to)).isEqualTo(n - 1 + 2);

        // Wall off every crossing between columns 20 and 21 except the bottom one.
        for (int r = 1; r < n - 1; r++) {
            m.removeRoad(cell(r, 20), cell(r, 21));
        }
        assertThat(m.shortestDistance(from, to)).isEqualTo((n - 1) + 2 * (n - 1));

        // A shortcut makes it cheap again, and a removal brings the long way back.
        m.addRoad(cell(0, 20), cell(0, 21), 1);
        assertThat(m.shortestDistance(from, to)).isEqualTo(n - 1);
        assertThat(m.removeRoad(cell(0, 21), cell(0, 20))).isEqualTo(1);
        assertThat(m.shortestDistance(from, to)).isEqualTo((n - 1) + 2 * (n - 1));
    }

    @Test
    void shouldReconnectSeparateComponentsAndRouteAcrossTheNewRoad() {
        CityMap m = Candidate.of(Solution::create);
        for (String s : List.of("a1", "a2", "b1", "b2")) {
            m.addLocation(s);
        }
        m.addRoad("a1", "a2", 5);
        m.addRoad("b1", "b2", 5);
        m.tagLandmark("b2", "park");
        assertThat(m.nearestLandmarks("a1", "park", 1)).isEmpty();
        assertThat(m.shortestPath("a1", "b2")).isEmpty();

        m.addRoad("a2", "b1", 2);
        assertThat(m.nearestLandmarks("a1", "park", 1)).containsExactly("b2:12");
        assertThat(m.shortestPath("a1", "b2")).containsExactly("a1", "a2", "b1", "b2");
        assertThat(m.withinRadius("a1", 11)).containsExactly("a1", "a2", "b1");
    }

    @Test
    void shouldReportLargeTotalsInLandmarkResults() {
        CityMap m = Candidate.of(Solution::create);
        for (String s : List.of("home", "mid", "far")) {
            m.addLocation(s);
        }
        m.addRoad("home", "mid", 3_000_000_000L);
        m.addRoad("mid", "far", 3_000_000_000L);
        m.tagLandmark("far", "Airport");
        m.tagLandmark("mid", "airport");
        assertThat(m.nearestLandmarks("home", "AIRPORT", 5)).containsExactly("mid:3000000000", "far:6000000000");
        assertThat(m.shortestDistance("far", "home")).isEqualTo(6_000_000_000L);
    }

    @Test
    void shouldBreakAThousandWayDistanceTieByNameAndCompareNamesAsPlainStrings() {
        CityMap m = Candidate.of(Solution::create);
        m.addLocation("centre");
        List<String> names = new ArrayList<>();
        for (int i = 0; i < 1_000; i++) {
            names.add("shop" + i);
        }
        names.add("Shop");
        for (String s : names) {
            m.addLocation(s);
            m.addRoad("centre", s, 10);
            m.tagLandmark(s, "shop");
        }
        // plain String.compareTo: "Shop" < "shop0" < "shop1" < "shop10" < "shop100" < "shop101" ...
        assertThat(m.nearestLandmarks("centre", "shop", 5))
                .containsExactly("Shop:10", "shop0:10", "shop1:10", "shop10:10", "shop100:10");
        assertThat(m.withinRadius("centre", 10)).hasSize(1_002);
        assertThat(m.withinRadius("centre", 10).get(0)).isEqualTo("Shop");
    }

    @Test
    void shouldKeepQueriesConsistentWhileTagsAndRoadsFlipConcurrently() {
        CityMap m = Candidate.of(Solution::create);
        for (String s : List.of("s", "x", "y")) {
            m.addLocation(s);
        }
        m.addRoad("s", "x", 4);
        m.addRoad("s", "y", 9);
        m.tagLandmark("s", "base");

        Concurrently.run(6, 90_000, t -> {
            if (t == 0) {
                for (int i = 0; i < 4_000; i++) {
                    m.tagLandmark("x", "cafe");
                    m.untagLandmark("x", "cafe");
                }
                return;
            }
            if (t == 1) {
                for (int i = 0; i < 4_000; i++) {
                    m.addRoad("x", "y", 1);
                    m.removeRoad("x", "y");
                }
                return;
            }
            for (int i = 0; i < 2_000; i++) {
                List<String> cafes = m.nearestLandmarks("s", "cafe", 5);
                assertThat(cafes).isIn(List.of(), List.of("x:4"));
                long d = m.shortestDistance("s", "y");
                assertThat(d).isIn(9L, 5L);
                assertThat(m.nearestLandmarks("s", "base", 5)).containsExactly("s:0");
                assertThat(m.withinRadius("s", 4)).contains("s", "x");
            }
        });
        assertThat(m.shortestDistance("s", "y")).isEqualTo(9);
        assertThat(m.nearestLandmarks("s", "cafe", 1)).isEmpty();
    }

    @Test
    void shouldStayCorrectWhenManyThreadsGrowAndQueryTheSameMap() {
        CityMap m = Candidate.of(Solution::create);
        m.addLocation("root");
        Concurrently.run(8, 90_000, t -> {
            String prev = "root";
            for (int i = 0; i < 300; i++) {
                String cur = "t" + t + "-" + i;
                m.addLocation(cur);
                m.addRoad(prev, cur, 2);
                prev = cur;
                assertThat(m.shortestDistance("root", cur)).isEqualTo(2L * (i + 1));
            }
        });
        assertThat(m.shortestDistance("t0-299", "t7-299")).isEqualTo(2L * 300 * 2);
        assertThat(m.shortestPath("t1-1", "t2-0")).containsExactly("t1-1", "t1-0", "root", "t2-0");
    }
}
