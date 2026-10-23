# 63 · Flyweight (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `MarkerStyle` (the flyweight), `StyleFactory`, `Marker` and `MapLayer` are stubs in `src/lib.rs`; `MapError` is provided. |

## Problem

A map app shows tens of thousands of pins. Each pin has a heavy icon bitmap (4096 bytes here; megabytes in real
life) that depends only on its **shape and colour**, plus a small position and label. Allocating a bitmap per pin
exhausts memory. Share the icons: however many markers a layer holds, only one icon exists per distinct shape/colour
pair, and many threads may ask for icons at the same time.

## Terms

- **Flyweight** (`MarkerStyle`): the shared, shape+colour-specific value holding the heavy data, handed out as `Arc<MarkerStyle>`.
- **Per-marker data**: latitude, longitude and label, which differ for every pin and stay on the `Marker`.
- **Blank**: empty, or only whitespace.

## Requirements

1. `MarkerStyle` — the flyweight: `shape()`, `color()`, `bitmap_size()` (always 4096). Styles come from the factory.
2. `StyleFactory::get(shape, color)` returns the **same `Arc`** for the same pair (exact, case-sensitive match); `("pin","red")`, `("pin","Red")` and `("flag","red")` are three different styles. Blank shape/colour → `Err(MapError::InvalidStyle)`. `instances_created()` counts styles created so far (a repeat request creates nothing; a rejected request creates nothing).
3. **Thread-safe:** the factory is `Send + Sync` and 32 threads asking for the same 5 keys together must still create exactly 5 styles, and every thread must get the same `Arc` (same pointer) for a given key.
4. `Marker::new(lat, lng, label, style)` keeps position, label and the style, exposed as `lat()`, `lng()`, `label()`, `style()`. Latitude `-90..=90` and longitude `-180..=180` inclusive, otherwise `Err(MapError::InvalidPosition)`. A marker holds a clone of the style `Arc`, not a bitmap of its own.
5. `MapLayer::new(Arc<StyleFactory>).add_marker(lat, lng, label, shape, color)` gets the style from the factory and appends a marker. Invalid arguments leave the layer unchanged and create no style: out-of-range position → `Err(InvalidPosition)` (an out-of-range position must not cause `instances_created()` to grow), blank shape/colour → `Err(InvalidStyle)`. `marker_count()`; `markers()` is a slice in insertion order; `render()` returns one line per marker, in order: `"<shape>/<color> @ <lat>,<lng> <label>"` using Rust's default `{}` formatting for the floats (so `-40.0` prints as `-40`), e.g. `pin/red @ 12.5,-3.25 Cafe`.

## Examples

| Call | Result |
|---|---|
| `get("pin","red")` twice | same `Arc` (`Arc::ptr_eq`); `instances_created()` = 1 |
| then `get("pin","blue")`, `get("flag","red")`, `get("pin","Red")` | three more distinct `Arc`s; `instances_created()` = 4 |
| 10,000 `add_marker` calls cycling over `(pin,red)`, `(pin,blue)`, `(flag,green)` | `marker_count()` = 10000, `instances_created()` = 3, markers share just 3 distinct style pointers |
| `add_marker(12.5, -3.25, "Cafe", "pin", "red")`, `add_marker(-40.0, 100.5, "Harbour", "flag", "blue")` | `render()` = `["pin/red @ 12.5,-3.25 Cafe", "flag/blue @ -40,100.5 Harbour"]` |
| `add_marker(91.0, 0.0, "x", "pin", "red")`, `add_marker(0.0, -181.0, ...)`, `add_marker(0.0, 0.0, "x", "", "red")` | `Err(InvalidPosition)`, `Err(InvalidPosition)`, `Err(InvalidStyle)`; `marker_count()` = 0, `instances_created()` = 0 |

## Running

```bash
cargo test -p p63_flyweight     # from rust/ — tests FAIL until you implement the stubs
cargo play run 63
```
