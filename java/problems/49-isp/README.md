# 49 · ISP — Interface Segregation Principle (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — three tiny interfaces (`Printer`, `Scanner`, `FaxMachine`) are provided; the rest is yours to write. |

## Problem

**What you are building.** An office has simple printers, flatbed scanners and expensive all-in-one machines that can print, scan and fax. The office software runs three kinds of jobs: a print job, a scan job and a fax job. Each job should run on **any** device that can do that one thing, including a test fake that can do nothing else, and a simple printer must never be asked to scan or fax. You write the devices and the three jobs.

## Requirements

Provided in `contract/`: three one-method interfaces: `Printer` (`String print(String document)`, returns what came out), `Scanner` (`String scan()`, returns the next page) and `FaxMachine` (`boolean fax(String number, String document)`, true if sent). You write `BasicPrinter`, `FlatbedScanner`, `AllInOne`, `PrintJob`, `ScanJob` and `FaxJob`.

1. `BasicPrinter` is a printer and nothing else. `print(doc)` returns `"PRINTED: " + doc`. A blank document (`null`, empty or whitespace only) → `IllegalArgumentException`.
2. `FlatbedScanner` is a scanner and nothing else. Each `scan()` returns the next page, numbered from 1 per scanner instance: `"SCAN-1"`, `"SCAN-2"`, ... A new scanner starts again at `"SCAN-1"`.
3. `AllInOne` can print, scan and fax, and can be used anywhere a printer, a scanner or a fax machine is expected. It prints like `BasicPrinter` and numbers its scanned pages like `FlatbedScanner` (`"SCAN-1"` first). `fax(number, doc)`: blank doc → `IllegalArgumentException`; a number that is not an optional `+` followed by 7 to 15 digits (and nothing else) → returns `false`; a valid number → returns `true`.
4. Jobs accept any device that can do the one thing needed, including a fake that can do nothing else:
   - `PrintJob.run(printer, documents)` prints every document in order and returns the printer's outputs in the same order. A `null` document list → empty list.
   - `ScanJob.run(scanner, pages)` scans `pages` times and returns the pages in scan order. `pages <= 0` → `IllegalArgumentException`.
   - `FaxJob.send(machine, number, document)` returns whatever the machine's `fax` returned.
   - A `null` device (printer, scanner or fax machine) → `IllegalArgumentException`.

## Examples

Devices:

```
new BasicPrinter().print("report")  -> "PRINTED: report"
new BasicPrinter().print(" ")       -> IllegalArgumentException
s = new FlatbedScanner();  s.scan() -> "SCAN-1";  s.scan() -> "SCAN-2"

m = new AllInOne()
m.print("a")                        -> "PRINTED: a"
m.scan()                            -> "SCAN-1"
m.fax("+4915112345678", "a")        -> true      // + and 13 digits
m.fax("5551234", "a")               -> true      // exactly 7 digits
m.fax("12", "a")                    -> false     // too short
m.fax("not-a-number", "a")          -> false
m.fax("5551234", "")                -> IllegalArgumentException
```

Jobs:

```
// a device that can only print: doc -> "fake:" + doc
PrintJob.run(printerOnly, List.of("a", "b"))   -> ["fake:a", "fake:b"]   // printed in order
PrintJob.run(new BasicPrinter(), null)         -> []
PrintJob.run(null, List.of("a"))               -> IllegalArgumentException

// a device that can only scan: returns "page1", "page2", ...
ScanJob.run(scannerOnly, 3)                    -> ["page1", "page2", "page3"]
ScanJob.run(new AllInOne(), 2)                 -> ["SCAN-1", "SCAN-2"]
ScanJob.run(new FlatbedScanner(), 0)           -> IllegalArgumentException

FaxJob.send(faxOnly, "5551234", "contract")    -> true    // faxOnly records the call and returns true
FaxJob.send(new AllInOne(), "bad", "contract") -> false   // the machine rejected the number
FaxJob.send(null, "5551234", "a")              -> IllegalArgumentException
```

## Edge cases

- Whitespace-only counts as blank, for documents.
- A number is valid only if the **whole** string is an optional leading `+` and then 7 to 15 digits: 6 digits, 16 digits, letters, spaces or dashes make it invalid (`false`, not an exception).
- Scan numbering belongs to each scanner instance, not to the program.
- A device that only prints (a basic printer) offers no scan or fax operation at all, and a scanner offers no print or fax.

## Glossary

- **Flatbed scanner**: a scanner you lay pages on; here it just yields numbered pages, one per scan.
- **All-in-one**: one machine that prints, scans and faxes.
- **Fax number**: an optional `+` then 7 to 15 digits, e.g. `5551234` or `+4915112345678`.
- **Blank document**: empty or only whitespace.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
