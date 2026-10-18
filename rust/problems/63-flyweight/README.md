# 63 · Flyweight (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `MarkerStyle` (the flyweight), `StyleFactory`, `Marker` and `MapLayer` are stubs in `src/lib.rs`; `MapError` is provided. |

> **Pattern:** *Flyweight* — share the heavy, **intrinsic** (identical-for-everyone) part of many objects, and keep
> only the light, **extrinsic** (per-object) part in each one. In Rust the shared part is an `Arc<MarkerStyle>`:
> 10,000 markers hold 10,000 cheap pointers to 3 icons.

## Problem

A map app shows tens of thousands of pins. Each pin has a heavy icon bitmap (4096 bytes here; megabytes in real
life) that depends only on its **shape and colour**, plus a small position and label. Allocating a bitmap per pin
exhausts memory. Share the icons.

## Requirements

1. `MarkerStyle` — the flyweight: `shape()`, `color()`, `bitmap_size()` (always 4096; allocate a real `Vec<u8>` of that size once per style). Give it a private constructor so only the factory can create one.
2. `StyleFactory::get(shape, color)` returns the **same `Arc`** for the same pair (exact, case-sensitive match); `("pin","red")`, `("pin","Red")` and `("flag","red")` are three different styles. Blank shape/colour → `Err(MapError::InvalidStyle)`. `instances_created()` counts styles created.
3. **Thread-safe:** the factory is `Send + Sync` and 32 threads asking for the same 5 keys together must still create exactly 5 styles (hint: a `Mutex<HashMap<..>>` and an `AtomicUsize` or the map length).
4. `Marker::new(lat, lng, label, style)` keeps position, label and the shared `Arc`. Latitude `-90..=90`, longitude `-180..=180`, otherwise `Err(MapError::InvalidPosition)`.
5. `MapLayer::new(Arc<StyleFactory>).add_marker(lat, lng, label, shape, color)` obtains the style from the factory; an invalid argument leaves the layer and the factory unchanged (validate before creating a style). `render()` → `"<shape>/<color> @ <lat>,<lng> <label>"` per marker using Rust's default `{}` formatting for the floats (so `-40.0` prints as `-40`).

## Constraints (what the tests catch)

- 10,000 markers over 3 kinds → `instances_created() == 3`, and the markers point at exactly 3 distinct styles (compared with `Arc::as_ptr`).

## Running

```bash
cargo test -p p63_flyweight     # from rust/ — tests FAIL until you implement the stubs
cargo play run 63
```
