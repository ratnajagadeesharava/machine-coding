package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.MusicService;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private MusicService service() {
        return Candidate.of(Solution::create);
    }

    /** Small catalogue plus users alice, bob. */
    private MusicService catalogue() {
        MusicService m = service();
        m.addSong("q1", "Bohemian Rhapsody", "Queen", 354);
        m.addSong("q2", "Don't Stop Me Now", "Queen", 209);
        m.addSong("q3", "Another One Bites the Dust", "Queen", 215);
        m.addSong("j1", "Imagine", "John Lennon", 183);
        m.addSong("b1", "Hey Jude", "The Beatles", 431);
        m.addSong("b2", "Let It Be", "The Beatles", 243);
        m.addSong("b3", "Yesterday", "The Beatles", 125);
        m.registerUser("alice");
        m.registerUser("bob");
        return m;
    }

    private static void plays(MusicService m, String user, String song, int times) {
        for (int i = 0; i < times; i++) {
            m.play(user, song);
        }
    }

    @Test
    void shouldRejectDuplicateAndInvalidSongs() {
        MusicService m = catalogue();

        assertThatThrownBy(() -> m.addSong("q1", "Other", "Other", 100)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addSong(" ", "T", "A", 100)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addSong("x", "", "A", 100)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addSong("x", "T", " ", 100)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addSong("x", "T", "A", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addSong(null, "T", "A", 10)).isInstanceOf(IllegalArgumentException.class);
        assertThat(m.search("other", 5)).isEmpty();
    }

    @Test
    void shouldRejectDuplicateAndBlankUsers() {
        MusicService m = catalogue();

        assertThatThrownBy(() -> m.registerUser("alice")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.registerUser(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.registerUser(null)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldSearchByTitlePrefixIgnoringCase() {
        MusicService m = catalogue();

        assertThat(m.search("BOHE", 10)).containsExactly("q1");
        assertThat(m.search("rhapsody", 10)).containsExactly("q1");
        assertThat(m.search("Imag", 10)).containsExactly("j1");
    }

    @Test
    void shouldSearchByArtistWordPrefix() {
        MusicService m = catalogue();

        assertThat(m.search("queen", 10)).containsExactly("q3", "q1", "q2");
        assertThat(m.search("ennon", 10)).isEmpty();
        assertThat(m.search("lenn", 10)).containsExactly("j1");
    }

    @Test
    void shouldRequireEveryTokenToMatchInTitleOrArtist() {
        MusicService m = catalogue();

        assertThat(m.search("queen rhap", 10)).containsExactly("q1");
        assertThat(m.search("rhap queen", 10)).containsExactly("q1");
        assertThat(m.search("john imag", 10)).containsExactly("j1");
        assertThat(m.search("the be", 10)).containsExactly("b1", "b2", "b3");
        assertThat(m.search("one dust", 10)).containsExactly("q3");
        assertThat(m.search("queen imagine", 10)).isEmpty();
    }

    @Test
    void shouldMatchWordsNotSubstringsAndNotWholeStringPrefixes() {
        MusicService m = catalogue();

        assertThat(m.search("hemian", 10)).isEmpty();
        assertThat(m.search("stop", 10)).containsExactly("q2");
        assertThat(m.search("me now", 10)).containsExactly("q2");
        assertThat(m.search("don't st", 10)).containsExactly("q2");
        assertThat(m.search("stop don't", 10)).containsExactly("q2");
    }

    @Test
    void shouldIgnoreExtraWhitespaceInTheQuery() {
        MusicService m = catalogue();

        assertThat(m.search("  queen \t  rhap  ", 10)).containsExactly("q1");
    }

    @Test
    void shouldRankSearchResultsByPlaysThenTitleThenId() {
        MusicService m = catalogue();
        plays(m, "alice", "b3", 2);
        plays(m, "bob", "q1", 1);

        assertThat(m.search("the be", 10)).containsExactly("b3", "b1", "b2");
        assertThat(m.search("queen", 10)).containsExactly("q1", "q3", "q2");
    }

    @Test
    void shouldBreakTitleTiesCaseInsensitivelyThenById() {
        MusicService m = service();
        m.addSong("a1", "Banana", "Fruit", 100);
        m.addSong("z1", "apple", "Fruit", 100);
        m.addSong("e2", "Echo", "Zed", 100);
        m.addSong("e1", "echo", "Zed", 100);

        assertThat(m.search("fruit", 10)).containsExactly("z1", "a1");
        assertThat(m.search("zed", 10)).containsExactly("e1", "e2");
    }

    @Test
    void shouldLimitSearchToKAndRejectBadSearchInput() {
        MusicService m = catalogue();

        assertThat(m.search("queen", 2)).containsExactly("q3", "q1");
        assertThat(m.search("nothing matches", 5)).isEmpty();
        assertThatThrownBy(() -> m.search("queen", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.search("queen", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.search("   ", 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.search(null, 5)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldCountPlaysGloballyAndPerUser() {
        MusicService m = catalogue();
        plays(m, "alice", "q1", 2);
        plays(m, "bob", "q1", 1);

        assertThat(m.playCount("q1")).isEqualTo(3);
        assertThat(m.userPlayCount("alice", "q1")).isEqualTo(2);
        assertThat(m.userPlayCount("bob", "q1")).isEqualTo(1);
        assertThat(m.userPlayCount("alice", "q2")).isZero();
        assertThat(m.playCount("q2")).isZero();
    }

    @Test
    void shouldRejectUnknownUsersAndSongsWithoutChangingState() {
        MusicService m = catalogue();

        assertThatThrownBy(() -> m.play("ghost", "q1")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.play("alice", "nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.like("ghost", "q1")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.like("alice", "nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.unlike("alice", "nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.playCount("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.songLikes("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.recentlyPlayed("ghost")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.enqueue("alice", "nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.playNext("ghost")).isInstanceOf(IllegalArgumentException.class);
        assertThat(m.playCount("q1")).isZero();
        assertThat(m.recentlyPlayed("alice")).isEmpty();
    }

    @Test
    void shouldKeepRecentHistoryMostRecentFirstWithoutDuplicates() {
        MusicService m = catalogue();
        m.play("alice", "q1");
        m.play("alice", "q2");
        m.play("alice", "q3");
        m.play("alice", "q1");

        assertThat(m.recentlyPlayed("alice")).containsExactly("q1", "q3", "q2");
        assertThat(m.recentlyPlayed("bob")).isEmpty();
    }

    @Test
    void shouldKeepOnlyTheTenMostRecentDistinctSongs() {
        MusicService m = service();
        m.registerUser("u");
        for (int i = 0; i < 12; i++) {
            m.addSong("g" + i, "Track " + i, "Band", 60);
        }
        for (int i = 0; i < 12; i++) {
            m.play("u", "g" + i);
        }
        assertThat(m.recentlyPlayed("u"))
                .containsExactly("g11", "g10", "g9", "g8", "g7", "g6", "g5", "g4", "g3", "g2");

        m.play("u", "g5");
        assertThat(m.recentlyPlayed("u"))
                .containsExactly("g5", "g11", "g10", "g9", "g8", "g7", "g6", "g4", "g3", "g2");

        m.play("u", "g0");
        assertThat(m.recentlyPlayed("u"))
                .containsExactly("g0", "g5", "g11", "g10", "g9", "g8", "g7", "g6", "g4", "g3");
        assertThat(m.userPlayCount("u", "g2")).isEqualTo(1);
    }

    @Test
    void shouldCountLikesPerUserAndStayIdempotent() {
        MusicService m = catalogue();
        m.like("alice", "q1");
        m.like("alice", "q1");
        assertThat(m.songLikes("q1")).isEqualTo(1);

        m.like("bob", "q1");
        assertThat(m.songLikes("q1")).isEqualTo(2);
        assertThat(m.songLikes("q2")).isZero();

        m.unlike("alice", "q1");
        m.unlike("alice", "q1");
        m.unlike("alice", "q2");
        assertThat(m.songLikes("q1")).isEqualTo(1);

        m.like("alice", "q1");
        assertThat(m.songLikes("q1")).isEqualTo(2);
    }

    @Test
    void shouldCreatePlaylistsWithUniqueNamesPerUser() {
        MusicService m = catalogue();
        String a = m.createPlaylist("alice", "road trip");
        String b = m.createPlaylist("alice", "gym");
        String c = m.createPlaylist("bob", "road trip");

        assertThat(List.of(a, b, c)).doesNotHaveDuplicates();
        assertThat(m.playlist("alice", a)).isEmpty();
        assertThatThrownBy(() -> m.createPlaylist("alice", "road trip")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.createPlaylist("alice", " ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.createPlaylist("ghost", "x")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldKeepPlaylistOrderAndRejectDuplicateAdds() {
        MusicService m = catalogue();
        String p = m.createPlaylist("alice", "mix");
        m.addToPlaylist("alice", p, "q2");
        m.addToPlaylist("alice", p, "b1");
        m.addToPlaylist("alice", p, "q1");

        assertThat(m.playlist("alice", p)).containsExactly("q2", "b1", "q1");
        assertThatThrownBy(() -> m.addToPlaylist("alice", p, "b1")).isInstanceOf(IllegalArgumentException.class);
        assertThat(m.playlist("alice", p)).containsExactly("q2", "b1", "q1");

        m.removeFromPlaylist("alice", p, "b1");
        assertThat(m.playlist("alice", p)).containsExactly("q2", "q1");
        m.addToPlaylist("alice", p, "b1");
        assertThat(m.playlist("alice", p)).containsExactly("q2", "q1", "b1");
    }

    @Test
    void shouldRejectBadPlaylistEdits() {
        MusicService m = catalogue();
        String p = m.createPlaylist("alice", "mix");

        assertThatThrownBy(() -> m.addToPlaylist("alice", p, "nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.removeFromPlaylist("alice", p, "q1")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addToPlaylist("alice", "pl-unknown", "q1"))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.playlist("alice", "pl-unknown")).isInstanceOf(IllegalArgumentException.class);
        assertThat(m.playlist("alice", p)).isEmpty();
    }

    @Test
    void shouldHidePlaylistsFromOtherUsers() {
        MusicService m = catalogue();
        String p = m.createPlaylist("alice", "mix");
        m.addToPlaylist("alice", p, "q1");

        assertThatThrownBy(() -> m.playlist("bob", p)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addToPlaylist("bob", p, "q2")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.deletePlaylist("bob", p)).isInstanceOf(IllegalArgumentException.class);
        assertThat(m.playlist("alice", p)).containsExactly("q1");
    }

    @Test
    void shouldDeletePlaylistAndFreeItsName() {
        MusicService m = catalogue();
        String p = m.createPlaylist("alice", "mix");
        m.addToPlaylist("alice", p, "q1");
        m.deletePlaylist("alice", p);

        assertThatThrownBy(() -> m.playlist("alice", p)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.deletePlaylist("alice", p)).isInstanceOf(IllegalArgumentException.class);
        String again = m.createPlaylist("alice", "mix");
        assertThat(again).isNotEqualTo(p);
        assertThat(m.playlist("alice", again)).isEmpty();
    }

    @Test
    void shouldPlayQueuedSongsInOrderAndCountThemAsPlays() {
        MusicService m = catalogue();
        m.enqueue("alice", "q1");
        m.enqueue("alice", "q2");
        m.enqueue("alice", "q1");

        assertThat(m.playNext("alice")).isEqualTo(Optional.of("q1"));
        assertThat(m.recentlyPlayed("alice")).containsExactly("q1");
        assertThat(m.playNext("alice")).isEqualTo(Optional.of("q2"));
        assertThat(m.playNext("alice")).isEqualTo(Optional.of("q1"));
        assertThat(m.playNext("alice")).isEmpty();

        assertThat(m.playCount("q1")).isEqualTo(2);
        assertThat(m.userPlayCount("alice", "q1")).isEqualTo(2);
        assertThat(m.recentlyPlayed("alice")).containsExactly("q1", "q2");
        assertThat(m.playNext("bob")).isEmpty();
        assertThat(m.recentlyPlayed("bob")).isEmpty();
    }

    @Test
    void shouldKeepQueuesSeparatePerUser() {
        MusicService m = catalogue();
        m.enqueue("alice", "q1");
        m.enqueue("bob", "q2");

        assertThat(m.playNext("bob")).isEqualTo(Optional.of("q2"));
        assertThat(m.playNext("bob")).isEmpty();
        assertThat(m.playNext("alice")).isEqualTo(Optional.of("q1"));
    }

    @Test
    void shouldListTopSongsByPlayCountThenTitle() {
        MusicService m = catalogue();
        assertThat(m.topSongs(5)).isEmpty();

        plays(m, "alice", "q1", 2);
        plays(m, "bob", "q1", 1);
        plays(m, "bob", "b1", 3);
        m.play("alice", "q2");

        assertThat(m.topSongs(10)).containsExactly("q1", "b1", "q2");
        assertThat(m.topSongs(2)).containsExactly("q1", "b1");
        assertThatThrownBy(() -> m.topSongs(0)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRecommendUnplayedSongsOfPlayedArtists() {
        MusicService m = catalogue();
        plays(m, "alice", "q1", 2);
        m.play("alice", "q2");
        m.play("alice", "b1");
        plays(m, "bob", "b3", 2);

        assertThat(m.recommendations("alice", 10)).containsExactly("q3", "b3", "b2");
        assertThat(m.recommendations("alice", 2)).containsExactly("q3", "b3");
        assertThat(m.recommendations("bob", 10)).containsExactly("b1", "b2");
    }

    @Test
    void shouldRankArtistsBeforeGlobalPopularityInRecommendations() {
        MusicService m = catalogue();
        plays(m, "alice", "b1", 3);
        m.play("alice", "q1");
        plays(m, "bob", "q3", 5);

        assertThat(m.recommendations("alice", 10)).containsExactly("b2", "b3", "q3", "q2");
    }

    @Test
    void shouldRecommendNothingForNewUsersOrFullyPlayedArtists() {
        MusicService m = catalogue();
        assertThat(m.recommendations("alice", 5)).isEmpty();

        m.play("alice", "j1");
        assertThat(m.recommendations("alice", 5)).isEmpty();
        assertThatThrownBy(() -> m.recommendations("alice", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.recommendations("ghost", 3)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldCountQueuedPlaysTowardsRecommendationsAndTreatArtistNamesCaseInsensitively() {
        MusicService m = catalogue();
        m.addSong("q4", "Radio Ga Ga", " queen ", 343);
        m.enqueue("alice", "q4");
        m.playNext("alice");

        assertThat(m.recommendations("alice", 10)).containsExactly("q1", "q2", "q3");
    }

    @Test
    void shouldCountEveryConcurrentPlay() {
        MusicService m = catalogue();
        Concurrently.run(16, t -> {
            for (int i = 0; i < 500; i++) {
                m.play(t % 2 == 0 ? "alice" : "bob", "q1");
            }
        });

        assertThat(m.playCount("q1")).isEqualTo(8000);
        assertThat(m.userPlayCount("alice", "q1")).isEqualTo(4000);
        assertThat(m.userPlayCount("bob", "q1")).isEqualTo(4000);
        assertThat(m.recentlyPlayed("alice")).containsExactly("q1");
    }

    @Test
    void shouldPopEachQueuedSongExactlyOnceUnderContention() {
        MusicService m = catalogue();
        for (int i = 0; i < 4000; i++) {
            m.enqueue("alice", i % 2 == 0 ? "q1" : "q2");
        }
        List<Integer> popped = Concurrently.collect(8, t -> {
            int n = 0;
            while (m.playNext("alice").isPresent()) {
                n++;
            }
            return n;
        });

        assertThat(popped.stream().mapToInt(Integer::intValue).sum()).isEqualTo(4000);
        assertThat(m.playCount("q1")).isEqualTo(2000);
        assertThat(m.playCount("q2")).isEqualTo(2000);
    }

    @Test
    void shouldAllowExactlyOneWinnerForConcurrentDuplicates() {
        MusicService m = catalogue();
        AtomicInteger users = new AtomicInteger();
        AtomicInteger songs = new AtomicInteger();
        AtomicInteger lists = new AtomicInteger();
        Concurrently.run(8, t -> {
            try {
                m.registerUser("carol");
                users.incrementAndGet();
            } catch (IllegalArgumentException ignored) {
                // lost the race
            }
            try {
                m.addSong("new", "New Song", "New Artist", 100);
                songs.incrementAndGet();
            } catch (IllegalArgumentException ignored) {
                // lost the race
            }
            try {
                m.createPlaylist("alice", "same");
                lists.incrementAndGet();
            } catch (IllegalArgumentException ignored) {
                // lost the race
            }
        });

        assertThat(users).hasValue(1);
        assertThat(songs).hasValue(1);
        assertThat(lists).hasValue(1);
    }

    @Test
    void shouldCountConcurrentLikesFromDifferentUsers() {
        MusicService m = service();
        m.addSong("s", "Song", "Artist", 100);
        List<String> ids = new ArrayList<>();
        for (int i = 0; i < 200; i++) {
            ids.add("u" + i);
            m.registerUser("u" + i);
        }
        Concurrently.run(8, t -> {
            for (int i = 0; i < 200; i++) {
                m.like(ids.get(i), "s");
            }
        });

        assertThat(m.songLikes("s")).isEqualTo(200);
    }
}
