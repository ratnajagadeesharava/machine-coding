# 49 · ISP — Interface Segregation Principle (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — three tiny traits (`Printer`, `Scanner`, `FaxMachine`) and `DeviceError` are provided in `src/contract.rs`; the rest is yours to write. |

## Problem

**What you are building.** An office has simple printers, flatbed scanners and expensive all-in-one machines that can print, scan and fax. The office software runs three kinds of jobs: a print job, a scan job and a fax job. Each job should run on **any** device that can do that one thing, including a test fake that can do nothing else, and a simple printer must never be asked to scan or fax. You write the devices and the three jobs.

## Requirements

Provided in `src/contract.rs`: three one-method traits `Printer` (`fn print(&self, document: &str) -> Result<String, DeviceError>`), `Scanner` (`fn scan(&mut self) -> String`, the next page) and `FaxMachine` (`fn fax(&mut self, number, document) -> Result<bool, DeviceError>`, `Ok(true)` if sent, `Ok(false)` if the number was rejected), plus `DeviceError { BlankDocument, InvalidPages }`. You write `BasicPrinter`, `FlatbedScanner`, `AllInOne` (each with `new()`) and the functions `print_job`, `scan_job` and `fax_job`.

1. `BasicPrinter` is a printer and nothing else. `print(doc)` returns `Ok("PRINTED: " + doc)`. A blank document (empty or whitespace only) → `Err(DeviceError::BlankDocument)`.
2. `FlatbedScanner` is a scanner and nothing else. Each `scan()` returns the next page, numbered from 1 per scanner instance: `"SCAN-1"`, `"SCAN-2"`, ... A new scanner starts again at `"SCAN-1"`.
3. `AllInOne` can print, scan and fax, and can be used anywhere a printer, a scanner or a fax machine is expected. It prints like `BasicPrinter` and numbers its scanned pages like `FlatbedScanner` (`"SCAN-1"` first). `fax(number, doc)`: blank doc → `Err(BlankDocument)`; a number that is not an optional `+` followed by 7 to 15 digits (and nothing else) → `Ok(false)`; a valid number → `Ok(true)`.
4. Jobs accept any device that can do the one thing needed, including a fake that can do nothing else:
   - `print_job(&dyn Printer, documents: &[&str]) -> Result<Vec<String>, DeviceError>` prints every document in order and returns the printer's outputs in the same order; an empty slice → `Ok(vec![])`. It stops at the first document that fails and returns that error (e.g. `["ok", " "]` with `BasicPrinter` → `Err(BlankDocument)`).
   - `scan_job(&mut dyn Scanner, pages: u32) -> Result<Vec<String>, DeviceError>` scans `pages` times and returns the pages in scan order. `pages == 0` → `Err(DeviceError::InvalidPages)`.
   - `fax_job(&mut dyn FaxMachine, number, document) -> Result<bool, DeviceError>` returns exactly what the machine's `fax` returned.

## Examples

Devices:

```
BasicPrinter::new().print("report")  -> Ok("PRINTED: report")
BasicPrinter::new().print(" ")       -> Err(BlankDocument)
s = FlatbedScanner::new();  s.scan() -> "SCAN-1";  s.scan() -> "SCAN-2"

m = AllInOne::new()
m.print("a")                        -> Ok("PRINTED: a")
m.scan()                            -> "SCAN-1"
m.fax("+4915112345678", "a")        -> Ok(true)    // + and 13 digits
m.fax("5551234", "a")               -> Ok(true)    // exactly 7 digits
m.fax("12", "a")                    -> Ok(false)   // too short
m.fax("not-a-number", "a")          -> Ok(false)
m.fax("5551234", "")                -> Err(BlankDocument)
```

Jobs:

```
// a device that can only print: doc -> "fake:" + doc
print_job(&printer_only, &["a", "b"])          -> Ok(["fake:a", "fake:b"])   // printed in order
print_job(&BasicPrinter::new(), &[])           -> Ok([])
print_job(&BasicPrinter::new(), &["ok", " "])  -> Err(BlankDocument)

// a device that can only scan: returns "page1", "page2", ...
scan_job(&mut scanner_only, 3)                 -> Ok(["page1", "page2", "page3"])
scan_job(&mut AllInOne::new(), 2)              -> Ok(["SCAN-1", "SCAN-2"])
scan_job(&mut FlatbedScanner::new(), 0)        -> Err(InvalidPages)

fax_job(&mut fax_only, "5551234", "contract")  -> Ok(true)    // fax_only records the call and returns Ok(true)
fax_job(&mut AllInOne::new(), "bad", "contract") -> Ok(false) // the machine rejected the number
```

## Edge cases

- Whitespace-only counts as blank, for documents.
- A number is valid only if the **whole** string is an optional leading `+` and then 7 to 15 digits: 6 digits, 16 digits, letters, spaces or dashes make it invalid (`Ok(false)`, not an error).
- Scan numbering belongs to each scanner instance, not to the program.
- A device that only prints (a basic printer) offers no scan or fax operation at all, and a scanner offers no print or fax.

## Glossary

- **Flatbed scanner**: a scanner you lay pages on; here it just yields numbered pages, one per scan.
- **All-in-one**: one machine that prints, scans and faxes.
- **Fax number**: an optional `+` then 7 to 15 digits, e.g. `5551234` or `+4915112345678`.
- **Blank document**: empty or only whitespace.

## Running

```bash
cargo test -p p49_isp     # from rust/ — tests FAIL until you implement the stubs
cargo play run 49
```
