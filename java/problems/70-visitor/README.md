# 70 · Visitor (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — the `Element` and `Visitor<R>` interfaces are provided; the elements, visitors and `Document` are stubs. |

## Problem

A document editor has paragraphs, images and tables. Product keeps asking for new things to do with a document —
export to HTML, count words, later PDF and accessibility audits. Adding a method to every element class each time
does not scale. Use visitors.

**What you are building:** a small document model made of three kinds of element (paragraph, image, table) and two
operations over documents: export to HTML and count words. New operations written later (the tests add an "image
counter") must work on the same elements without changing the element classes. A `Document` holds elements in order
and can apply any operation to all of them.

### Provided types

- `Element` — `<R> R accept(Visitor<R> visitor)`.
- `Visitor<R>` — `R visitParagraph(String text)`, `R visitImage(String url, int width, int height)`, `R visitTable(List<List<String>> rows)` (`rows` is a rectangular, non-empty grid of cell texts).

### Glossary

- **Visitor**: an operation over all element kinds, with one method per kind; its return type `R` is the result per element.
- **Word**: a maximal run of non-whitespace characters. `"  two   words "` has 2 words; `""` has 0.
- **Ragged table**: rows of different lengths.

## Requirements

1. `Paragraph(text)`, `Image(url, width, height)`, `Table(rows)` implement `Element`; `accept(visitor)` calls the matching `visitParagraph(text)`, `visitImage(url, w, h)` or `visitTable(rows)` with the element's own data and **returns the visitor's result**. Validation (constructors, `IllegalArgumentException`): `null` paragraph text (empty text is allowed); blank url or width/height `<= 0`; empty, ragged, zero-width or null-cell tables, or `null` rows. A `Table` copies its rows (later changes to the original lists must not leak in).
2. `HtmlExporter implements Visitor<String>`; escaping `&` `<` `>` `"` (ampersand first, so `<` becomes `&lt;` and not `&amp;lt;`) in text, cell texts and attribute values (the image url):
   - paragraph `<p>TEXT</p>`
   - image `<img src="URL" width="W" height="H"/>`
   - table `<table><tr><td>A</td><td>B</td></tr><tr>…</tr></table>` (rows in order, cells in order, no whitespace between tags)
3. `WordCounter implements Visitor<Integer>`: paragraph → its number of words, image → 0, table → all words in all cells.
4. `Document` (no-argument constructor): `add(element)` (null → `IllegalArgumentException`), `size()`, `visitAll(visitor)` → list of the visitor's results in insertion order, `toHtml()` → every element's HTML joined with `"\n"` (`""` when empty, no trailing newline), `totalWords()` (sum over elements; 0 when empty).

## Examples

Sample document: `Paragraph("Hello big world")`, `Image("cat.png", 640, 480)`, `Table([["a b","c"],["d","e f g"]])`, `Paragraph("The end")`.

| Operation | Result |
|---|---|
| `size()` | `4` |
| `totalWords()` | `3 + 0 + 7 + 2 = 12` |
| `toHtml()` | the four lines below joined with `\n` |

```
<p>Hello big world</p>
<img src="cat.png" width="640" height="480"/>
<table><tr><td>a b</td><td>c</td></tr><tr><td>d</td><td>e f g</td></tr></table>
<p>The end</p>
```

Escaping and dispatch:

```
new Paragraph("Fish & Chips <3").accept(new HtmlExporter())   -> "<p>Fish &amp; Chips &lt;3</p>"
new Image("a\"b.png", 10, 20).accept(new HtmlExporter())      -> "<img src=\"a&quot;b.png\" width=\"10\" height=\"20\"/>"
new Table(List.of(List.of("x","y"))).accept(traceVisitor)     -> whatever visitTable([[x, y]]) returns
```

A new operation: a visitor returning 1 for images and 0 otherwise, passed to `sample.visitAll(...)`, gives `[0, 1, 0, 0]` with no change to the element classes.

## Edge cases

- `new Paragraph("").accept(new WordCounter())` is `0`; `new Document().toHtml()` is `""`; `new Document().totalWords()` is `0`.
- Invalid: `new Paragraph(null)`, `new Image(" ", 1, 1)`, `new Image("u", 0, 1)`, `new Image("u", 1, -1)`, `new Table(List.of())`, `new Table(List.of(List.of()))`, `new Table(List.of(List.of("a"), List.of("b","c")))` all throw `IllegalArgumentException`.
- Building a `Table` from mutable lists and then mutating them does not change what the table reports.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
