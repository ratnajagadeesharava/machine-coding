# 49 · ISP — Interface Segregation Principle (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — three tiny interfaces (`Printer`, `Scanner`, `FaxMachine`) are provided; the rest is yours to write. |

## Problem

An office has simple printers, flatbed scanners and expensive all-in-one machines. Jobs (print, scan, fax) should
run on **any** device that can do that one thing, including a test fake that can do nothing else.

## Requirements

1. `BasicPrinter` — `print(doc)` returns `"PRINTED: " + doc`; a blank document → `IllegalArgumentException`.
2. `FlatbedScanner` — pages are `"SCAN-1"`, `"SCAN-2"`, ...
3. `AllInOne` — can print, scan and fax. `fax(number, doc)`: blank doc → `IllegalArgumentException`; a number that is not an optional `+` followed by 7–15 digits → `false`; a good number → `true`.
4. `PrintJob.run(printer, docs)` prints every document in order and returns the outputs (`null` docs → empty list). `ScanJob.run(scanner, pages)` (`pages <= 0` → `IllegalArgumentException`). `FaxJob.send(machine, number, doc)` returns whether it went out. A `null` device → `IllegalArgumentException`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
