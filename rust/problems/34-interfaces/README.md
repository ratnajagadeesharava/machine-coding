# 34 · Interfaces (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — the `Sensor` trait, `SensorFault` and `HubError` are provided; the rest is yours to write. |

## Problem

**What you are building.** A building-monitoring system collects readings from many kinds of sensors (temperature,
humidity, and kinds nobody has invented yet). You write two concrete sensors and a `SensorHub` that registers
sensors and collects their readings. The hub must work with *any* type that implements the provided `Sensor` trait,
including ones it has never seen, and a sensor that reports an impossible value (a "fault") must be reported as
faulty without breaking the readings of the healthy ones.

### Terms used below

- **Probe**: the closure that supplies the raw number a sensor reads; it is called on every `read()`.
- **Fault**: a sensor reading that is physically impossible, so the sensor is considered broken for that read (`Err(SensorFault(..))`).
- **Healthy** sensor: one whose `read()` succeeded during the readings being looked at.
- **Unit**: the label of what a sensor measures: `"C"` (degrees Celsius), `"%"` (percent), or anything a custom sensor declares (`"lx"`).

## Requirements

1. `contract::Sensor` (provided) has `id()`, `unit()` and `read() -> Result<f64, SensorFault>`.
2. `TemperatureSensor::new(id, probe)` (the probe is a `Box<dyn Fn() -> f64>`): unit `"C"`; `read()` returns `Ok(probe value)`. A probe value below `-273.15` makes `read()` return `Err(SensorFault(..))`. `-273.15` itself is fine.
3. `HumiditySensor::new(id, probe)`: unit `"%"`; `read()` returns `Ok(probe value)`. A probe value outside `0..=100` (inclusive on both ends) is a fault: `100.0` is fine, `120.0` and `-0.5` are faults.
4. `SensorHub::register(Box<dyn Sensor>)`: a duplicate id gives `Err(HubError::DuplicateId)` and nothing changes. `unregister(id)` returns `true` if a sensor was removed, `false` if the id was unknown.
5. `read_all()` returns `(id, reading)` pairs in **registration order**, leaving out faulty sensors. `last_faults()` lists the ids that were faulty during the last `read_all()` (registration order).
6. `average(unit)` is the mean of the readings of the healthy sensors with that unit, or `None` if there are none (unknown unit, or all faulty).
7. `ids()` lists registered ids in registration order.

## Examples

```
hub: t1=20C, t2=30C, broken temp, h1=80%      average("C") -> Some((20+30) / 2) = Some(25.0)  (the broken one is skipped)
```

Registration order and faults:

```
register TemperatureSensor("b", 10.0)     register HumiditySensor("broken", -5.0)     register HumiditySensor("a", 55.0)
read_all()    -> [("b", 10.0), ("a", 55.0)]       ("broken" is left out)
last_faults() -> ["broken"]
```

A custom sensor and duplicates:

```
register custom Sensor("lux1", "lx", 300.0)   register TemperatureSensor("t1", 20.0)
ids() -> ["lux1", "t1"]                       average("lx") -> Some(300.0)      average("%") -> None
register another sensor with id "t1"          -> Err(HubError::DuplicateId), hub unchanged
```

Averages: t1=20, t2=30, t3=-500 (faulty), custom t4 (unit "C") =10, h1=80% → `average("C")` = `Some(20.0)`; `average("%")` = `Some(80.0)`; `average("lx")` = `None`.

## Edge cases

- `read_all()` on a hub with no sensors (or only faulty ones) returns an empty list; `last_faults()` then lists only the faulty ids.
- Unregistering an id that is not registered returns `false` and changes nothing.
- After `unregister("t1")`, `ids()` no longer contains it and the order of the others is unchanged.
- `average` ignores faulty sensors but still counts sensors of any type (provided or custom) that declare the unit.

## Running

```bash
cargo test -p p34_interfaces     # from rust/ — tests FAIL until you implement the stubs
cargo play run 34
```
