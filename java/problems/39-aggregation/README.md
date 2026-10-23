# 39 · Aggregation (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — `Song` and `Playlist` stubs with the signatures; you write the bodies. |

## Problem

You are building the playlist feature of a music app. Songs exist on their own (in the catalogue); a playlist is just
a named list of references to songs, in the order the user added them. The same song can sit in many playlists,
renaming a song shows up everywhere it appears, and deleting or clearing a playlist must never delete a song.

## Glossary

- **same instance**: the very same object (reference identity, `==`), as opposed to another `Song` that merely has an equal title and length.
- **seconds**: a song's length in whole seconds.

## Requirements

1. `Song(title, seconds)`: blank title or `seconds <= 0` throws `IllegalArgumentException`. Accessors `title()` and `seconds()`. `rename(newTitle)` changes the title (blank throws `IllegalArgumentException`, title unchanged).
2. `new Playlist(name)`, with `name()`. `Playlist.add(song)` stores **the very same instance** (no copying) and returns `true`. Adding an instance that is already present returns `false` and changes nothing; `null` throws `IllegalArgumentException`. Two different `Song` objects that merely look alike (same title and length) are both allowed.
3. `songs()` returns the songs in insertion order; callers cannot modify the returned list (attempting to throws `UnsupportedOperationException`).
4. `totalSeconds()` sums the songs' lengths (0 for an empty playlist).
5. `remove(song)` returns `true` if that instance was in the playlist and removed it, `false` otherwise. `clear()` empties the playlist. Both only affect that playlist: the song itself and other playlists are untouched, and the song can be added to another playlist afterwards.
6. Renaming a song is visible through every playlist that contains it.

## Examples

```java
Song a = new Song("A", 180), b = new Song("B", 200);
Playlist p = new Playlist("Road trip");
p.add(a);               // true
p.add(b);               // true
p.add(a);               // false (already there)
p.songs();              // [a, b]
p.totalSeconds();       // 380

Song twin = new Song("A", 180);
p.add(twin);            // true: a different object, even though it looks like a
p.songs().size();       // 3

Song shared = new Song("Shared", 90);
Playlist one = new Playlist("one"), two = new Playlist("two");
one.add(shared); two.add(shared);
shared.rename("Edited once");
two.songs().get(0).title();  // "Edited once"
one.remove(shared);          // true
one.remove(shared);          // false
two.songs();                 // [shared] still there
```

## Edge cases

- `new Song(" ", 10)` and `new Song("T", 0)` throw `IllegalArgumentException`; so does `new Song("T", 5).rename("")`.
- `p.songs().add(...)` throws `UnsupportedOperationException` and the playlist is unchanged.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
