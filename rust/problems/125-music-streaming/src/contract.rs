//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//! A failed call leaves the state unchanged.

#[derive(Debug, PartialEq, Eq)]
pub enum MusicError {
    UnknownUser,
    UnknownSong,
    UnknownPlaylist,
    /// duplicate song id, user id, playlist name for the same user, or song already in the playlist
    AlreadyExists,
    /// remove_from_playlist for a song that is not in the playlist
    NotInPlaylist,
    /// blank text, non-positive duration, non-positive k
    InvalidArgument,
}

pub trait MusicService: Send + Sync {
    /// Adds a song to the catalogue. `AlreadyExists` if the id exists, `InvalidArgument` for blank text or `duration_sec == 0`.
    fn add_song(
        &self,
        song_id: &str,
        title: &str,
        artist: &str,
        duration_sec: u32,
    ) -> Result<(), MusicError>;

    /// Registers a user. `AlreadyExists` if the id is already registered, `InvalidArgument` if blank.
    fn register_user(&self, user_id: &str) -> Result<(), MusicError>;

    /// Song ids matching the query, best first. Case-insensitive. A song matches when EVERY whitespace-separated
    /// query token is a prefix of some whitespace-separated word of the title or of the artist name. Ranking: global
    /// play count descending, then title ascending (case-insensitive), then song id ascending. At most `k` ids.
    /// `InvalidArgument` if the query is blank or `k <= 0`.
    fn search(&self, query: &str, k: i32) -> Result<Vec<String>, MusicError>;

    /// Counts one play of the song for the user: global count, the user's own count and the user's recent history.
    fn play(&self, user_id: &str, song_id: &str) -> Result<(), MusicError>;

    /// Total number of plays of the song by everybody.
    fn play_count(&self, song_id: &str) -> Result<u64, MusicError>;

    /// Number of times this user played this song.
    fn user_play_count(&self, user_id: &str, song_id: &str) -> Result<u64, MusicError>;

    /// The user's last distinct songs, most recent first, at most 10; replaying a song moves it to the front.
    fn recently_played(&self, user_id: &str) -> Result<Vec<String>, MusicError>;

    /// The user likes the song. Idempotent.
    fn like(&self, user_id: &str, song_id: &str) -> Result<(), MusicError>;

    /// The user withdraws the like. Idempotent (also when there was no like).
    fn unlike(&self, user_id: &str, song_id: &str) -> Result<(), MusicError>;

    /// Number of distinct users currently liking the song.
    fn song_likes(&self, song_id: &str) -> Result<usize, MusicError>;

    /// Creates an empty playlist owned by the user and returns its id. `AlreadyExists` if the user already has a
    /// playlist with that name; `InvalidArgument` for a blank name.
    fn create_playlist(&self, user_id: &str, name: &str) -> Result<String, MusicError>;

    /// Appends the song to the end of the playlist. `AlreadyExists` if the song is already in it.
    fn add_to_playlist(
        &self,
        user_id: &str,
        playlist_id: &str,
        song_id: &str,
    ) -> Result<(), MusicError>;

    /// Removes the song. `NotInPlaylist` if it is not there. The other songs keep their order.
    fn remove_from_playlist(
        &self,
        user_id: &str,
        playlist_id: &str,
        song_id: &str,
    ) -> Result<(), MusicError>;

    /// Song ids in playlist order. A playlist is only visible to its owner: for anybody else it is `UnknownPlaylist`.
    fn playlist(&self, user_id: &str, playlist_id: &str) -> Result<Vec<String>, MusicError>;

    /// Deletes the playlist; its name becomes free again for that user.
    fn delete_playlist(&self, user_id: &str, playlist_id: &str) -> Result<(), MusicError>;

    /// Appends the song to the end of the user's play queue (duplicates allowed).
    fn enqueue(&self, user_id: &str, song_id: &str) -> Result<(), MusicError>;

    /// Removes the oldest queued song and plays it (exactly like `play`). Empty queue: `Ok(None)`, nothing changes.
    fn play_next(&self, user_id: &str) -> Result<Option<String>, MusicError>;

    /// Songs with at least one play, most played first (ties: title ascending case-insensitive, then id). At most `k`.
    /// `InvalidArgument` if `k <= 0`.
    fn top_songs(&self, k: i32) -> Result<Vec<String>, MusicError>;

    /// Songs by artists the user has played, that the user has never played. Artists are ranked by how many plays the
    /// user has made of their songs (artist names compare case-insensitively, ignoring surrounding spaces). Songs rank
    /// by that artist play count descending, then global play count descending, then song id ascending. At most `k`.
    /// `InvalidArgument` if `k <= 0`.
    fn recommendations(&self, user_id: &str, k: i32) -> Result<Vec<String>, MusicError>;
}
