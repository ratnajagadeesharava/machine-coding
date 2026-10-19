# 70 · Visitor (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — the `Element` and `Visitor<R>` interfaces are provided; the elements, visitors and `Document` are stubs. |

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

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
