# 49 · ISP — Interface Segregation Principle (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — three tiny interfaces (`Printer`, `Scanner`, `FaxMachine`) are provided; three devices and three jobs are stubs. |

> **Principle:** no client should be forced to depend on methods it does not use. Prefer several small,
> focused interfaces over one fat `Machine` interface that makes a plain printer implement `scan()` and `fax()` with
> `throw new UnsupportedOperationException()`.

## Problem

An office has simple printers, flatbed scanners and expensive all-in-one machines. Jobs (print, scan, fax) should
run on **any** device that can do that one thing, including a test fake that can do nothing else.

## Requirements

1. `BasicPrinter` implements **only** `Printer`: `print(doc)` returns `"PRINTED: " + doc`; a blank document → `IllegalArgumentException`.
2. `FlatbedScanner` implements **only** `Scanner`: pages are `"SCAN-1"`, `"SCAN-2"`, ...
3. `AllInOne` implements `Printer`, `Scanner` and `FaxMachine` **directly** (no combined interface). `fax(number, doc)`: blank doc → `IllegalArgumentException`; a number that is not an optional `+` followed by 7–15 digits → `false`; a good number → `true`.
4. `PrintJob.run(printer, docs)` prints every document in order and returns the outputs (`null` docs → empty list). `ScanJob.run(scanner, pages)` (`pages <= 0` → `IllegalArgumentException`). `FaxJob.send(machine, number, doc)` returns whether it went out. A `null` device → `IllegalArgumentException`.

## Constraints (what the tests catch)

- The jobs are called with lambdas that implement just one tiny interface. If a job asks for `AllInOne` (or any bigger type) the test does not compile.
- Reflection checks that `BasicPrinter` / `FlatbedScanner` have exactly one direct interface and `AllInOne` exactly the three.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
