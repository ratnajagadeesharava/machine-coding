# 63 · Flyweight (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — four stub classes: `MarkerStyle` (the flyweight), `StyleFactory`, `Marker` and `MapLayer`. |

## Problem

A map app shows tens of thousands of pins. Each pin has a heavy icon bitmap (4096 bytes here; megabytes in real
life) that depends only on its **shape and colour**, plus a small position and label. Allocating a bitmap per pin
exhausts memory. Share the icons: however many markers a layer holds, only one icon object exists per distinct
shape/colour pair, and many threads may ask for icons at the same time.

## Terms

- **Flyweight** (`MarkerStyle`): the shared, shape+colour-specific object holding the heavy data.
- **Per-marker data**: latitude, longitude and label, which differ for every pin and stay on the `Marker`.
- **Blank**: `null`, empty, or only whitespace.

## Requirements

1. `MarkerStyle` — the flyweight: `shape()`, `color()`, `bitmapSize()` (always 4096). Callers cannot create one directly (no public constructor); styles come from the factory.
2. `StyleFactory.get(shape, color)` returns the **same instance** for the same pair (exact, case-sensitive match); `("pin","red")`, `("pin","Red")` and `("flag","red")` are three different styles. Blank (or `null`) shape/colour → `IllegalArgumentException`. `instancesCreated()` counts styles created so far (a repeat request creates nothing; a rejected request creates nothing).
3. **Thread-safe:** 32 threads asking for the same 5 keys together must still create exactly 5 styles, and every thread must get the same instance for a given key.
4. `Marker(lat, lng, label, style)` keeps position, label and the style, exposed as `lat()`, `lng()`, `label()`, `style()`. Latitude `-90..90` and longitude `-180..180` inclusive, and a non-null style, else `IllegalArgumentException`. A marker holds no bitmap of its own.
5. `MapLayer(factory).addMarker(lat, lng, label, shape, color)` gets the style from the factory and appends a marker. Invalid arguments (position out of range, blank shape/colour) → `IllegalArgumentException`, the layer is unchanged, and no style is created (an out-of-range position must not cause `instancesCreated()` to grow). `markerCount()`; `markers()` is a read-only list in insertion order (modifying it → `UnsupportedOperationException`); `render()` returns one line per marker, in order: `"<shape>/<color> @ <lat>,<lng> <label>"` with the doubles printed with Java's default `double` text (so `-40.0` stays `-40.0`), e.g. `pin/red @ 12.5,-3.25 Cafe`.

## Examples

| Call | Result |
|---|---|
| `get("pin","red")` twice | same object; `instancesCreated()` = 1 |
| then `get("pin","blue")`, `get("flag","red")`, `get("pin","Red")` | three more distinct objects; `instancesCreated()` = 4 |
| 10,000 `addMarker` calls cycling over `(pin,red)`, `(pin,blue)`, `(flag,green)` | `markerCount()` = 10000, `instancesCreated()` = 3, markers share just 3 distinct style objects |
| `addMarker(12.5, -3.25, "Cafe", "pin", "red")`, `addMarker(-40.0, 100.5, "Harbour", "flag", "blue")` | `render()` = `["pin/red @ 12.5,-3.25 Cafe", "flag/blue @ -40.0,100.5 Harbour"]` |
| `addMarker(91, 0, "x", "pin", "red")`, `addMarker(0, -181, ...)`, `addMarker(0, 0, "x", "", "red")` | each `IllegalArgumentException`; `markerCount()` = 0, `instancesCreated()` = 0 |

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
