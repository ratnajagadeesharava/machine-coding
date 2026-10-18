# 34 · Interfaces (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — the `Sensor` interface is provided; the classes you write are stubs that throw `UnsupportedOperationException`. |

> **Concept:** an interface is a contract. Code that depends on the interface works with every implementation,
> including ones that do not exist yet.

## Problem

A building-monitoring system collects readings from many kinds of sensors. Write two sensors and a `SensorHub`
that collects readings **without knowing which concrete sensor types exist**.

## Requirements

1. `contract.Sensor` (provided) has `id()`, `read()` and `unit()`.
2. `TemperatureSensor(id, DoubleSupplier probe)`: unit `"C"`. If the probe returns less than `-273.15`, `read()` throws `IllegalStateException` (faulty sensor).
3. `HumiditySensor(id, DoubleSupplier probe)`: unit `"%"`. A probe value outside `0..100` (inclusive) is a fault (`IllegalStateException`).
4. `SensorHub.register(sensor)`; a second sensor with an existing id → `IllegalArgumentException` and nothing changes. `unregister(id)` returns whether something was removed.
5. `readAll()` returns `id -> reading` in **registration order**, leaving out faulty sensors. `lastFaults()` lists the ids that were faulty during the last `readAll()`.
6. `average(unit)` is the mean of the healthy sensors with that unit, or empty if there are none.
7. `sensors()` returns the sensors in registration order; callers cannot modify the returned list.

## Constraints

- `SensorHub` may use only the `Sensor` interface. The tests register a sensor class that exists only in the test file.

## Examples

```
hub: t1=20C, t2=30C, broken temp, h1=80%      average("C") -> 20+30 / 2 = 25  (the broken one is skipped)
```

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the three classes
```
