# 49 · ISP — Interface Segregation Principle (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — three tiny traits (`Printer`, `Scanner`, `FaxMachine`) and `DeviceError` are provided in `src/contract.rs`; three devices and three job functions are stubs. |

> **Principle:** no client should be forced to depend on methods it does not use. Prefer several small, focused
> traits over one fat `Machine` trait that makes a plain printer implement `scan()` and `fax()` with `todo!()` or `panic!()`.

## Problem

An office has simple printers, flat-bed scanners and expensive all-in-one machines. Jobs (print, scan, fax) should
run on **any** device that can do that one thing, including a test fake that can do nothing else.

## Requirements

1. `BasicPrinter` implements **only** `Printer`: `print(doc)` returns `"PRINTED: " + doc`; a blank document → `Err(DeviceError::BlankDocument)`.
2. `FlatbedScanner` implements **only** `Scanner`: pages are `"SCAN-1"`, `"SCAN-2"`, ...
3. `AllInOne` implements `Printer`, `Scanner` and `FaxMachine` **directly**. `fax(number, doc)`: blank doc → `Err(BlankDocument)`; a number that is not an optional `+` followed by 7–15 digits → `Ok(false)`; a good number → `Ok(true)`.
4. `print_job(&dyn Printer, docs)` prints every document in order and returns the outputs (stops with the error on the first blank one). `scan_job(&mut dyn Scanner, pages)` (`pages == 0` → `Err(InvalidPages)`). `fax_job(&mut dyn FaxMachine, number, doc)` returns what the machine returned.

## Constraints (what the tests catch)

- The jobs are called with test types that implement **just one** tiny trait (`PrinterOnly`, `ScannerOnly`, `FaxOnly`). If a job asks for anything bigger, the test does not compile.
- **Rust adaptation:** Java also asserts `AllInOne.class.getInterfaces()` by reflection; Rust has no equivalent, so the compiler plays that role.

## Running

```bash
cargo test -p p49_isp     # from rust/ — tests FAIL until you implement the stubs
cargo play run 49
```
