# 70 · Visitor (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — the `Element` and `Visitor` traits and `DocError` are provided; three elements, two visitors and `Document` are stubs. |

> **Pattern:** *Visitor* — represent an **operation** on the elements of an object structure as a separate object.
> You can add new operations (HTML export, word count, "list the images"...) **without touching the element types**.
> The trick is *double dispatch*: `element.accept(visitor)` calls back `visitor.visit_xxx(...)`, so the right method
> is chosen by both the element type and the visitor type.
>
> **Rust adaptation:** a generic `fn accept<R>(&self, v: &mut dyn Visitor<R>) -> R` would make `Element` unusable as
> `Box<dyn Element>`. So a visitor is a `&mut` object that **accumulates its result inside itself**
> (`HtmlExporter::fragments()`, `WordCounter::total()`), and `accept` returns nothing.

## Problem

A document editor has paragraphs, images and tables. Product keeps asking for new things to do with a document —
export to HTML, count words, later PDF and accessibility audits. Adding a method to every element type each time
does not scale. Use visitors.

## Requirements

1. `Paragraph::new(text)`, `Image::new(url, w, h)`, `Table::new(rows)` implement `Element`; `accept` calls the matching `visit_paragraph(text)`, `visit_image(url, w, h)` or `visit_table(rows)`. Validation: blank url / zero size → `Err(DocError::InvalidImage)`; empty, ragged or zero-width table → `Err(DocError::InvalidTable)`.
2. `HtmlExporter` pushes one fragment per visited element (see `fragments()`), escaping `&` `<` `>` `"` (ampersand first) in text and attribute values:
   - paragraph `<p>TEXT</p>`
   - image `<img src="URL" width="W" height="H"/>`
   - table `<table><tr><td>A</td><td>B</td></tr><tr>…</tr></table>`
3. `WordCounter` accumulates `total()`: paragraph → its words (runs of non-whitespace), image → 0, table → all words in all cells.
4. `Document`: `add(Box<dyn Element>)`, `len()`, `accept_all(&mut dyn Visitor)` (in order), `to_html()` → fragments joined with `"\n"` (`""` when empty), `total_words()`.

## Constraints (what the tests catch)

- A `Trace` visitor shows which method each element calls; an `ImageCounter` visitor and a `Divider` element defined in the test work with everything else untouched.
- `Document::to_html()` / `total_words()` must be built on your visitors.

## Running

```bash
cargo test -p p70_visitor     # from rust/ — tests FAIL until you implement the stubs
cargo play run 70
```
