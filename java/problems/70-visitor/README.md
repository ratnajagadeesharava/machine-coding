# 70 · Visitor (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — the `Element` and `Visitor<R>` interfaces are provided; three elements, two visitors and `Document` are stubs. |

> **Pattern:** *Visitor* — represent an **operation** to be performed on the elements of an object structure as a
> separate object. You can add new operations (HTML export, word count, "list the images"...) **without touching the
> element classes**. The trick is *double dispatch*: `element.accept(visitor)` calls back `visitor.visitXxx(...)`, so
> the right method is chosen by both the element type and the visitor type.

## Problem

A document editor has paragraphs, images and tables. Product keeps asking for new things to do with a document —
export to HTML, count words, later PDF and accessibility audits. Adding a method to every element class each time
does not scale. Use visitors.

## Requirements

1. `Paragraph(text)`, `Image(url, width, height)`, `Table(rows)` implement `Element`; `accept(visitor)` calls the matching `visitParagraph(text)`, `visitImage(url, w, h)` or `visitTable(rows)` and **returns the visitor's result**. Validation: `null` paragraph text; blank url or non-positive size; empty / ragged / zero-width / null-cell tables → `IllegalArgumentException`. A `Table` copies its rows (later changes to the original lists must not leak in).
2. `HtmlExporter implements Visitor<String>`; escaping `&` `<` `>` `"` (ampersand first) in text and attribute values:
   - paragraph `<p>TEXT</p>`
   - image `<img src="URL" width="W" height="H"/>`
   - table `<table><tr><td>A</td><td>B</td></tr><tr>…</tr></table>`
3. `WordCounter implements Visitor<Integer>`: paragraph → its words (runs of non-whitespace), image → 0, table → all words in all cells.
4. `Document`: `add(element)` (null → `IllegalArgumentException`), `size()`, `visitAll(visitor)` → results in order, `toHtml()` → every element's HTML joined with `"\n"` (`""` when empty), `totalWords()`.

## Constraints (what the tests catch)

- A `Trace` visitor shows which method each element calls; an `ImageCounter` visitor defined in the test works with the existing elements untouched.
- `Document.toHtml()` / `totalWords()` must be built on your visitors, not on `instanceof` checks.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
