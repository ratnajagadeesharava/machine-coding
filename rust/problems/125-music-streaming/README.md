# 125 · Music Streaming and Search (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

The core of a Spotify-like service: a catalogue of songs that people can search by typing a few letters of a title or an artist, play, like, collect in playlists and line up in a queue. The service keeps counters (how often each song was played, overall and by each user), a short listening history per user, and uses them to rank search results, list the most played songs and recommend more songs by the artists a user listens to. There is no real audio and no wall clock: "playing" a song just means the call `play` happened, and the order of calls is the order of events. Many listeners play at once, and no play may ever be lost.

## Requirements

**Catalogue and users**

1. `addSong(songId, title, artist, durationSec)` adds a song. A duplicate song id, blank id/title/artist, or a duration that is not positive is an error. Song ids are case-sensitive.
2. `registerUser(userId)` registers a user. A blank or already registered id is an error. Every other operation that names a user requires a registered one.

**Search**

3. `search(query, k)` returns at most `k` song ids. The query is split on whitespace into tokens (any amount of spaces/tabs around and between them). A song matches when **every** token is a prefix of **some word** of the title **or** of some word of the artist name. Words are the whitespace-separated pieces of the title and of the artist; punctuation stays part of the word (`Don't` is the word `don't`). Comparison ignores case. Different tokens may match different words, the title and the artist, and may even match the same word.
4. Ranking: global play count descending, then title ascending (compared in lower case), then song id ascending. Songs that were never played are matches like any other.
5. A blank (or null) query or `k <= 0` is an error. No match is an empty result, not an error.

**Playback, history and likes**

6. `play(userId, songId)` records one play: it increments the song's global count and the user's own count for that song, and puts the song at the front of the user's recent history. Unknown user or song is an error and changes nothing.
7. The recent history lists distinct songs, most recent first, at most 10. Playing a song that is already in the history moves it to the front (no duplicate); when an 11th distinct song arrives the oldest one drops out. A song that dropped out keeps its play counts.
8. `like(userId, songId)` / `unlike(userId, songId)` are idempotent. Likes are per user; `songLikes(songId)` is the number of distinct users currently liking the song. `unlike` without a like is not an error. Likes do not influence ranking.
9. `playCount(songId)` and `userPlayCount(userId, songId)` expose the counters (0 when never played).

**Playlists**

10. `createPlaylist(userId, name)` returns a new playlist id that is unique across the whole service. A playlist belongs to its creator. A blank name, or a name the same user already uses, is an error (two different users may use the same name; names are case-sensitive).
11. `addToPlaylist` appends a song at the end. Adding a song that is already in the playlist is an **error** (the playlist is unchanged). `removeFromPlaylist` removes a song; removing a song that is not in the playlist is an error; the remaining songs keep their order. Adding a removed song again puts it at the end.
12. `playlist(userId, playlistId)` returns the song ids in order. A playlist can only be read or changed by its owner: for any other registered user it behaves as an unknown playlist (error). `deletePlaylist` removes it; afterwards its id is unknown and the owner may reuse the name. Playing songs does not touch playlists, and playlists do not touch counters.

**Queue**

13. `enqueue(userId, songId)` appends a song to the user's own queue (the same song may be queued several times). `playNext(userId)` removes the oldest queued song and plays it exactly like `play`, then returns its id. With an empty queue it returns an empty result (`Optional.empty()` / `None`) and changes nothing. Queues are per user.

**Rankings**

