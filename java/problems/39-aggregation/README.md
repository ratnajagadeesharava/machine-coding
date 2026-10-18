# 39 · Aggregation (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — two stub classes, `Song` and `Playlist`. Bodies throw `UnsupportedOperationException`. |

> **Concept:** aggregation is a *has-a* relationship where the whole **does not own** its parts. The parts are
> created elsewhere, can be shared between wholes, and **outlive** the whole.

## Problem

A music app has songs and playlists. A playlist is just a list of references to songs. Deleting or clearing a
playlist must never delete a song, and the same song can sit in many playlists.

## Requirements

1. `Song(title, seconds)`: blank title or `seconds <= 0` → `IllegalArgumentException`. `rename(newTitle)` changes the title (blank → `IllegalArgumentException`).
2. `Playlist.add(song)` stores **the very same instance** (no copying). Adding an instance that is already present returns `false`; `null` → `IllegalArgumentException`. Two different `Song` objects that merely look alike are both allowed.
3. `songs()` returns the songs in insertion order; callers cannot modify the returned list.
4. `totalSeconds()` sums the songs' lengths.
5. `remove(song)` / `clear()` only affect that playlist. The song and other playlists are untouched.
6. Renaming a song is visible through every playlist that contains it.

## Constraints

- Identity, not equality: do not override `equals` on `Song` (the tests use `isSameAs`).

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
