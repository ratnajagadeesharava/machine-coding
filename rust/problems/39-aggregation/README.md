# 39 · Aggregation (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — `Song` and `Playlist` stubs and `MusicError` are provided; you write the bodies. |

## Problem

You are building the playlist feature of a music app. Songs exist on their own (in the catalogue); a playlist is just
a named list of shared references to songs, in the order the user added them. The same song can sit in many
playlists, renaming a song shows up everywhere it appears, and dropping or clearing a playlist must never destroy a
song.

## Glossary

- **same instance**: the very same allocation, compared with `Rc::ptr_eq`, as opposed to another `Song` that merely has an equal title and length.
- **seconds**: a song's length in whole seconds (`u32`).
- **strong count**: `Rc::strong_count`, the number of live `Rc` handles to a song.

## Requirements

1. `Song::new(title, seconds)`: blank title gives `Err(MusicError::InvalidTitle)`; zero seconds gives `Err(MusicError::InvalidDuration)`. Accessors `title()` and `seconds()`. `rename(&self, new_title)` takes `&self`, changes the title, and rejects a blank title with `Err(InvalidTitle)` (title unchanged).
2. `Playlist::new(name)`, with `name()`. `Playlist::add(Rc<Song>)` shares the song (never a copy) and returns `true`. Adding an `Rc` that is already present (`Rc::ptr_eq`) returns `false` and changes nothing. Two different songs that look alike are both allowed.
3. `songs()` returns the songs in insertion order.
4. `total_seconds()` sums the lengths (0 for an empty playlist).
5. `remove(&Rc<Song>)` returns `true` if that instance was present and removed, `false` otherwise. `clear()` empties the playlist. Both, and dropping the playlist, only release that playlist's reference: the song stays alive and usable and can be added to another playlist. The tests watch `Rc::strong_count` to check this.
6. Renaming a song is visible through every playlist that contains it.

## Examples

```rust
let a = Rc::new(Song::new("A", 180).unwrap());
let b = Rc::new(Song::new("B", 200).unwrap());
let mut p = Playlist::new("Road trip");
p.add(a.clone());       // true
p.add(b.clone());       // true
p.add(a.clone());       // false (already there)
p.total_seconds();      // 380

let twin = Rc::new(Song::new("A", 180).unwrap());
p.add(twin);            // true: a different allocation, even though it looks like a

// reference counting
let s = Rc::new(Song::new("S", 60).unwrap());      // strong_count == 1
let mut one = Playlist::new("one");
let mut two = Playlist::new("two");
one.add(s.clone()); two.add(s.clone());            // strong_count == 3
drop(one);                                         // strong_count == 2
two.clear();                                       // strong_count == 1, s still usable
```

## Edge cases

- `Song::new(" ", 10)` gives `Err(InvalidTitle)`, `Song::new("T", 0)` gives `Err(InvalidDuration)`, and `Song::new("T", 5).unwrap().rename("")` gives `Err(InvalidTitle)`.
- Removing a song that is not in the playlist returns `false` and changes no counts.

## Running

```bash
cargo test -p p39_aggregation     # from rust/ — tests FAIL until you implement the stubs
cargo play run 39
```
