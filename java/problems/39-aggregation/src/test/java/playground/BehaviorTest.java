package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import org.junit.jupiter.api.Test;
import solution.Playlist;
import solution.Song;

class BehaviorTest {

    @Test
    void shouldKeepSongsInOrderAndSumTheirLength() {
        Playlist p = new Playlist("Road trip");
        Song a = new Song("A", 180);
        Song b = new Song("B", 200);

        assertThat(p.add(a)).isTrue();
        assertThat(p.add(b)).isTrue();

        assertThat(p.name()).isEqualTo("Road trip");
        assertThat(p.songs()).containsExactly(a, b);
        assertThat(p.totalSeconds()).isEqualTo(380);
    }

    @Test
    void shouldHoldTheSameInstancesNotCopies() {
        Playlist p = new Playlist("P");
        Song song = new Song("Original", 100);
        p.add(song);

        song.rename("Renamed");

        assertThat(p.songs().get(0)).isSameAs(song);
        assertThat(p.songs().get(0).title()).isEqualTo("Renamed");
    }

    @Test
    void shouldLetOneSongBelongToSeveralPlaylists() {
        Song shared = new Song("Shared", 90);
        Playlist one = new Playlist("one");
        Playlist two = new Playlist("two");
        one.add(shared);
        two.add(shared);

        shared.rename("Edited once");

        assertThat(one.songs().get(0).title()).isEqualTo("Edited once");
        assertThat(two.songs().get(0)).isSameAs(one.songs().get(0));
    }

    @Test
    void shouldLetSongsSurviveWhenThePlaylistIsEmptiedOrRemoved() {
        Song song = new Song("Survivor", 60);
        Playlist first = new Playlist("first");
        first.add(song);
        first.clear();
        Playlist second = new Playlist("second");

        assertThat(first.songs()).isEmpty();
        assertThat(song.title()).isEqualTo("Survivor");
        assertThat(second.add(song)).isTrue();
        assertThat(second.totalSeconds()).isEqualTo(60);
    }

    @Test
    void shouldRemoveOnlyFromThePlaylistNotFromOthers() {
        Song song = new Song("S", 10);
        Playlist one = new Playlist("one");
        Playlist two = new Playlist("two");
        one.add(song);
        two.add(song);

        assertThat(one.remove(song)).isTrue();
        assertThat(one.remove(song)).isFalse();

        assertThat(one.songs()).isEmpty();
        assertThat(two.songs()).containsExactly(song);
        assertThat(song.seconds()).isEqualTo(10);
    }

    @Test
    void shouldRefuseTheSameInstanceTwiceButAllowEqualLookingSongs() {
        Playlist p = new Playlist("P");
        Song a = new Song("Same", 10);
        Song twin = new Song("Same", 10);

        assertThat(p.add(a)).isTrue();
        assertThat(p.add(a)).isFalse();
        assertThat(p.add(twin)).isTrue();
        assertThat(p.songs()).hasSize(2);
    }

    @Test
    void shouldRejectInvalidSongsAndNullAdds() {
        Playlist p = new Playlist("P");

        assertThatThrownBy(() -> new Song(" ", 10)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Song("T", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.add(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Song("T", 5).rename("")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNotLetCallersModifyThePlaylistThroughTheReturnedList() {
        Playlist p = new Playlist("P");
        p.add(new Song("A", 1));

        assertThatThrownBy(() -> p.songs().add(new Song("B", 1))).isInstanceOf(UnsupportedOperationException.class);
        assertThat(p.songs()).hasSize(1);
    }
}
