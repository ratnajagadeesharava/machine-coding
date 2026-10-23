# 70 · Visitor (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — the `Element` and `Visitor` traits and `DocError` are provided; the elements, visitors and `Document` are stubs. |

## Problem

A document editor has paragraphs, images and tables. Product keeps asking for new things to do with a document —
export to HTML, count words, later PDF and accessibility audits. Adding a method to every element type each time
does not scale. Use visitors.

**What you are building:** a small document model made of three kinds of element (paragraph, image, table) and two
operations over documents: export to HTML and count words. New operations written later (the tests add an "image
counter") must work on the same elements without changing the element types, and new element types written by
callers (the tests add a "divider") must work with the existing operations. A `Document` holds elements in order and
can apply any operation to all of them.

### Provided types

- `Element` — `fn accept(&self, visitor: &mut dyn Visitor)`.
- `Visitor` — `visit_paragraph(&mut self, text: &str)`, `visit_image(&mut self, url: &str, width: u32, height: u32)`, `visit_table(&mut self, rows: &[Vec<String>])` (`rows` is a rectangular, non-empty grid of cell texts).
- `DocError::{InvalidImage, InvalidTable}`.

### Glossary

- **Visitor**: an operation over all element kinds, with one method per kind. A visitor is a `&mut` object that accumulates its result inside itself; `accept` returns nothing.
- **Word**: a maximal run of non-whitespace characters. `"  two   words "` has 2 words; `""` has 0.
- **Ragged table**: rows of different lengths.

## Requirements

1. `Paragraph::new(text)` (any text, including empty), `Image::new(url, w, h)`, `Table::new(rows)` implement `Element`; `accept` calls the matching `visit_paragraph(text)`, `visit_image(url, w, h)` or `visit_table(rows)` once, with the element's own data. Validation: blank url or zero width/height → `Err(DocError::InvalidImage)`; empty, ragged or zero-width table → `Err(DocError::InvalidTable)`.
2. `HtmlExporter` (`HtmlExporter::new()`) pushes one fragment per visited element, in visiting order (see `fragments()`), escaping `&` `<` `>` `"` (ampersand first, so `<` becomes `&lt;` and not `&amp;lt;`) in text, cell texts and attribute values (the image url):
   - paragraph `<p>TEXT</p>`
   - image `<img src="URL" width="W" height="H"/>`
   - table `<table><tr><td>A</td><td>B</td></tr><tr>…</tr></table>` (rows in order, cells in order, no whitespace between tags)
3. `WordCounter` (`WordCounter::new()`) accumulates `total()` across everything it visits: paragraph → its number of words, image → 0, table → all words in all cells.
4. `Document` (`Document::new()`): `add(Box<dyn Element>)`, `len()`, `accept_all(&mut dyn Visitor)` (visits every element in insertion order), `to_html()` → every element's fragment joined with `"\n"` (`""` when empty, no trailing newline), `total_words()` (sum over elements; 0 when empty). `to_html` and `total_words` also work for caller-defined `Element` types, which only call the visitor's methods (e.g. an element calling `visit_paragraph("---")` appears as `<p>---</p>`).

## Examples

Sample document: `Paragraph("Hello big world")`, `Image("cat.png", 640, 480)`, `Table([["a b","c"],["d","e f g"]])`, `Paragraph("The end")`.

| Operation | Result |
|---|---|
| `len()` | `4` |
| `total_words()` | `3 + 0 + 7 + 2 = 12` |
| `to_html()` | the four lines below joined with `\n` |

```
<p>Hello big world</p>
<img src="cat.png" width="640" height="480"/>
<table><tr><td>a b</td><td>c</td></tr><tr><td>d</td><td>e f g</td></tr></table>
<p>The end</p>
```

Escaping:

```
Paragraph::new("Fish & Chips <3").accept(&mut html)   -> fragment "<p>Fish &amp; Chips &lt;3</p>"
Image::new("a\"b.png", 10, 20)?.accept(&mut html)     -> fragment "<img src=\"a&quot;b.png\" width=\"10\" height=\"20\"/>"
```

A `WordCounter` is cumulative: visiting `"  two   words "` gives `total() == 2`; visiting an empty paragraph and an image leaves it at 2; then visiting a table `[["a b","c"],["d","e f g"]]` makes it 9.

A new operation: a visitor that counts `visit_image` calls, passed to `sample.accept_all(&mut counter)`, ends at 1 with no change to the element types.

## Edge cases

- `Document::new().to_html()` is `""`; `Document::new().total_words()` is `0`.
- Invalid: `Image::new(" ", 1, 1)`, `Image::new("u", 0, 1)`, `Image::new("u", 1, 0)` → `Err(InvalidImage)`; `Table::new(vec![])`, `Table::new(vec![vec![]])`, a ragged grid → `Err(InvalidTable)`; `Table::new` of a one-cell grid is `Ok`.

## Running

```bash
cargo test -p p70_visitor     # from rust/ — tests FAIL until you implement the stubs
cargo play run 70
```
