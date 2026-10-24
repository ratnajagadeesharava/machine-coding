package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.MusicService;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private MusicService service() {
        return Candidate.of(Solution::create);
    }

    @Test
    void shouldFindSongsInALargeCatalogueQuickly() {
        MusicService m = service();
        m.registerUser("u");
        for (int i = 0; i < 100_000; i++) {
            m.addSong("s" + i, "Song w" + i + " moon", "Artist a" + (i % 1000), 100);
        }
        m.play("u", "s4242");

        long start = System.nanoTime();
        for (int i = 0; i < 300; i++) {
            List<String> r = m.search("w4242 a242", 5);
            assertThat(r).containsExactly("s4242");
            assertThat(m.search("w" + (i * 7 + 10) + " a" + ((i * 7 + 10) % 1000), 3)).contains("s" + (i * 7 + 10));
        }
        assertThat((System.nanoTime() - start) / 1_000_000).isLessThan(10_000);
    }

    @Test
    void shouldReRankSearchAsPlaysAccumulate() {
        MusicService m = service();
        m.registerUser("u");
        m.addSong("a", "Alpha Wave", "Zen", 100);
        m.addSong("b", "Beta Wave", "Zen", 100);
        m.addSong("c", "Gamma Wave", "Zen", 100);

        assertThat(m.search("wave", 3)).containsExactly("a", "b", "c");
        m.play("u", "c");
        assertThat(m.search("wave", 3)).containsExactly("c", "a", "b");
        m.play("u", "b");
        m.play("u", "b");
        assertThat(m.search("wave", 3)).containsExactly("b", "c", "a");
        m.play("u", "c");
        assertThat(m.search("wave", 3)).containsExactly("b", "c", "a");
        m.play("u", "c");
        assertThat(m.search("wave", 2)).containsExactly("c", "b");
    }

    @Test
    void shouldAllowAQueryTokenToMatchTheSameWordAsAnotherToken() {
        MusicService m = service();
        m.addSong("x", "Rock Rocket", "Rocky Road", 100);
        m.addSong("y", "Rock", "Band", 100);

        assertThat(m.search("rock rock", 5)).containsExactly("y", "x");
        assertThat(m.search("rocket rocky", 5)).containsExactly("x");
        assertThat(m.search("r r r", 5)).containsExactly("y", "x").doesNotHaveDuplicates();
        assertThat(m.search("rock road band", 5)).isEmpty();
    }

    @Test
    void shouldKeepHistoryDistinctWhenTheSameSongIsPlayedRepeatedly() {
        MusicService m = service();
        m.registerUser("u");
        for (int i = 0; i < 11; i++) {
            m.addSong("g" + i, "T" + i, "B", 60);
        }
        for (int i = 0; i < 11; i++) {
            m.play("u", "g" + i);
            m.play("u", "g" + i);
        }
        assertThat(m.recentlyPlayed("u")).hasSize(10).doesNotHaveDuplicates();
        assertThat(m.recentlyPlayed("u").get(0)).isEqualTo("g10");
        assertThat(m.recentlyPlayed("u")).doesNotContain("g0");
        assertThat(m.userPlayCount("u", "g0")).isEqualTo(2);

        m.play("u", "g0");
        assertThat(m.recentlyPlayed("u")).hasSize(10).startsWith("g0", "g10").doesNotContain("g1");
    }

    @Test
    void shouldKeepTwoUsersCompletelyIndependent() {
        MusicService m = service();
        m.registerUser("a");
        m.registerUser("b");
        m.addSong("s1", "One", "X", 10);
        m.addSong("s2", "Two", "X", 10);
        m.addSong("s3", "Three", "X", 10);

        String pa = m.createPlaylist("a", "same");
        String pb = m.createPlaylist("b", "same");
        m.addToPlaylist("a", pa, "s1");
        m.addToPlaylist("b", pb, "s2");
        m.enqueue("a", "s3");
        m.play("a", "s1");

        assertThat(m.playlist("a", pa)).containsExactly("s1");
        assertThat(m.playlist("b", pb)).containsExactly("s2");
        assertThat(m.recentlyPlayed("b")).isEmpty();
        assertThat(m.playNext("b")).isEmpty();
        assertThat(m.recommendations("b", 5)).isEmpty();
        assertThat(m.recommendations("a", 5)).containsExactly("s2", "s3");
        m.deletePlaylist("a", pa);
        assertThat(m.playlist("b", pb)).containsExactly("s2");
    }

    @Test
    void shouldExcludeSongsPlayedThroughTheQueueFromRecommendationsAndRankByArtistPlays() {
        MusicService m = service();
        m.registerUser("u");
        m.addSong("a1", "A one", "Alpha", 10);
        m.addSong("a2", "A two", "alpha", 10);
        m.addSong("a3", "A three", "Alpha", 10);
        m.addSong("b1", "B one", "Beta", 10);
        m.addSong("b2", "B two", "Beta", 10);
        m.addSong("c1", "C one", "Gamma", 10);

        m.enqueue("u", "a1");
        m.enqueue("u", "a2");
        m.enqueue("u", "b1");
        m.playNext("u");
        m.playNext("u");
        m.playNext("u");

        assertThat(m.recommendations("u", 10)).containsExactly("a3", "b2");
        m.play("u", "a1");
        m.play("u", "b1");
        m.play("u", "b1");
        m.play("u", "b1");
        assertThat(m.recommendations("u", 10)).containsExactly("b2", "a3");
        assertThat(m.recommendations("u", 1)).containsExactly("b2");
    }

    @Test
    void shouldStayConsistentUnderMixedConcurrentTraffic() {
        MusicService m = service();
        for (int i = 0; i < 50; i++) {
            m.addSong("s" + i, "Song " + i, "Artist " + (i % 5), 100);
        }
        String[] playlists = new String[8];
        for (int i = 0; i < 8; i++) {
            m.registerUser("u" + i);
        }

        Concurrently.run(8, 60_000, t -> {
            String u = "u" + t;
            playlists[t] = m.createPlaylist(u, "list");
            for (int i = 0; i < 2_000; i++) {
                String s = "s" + (i % 50);
                m.play(u, s);
                if (i % 7 == 0) {
                    m.like(u, s);
                }
                if (i % 11 == 0) {
                    m.enqueue(u, s);
                }
                if (i % 13 == 0) {
                    m.playNext(u);
                }
                if (i < 50) {
                    m.addToPlaylist(u, playlists[t], s);
                }
                m.search("song artist", 5);
                m.topSongs(5);
                m.recommendations(u, 5);
                m.recentlyPlayed(u);
            }
        });

        // Draining the queues turns every enqueue (182 per user: i % 11 == 0 over 0..1999) into exactly one play.
        for (int t = 0; t < 8; t++) {
            while (m.playNext("u" + t).isPresent()) {
                // drain
            }
        }
        long total = 0;
        for (int i = 0; i < 50; i++) {
            total += m.playCount("s" + i);
        }
        assertThat(total).isEqualTo(8L * 2_000 + 8L * 182);
        assertThat(m.topSongs(100)).hasSize(50);
        List<String> expected = new ArrayList<>();
        for (int i = 0; i < 50; i++) {
            expected.add("s" + i);
        }
        assertThat(m.playlist("u3", playlists[3])).isEqualTo(expected);
        assertThat(m.recentlyPlayed("u3")).hasSize(10).doesNotHaveDuplicates();
        assertThat(m.songLikes("s0")).isEqualTo(8);
    }

    @Test
    void shouldSerializeConcurrentQueueAppendsWithoutLosingAny() {
        MusicService m = service();
        m.addSong("s", "Song", "Artist", 100);
        m.registerUser("u");
        Concurrently.run(8, t -> {
            for (int i = 0; i < 1_000; i++) {
                m.enqueue("u", "s");
            }
        });
        int n = 0;
        while (m.playNext("u").isPresent()) {
            n++;
        }
        assertThat(n).isEqualTo(8_000);
        assertThat(m.playCount("s")).isEqualTo(8_000);
    }
}
