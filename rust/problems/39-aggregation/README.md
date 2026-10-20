# 39 · Aggregation (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — `Song` and `Playlist` stubs and `MusicError` are provided; you write the bodies. |

## Problem

A music app has songs and playlists. A playlist is a list of references to songs. Dropping or clearing a playlist
must never destroy a song, and the same song can sit in many playlists.

## Requirements

1. `Song::new(title, seconds)`: blank title → `Err(MusicError::InvalidTitle)`; zero seconds → `Err(InvalidDuration)`. `rename(&self, ..)` takes `&self` and rejects a blank title.
2. `Playlist::add(Rc<Song>)` shares the song (never a copy). Adding a `Rc` that is already present (`Rc::ptr_eq`) returns `false`. Two different songs that look alike are both allowed.
3. `songs()` returns the songs in insertion order.
4. `total_seconds()` sums the lengths.
5. `remove(&Rc<Song>)` / `clear()` / dropping the playlist only release that playlist's references. The tests watch `Rc::strong_count` to check this.
6. Renaming a song is visible through every playlist that contains it.

## Running

```bash
cargo test -p p39_aggregation     # from rust/ — tests FAIL until you implement the stubs
cargo play run 39
```
