# 34 · Interfaces (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — the `Sensor` interface is provided; the rest is yours to write. |

## Problem

**What you are building.** A building-monitoring system collects readings from many kinds of sensors (temperature,
humidity, and kinds nobody has invented yet). You write two concrete sensors and a `SensorHub` that registers
sensors and collects their readings. The hub must work with *any* object that satisfies the provided `Sensor`
interface, including ones it has never seen, and a sensor that reports an impossible value (a "fault") must be
reported as faulty without breaking the readings of the healthy ones.

### Terms used below

- **Probe**: the supplier of the raw number a sensor reads (`DoubleSupplier`); it is called on every `read()`.
- **Fault**: a sensor reading that is physically impossible, so the sensor is considered broken for that read.
- **Healthy** sensor: one whose `read()` succeeded during the readings being looked at.
- **Unit**: the label of what a sensor measures: `"C"` (degrees Celsius), `"%"` (percent), or anything a custom sensor declares (`"lx"`).

## Requirements

1. `contract.Sensor` (provided) has `id()`, `read()` and `unit()`. `read()` throws `IllegalStateException` when the sensor is faulty.
2. `TemperatureSensor(id, DoubleSupplier probe)`: unit `"C"`; `read()` returns the probe's value. If the probe returns less than `-273.15`, `read()` throws `IllegalStateException` (faulty sensor). `-273.15` itself is fine.
3. `HumiditySensor(id, DoubleSupplier probe)`: unit `"%"`; `read()` returns the probe's value. A probe value outside `0..100` (inclusive on both ends) is a fault (`IllegalStateException`): `100.0` is fine, `120.0` and `-5.0` are faults.
4. `SensorHub.register(sensor)`; a second sensor with an existing id → `IllegalArgumentException` and nothing changes. `unregister(id)` returns `true` if a sensor was removed, `false` if the id was unknown.
5. `readAll()` returns `id -> reading` in **registration order**, leaving out faulty sensors. `lastFaults()` lists the ids that were faulty during the last `readAll()` (registration order).
6. `average(unit)` is the mean of the readings of the healthy sensors with that unit, or empty if there are none (unknown unit, or all faulty).
7. `sensors()` returns the sensors in registration order; callers cannot modify the returned list (attempts throw `UnsupportedOperationException`).

## Examples

```
hub: t1=20C, t2=30C, broken temp, h1=80%      average("C") -> 20+30 / 2 = 25  (the broken one is skipped)
```

Registration order and faults:

```
register TemperatureSensor("b", 10.0)     register HumiditySensor("broken", -5.0)     register HumiditySensor("a", 55.0)
readAll()    -> {b=10.0, a=55.0}          (in that order; "broken" is left out)
lastFaults() -> ["broken"]
```

A custom sensor and duplicates:

```
register custom Sensor("lux1", "lx", 300.0)   register TemperatureSensor("t1", 20.0)
readAll() -> {lux1=300.0, t1=20.0}            average("lx") -> 300.0      average("%") -> empty
register another sensor with id "t1"          -> IllegalArgumentException, hub unchanged
```

Averages: t1=20, t2=30, t3=-500 (faulty), custom t4 (unit "C") =10, h1=80% → `average("C")` = (20+30+10)/3 = `20.0`; `average("%")` = `80.0`; `average("lx")` = empty.

## Edge cases

- `readAll()` on a hub with no sensors (or only faulty ones) returns an empty map; `lastFaults()` then lists only the faulty ids.
- Unregistering an id that is not registered returns `false` and changes nothing.
- After `unregister("t1")`, `sensors()` no longer contains it and the order of the others is unchanged.
- `average` ignores faulty sensors but still counts sensors of any class (provided or custom) that declare the unit.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the three classes
```