14. `topSongs(k)` returns at most `k` songs that have at least one play, ordered like the search ranking (plays descending, title ascending in lower case, song id ascending). `k <= 0` is an error.
15. `recommendations(userId, k)` returns at most `k` songs:
    - Count, for every artist, the user's plays of that artist's songs (sum of the user's own play counts). Artist names are the same artist when equal ignoring case and surrounding spaces.
    - Candidates are the songs of artists with a positive count that the user has **never played** (user play count 0).
    - Order: that artist count descending, then the song's global play count descending, then song id ascending.
    - A user without plays gets an empty result. `k <= 0` and an unknown user are errors.

## Constraints

- The service is used by many threads at once: concurrent `play` calls on the same song, concurrent `playNext` calls for one user, concurrent registrations/likes/playlist creations. Every play is counted exactly once, every queued song is popped exactly once, and a duplicate that races with another duplicate yields exactly one winner.
- A failed call (unknown id, duplicate, bad argument) changes nothing.
- Search must stay fast on a catalogue of 100,000 songs (hidden scenario: hundreds of searches within a few seconds).
- No wall clock, randomness or sleeping: everything is determined by the order of calls.

## Examples

Each block below starts from a fresh service with the same five songs and two users; the blocks do not carry over into each other.

```
addSong(q1, "Bohemian Rhapsody", "Queen", 354)    addSong(q2, "Don't Stop Me Now", "Queen", 209)
addSong(q3, "Another One Bites the Dust", "Queen", 215)    addSong(b1, "Hey Jude", "The Beatles", 431)
addSong(b2, "Let It Be", "The Beatles", 243)      registerUser(alice)   registerUser(bob)

search("QUEEN rhap", 10)  -> [q1]                  (both tokens match: artist word, title word)
search("hemian", 10)      -> []                    (a prefix of a word, not a substring)
search("queen", 10)       -> [q3, q1, q2]          (nobody played anything: title order a < b < d)
search("the be", 10)      -> [b1, b2]              ("the" = artist word, "be" = Beatles / Be)

play(alice, q2) x2, play(bob, q1)
search("queen", 10)       -> [q2, q1, q3]          (2 plays, 1 play, 0 plays)
topSongs(5)               -> [q2, q1]              (unplayed songs are not listed)
```

```
play(alice, q1), play(alice, q2), play(alice, q3), play(alice, q1)
recentlyPlayed(alice)     -> [q1, q3, q2]          (q1 moved to the front, listed once)
userPlayCount(alice, q1)  -> 2

enqueue(alice, b1), enqueue(alice, b2), enqueue(alice, b1)
playNext(alice) -> b1,  playNext(alice) -> b2,  playNext(alice) -> b1,  playNext(alice) -> empty
playCount(b1) -> 2        recentlyPlayed(alice) -> [b1, b2, q1, q3, q2]
```

```
alice: plays q1 x2, q2 x1, b1 x1        bob: plays b2 x2
Queen count for alice = 3, The Beatles count = 1
recommendations(alice, 10) -> [q3, b2]     (q3: artist 3; b2: artist 1; q1, q2, b1 already played)
recommendations(alice, 1)  -> [q3]
recommendations(bob, 10)   -> [b1]         (b1 is unplayed by bob; alice's plays do not count for bob)

createPlaylist(alice, "mix") -> p1        addToPlaylist(alice, p1, q2), (p1, b1)    playlist(alice, p1) -> [q2, b1]
addToPlaylist(alice, p1, b1)  -> error     playlist(bob, p1) -> error
removeFromPlaylist(alice, p1, q2), addToPlaylist(alice, p1, q2)   playlist(alice, p1) -> [b1, q2]
```

### Edge cases

- Tokens are all required (AND), and a token only needs to match the start of a word; `"stop don't"` finds `Don't Stop Me Now` whatever the token order. Repeated tokens (`"rock rock"`) are fine.
- Title ties compare in lower case (`apple` before `Banana`), then song id ascending.
- The history holds 10 distinct songs; replaying the oldest one moves it to the front and drops nothing, replaying a song that already dropped out re-adds it and drops the current oldest.
- A queued song counts as a play only when it is popped, not when it is enqueued. Songs that are queued twice are played twice.
- `songLikes` counts users, not clicks. Liking twice then unliking once leaves 0 likes.
- Artist identity for recommendations ignores case and surrounding spaces (`Queen` and ` queen ` are one artist). Search still works on each song's own artist text.
- Unknown ids, duplicates, blank text, `k <= 0`, a song that is not in a playlist and a foreign playlist are all errors that leave everything unchanged.

## Glossary

- **Song**: a catalogue entry with id, title, artist and duration (the duration is stored but not used by any ranking).
- **Play**: one recorded listen; counted globally and per user.
- **History**: the user's last distinct songs, most recent first, capped at 10.
- **Queue**: the user's own "up next" list; popping a song plays it.
- **Artist count**: how many plays a given user has made of one artist's songs; drives recommendations.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- `Result<_, MusicError>` with `UnknownUser`, `UnknownSong`, `UnknownPlaylist`, `AlreadyExists` (duplicate song id, user id, playlist name for that user, or a song already in the playlist), `NotInPlaylist`, `InvalidArgument` (blank text, zero duration, `k <= 0`). `k` is an `i32`, counts are `u64`, `song_likes` is `usize`, `play_next` returns `Result<Option<String>, _>`.
- Playlist ids are opaque strings; tests only use the ids returned by `createPlaylist`.
- Search words come from splitting on whitespace and lowercasing; there is no stemming, punctuation stripping or accent folding.
- Rankings are fully determined by the three keys listed, so the result order is exact.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below. It is **not** your domain model — it is the doorway. Create whatever types you like and wire them up.

```rust
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
```

Your wiring point is `create()` in `src/lib.rs`, returning `Option<Arc<dyn MusicService>>` (`None` until you implement it).

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. No clock is needed.

- Catalogue and user errors, search by title/artist prefix, multi-token AND, whitespace handling, ranking and tie-breaks, `k` handling.
- Counters, history order and cap, likes, playlists (order, duplicates, ownership, delete and name reuse), queue behaviour.
- Top songs and recommendations (artist count, global plays, id; played songs excluded; case-insensitive artists).
- Concurrency: 16 threads playing, 8 threads draining one queue, racing duplicates, concurrent likes.
- More scenarios (100,000-song catalogue, mixed concurrent traffic, re-ranking as plays accumulate) are in `interviewer/`.

## Running

```bash
cargo test -p p125_music_streaming
STRICT=1 cargo test -p p125_music_streaming   # fail instead of skip while unimplemented
cargo test -p p125_music_streaming --features hidden   # interviewer scenarios
cargo play run 125              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`sut!`, `run_concurrently`): `rust/support`.
