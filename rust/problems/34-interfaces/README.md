# 34 · Interfaces (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — the `Sensor` trait is provided in `src/contract.rs`; the types you write are stubs in `src/lib.rs` with `todo!()` bodies. |

> **Concept:** a trait is Rust's interface. Code that depends on the trait works with every implementor,
> including ones that do not exist yet (`Box<dyn Sensor>`).

## Problem

A building-monitoring system collects readings from many kinds of sensors. Write two sensors and a `SensorHub`
that collects readings **without knowing which concrete sensor types exist**.

## Requirements

1. `contract::Sensor` (provided) has `id()`, `unit()` and `read() -> Result<f64, SensorFault>`.
2. `TemperatureSensor::new(id, probe)`: unit `"C"`. A probe value below `-273.15` makes `read()` return `Err(SensorFault(..))`.
3. `HumiditySensor::new(id, probe)`: unit `"%"`. A probe value outside `0..=100` is a fault.
4. `SensorHub::register(Box<dyn Sensor>)`: a duplicate id gives `Err(HubError::DuplicateId)` and nothing changes. `unregister(id)` returns whether something was removed.
5. `read_all()` returns `(id, reading)` in **registration order**, leaving out faulty sensors. `last_faults()` lists the ids that were faulty during the last `read_all()`.
6. `average(unit)` is the mean of the healthy sensors with that unit, or `None`.
7. `ids()` lists registered ids in registration order.

## Constraints

- `SensorHub` may use only the `Sensor` trait. The tests register a sensor type that exists only in the test file.

## Running

```bash
cargo test -p p34_interfaces     # from rust/ — tests FAIL until you implement the stubs
cargo play run 34
```
