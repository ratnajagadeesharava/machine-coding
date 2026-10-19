# 70 · Visitor (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — the `Element` and `Visitor` traits and `DocError` are provided; the elements, visitors and `Document` are stubs. |

## Problem

A document editor has paragraphs, images and tables. Product keeps asking for new things to do with a document —
export to HTML, count words, later PDF and accessibility audits. Adding a method to every element type each time
does not scale. Use visitors.

## Requirements

A visitor is a `&mut` object that accumulates its result inside itself; `accept` returns nothing.

1. `Paragraph::new(text)`, `Image::new(url, w, h)`, `Table::new(rows)` implement `Element`; `accept` calls the matching `visit_paragraph(text)`, `visit_image(url, w, h)` or `visit_table(rows)`. Validation: blank url / zero size → `Err(DocError::InvalidImage)`; empty, ragged or zero-width table → `Err(DocError::InvalidTable)`.
2. `HtmlExporter` pushes one fragment per visited element (see `fragments()`), escaping `&` `<` `>` `"` (ampersand first) in text and attribute values:
   - paragraph `<p>TEXT</p>`
   - image `<img src="URL" width="W" height="H"/>`
   - table `<table><tr><td>A</td><td>B</td></tr><tr>…</tr></table>`
3. `WordCounter` accumulates `total()`: paragraph → its words (runs of non-whitespace), image → 0, table → all words in all cells.
4. `Document`: `add(Box<dyn Element>)`, `len()`, `accept_all(&mut dyn Visitor)` (in order), `to_html()` → fragments joined with `"\n"` (`""` when empty), `total_words()`.

## Running

```bash
cargo test -p p70_visitor     # from rust/ — tests FAIL until you implement the stubs
cargo play run 70
```
