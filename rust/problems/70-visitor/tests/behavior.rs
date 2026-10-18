use p70_visitor::contract::*;
use p70_visitor::{Document, HtmlExporter, Image, Paragraph, Table, WordCounter};

/// Tells which visit method was called with which data - the double dispatch made visible.
#[derive(Default)]
struct Trace(Vec<String>);

impl Visitor for Trace {
    fn visit_paragraph(&mut self, text: &str) {
        self.0.push(format!("P({text})"));
    }
    fn visit_image(&mut self, url: &str, width: u32, height: u32) {
        self.0.push(format!("I({url},{width}x{height})"));
    }
    fn visit_table(&mut self, rows: &[Vec<String>]) {
        self.0.push(format!("T({rows:?})"));
    }
}

/// A brand-new operation written in the test: it needs no change to any element type.
#[derive(Default)]
struct ImageCounter(usize);

impl Visitor for ImageCounter {
    fn visit_paragraph(&mut self, _text: &str) {}
    fn visit_image(&mut self, _url: &str, _w: u32, _h: u32) {
        self.0 += 1;
    }
    fn visit_table(&mut self, _rows: &[Vec<String>]) {}
}

fn grid(rows: &[&[&str]]) -> Vec<Vec<String>> {
    rows.iter().map(|r| r.iter().map(|c| c.to_string()).collect()).collect()
}

fn sample() -> Document {
    let mut doc = Document::new();
    doc.add(Box::new(Paragraph::new("Hello big world")));
    doc.add(Box::new(Image::new("cat.png", 640, 480).unwrap()));
    doc.add(Box::new(Table::new(grid(&[&["a b", "c"], &["d", "e f g"]])).unwrap()));
    doc.add(Box::new(Paragraph::new("The end")));
    doc
}

#[test]
fn should_dispatch_to_the_method_that_matches_the_element_type() {
    let mut trace = Trace::default();

    Paragraph::new("hi").accept(&mut trace);
    Image::new("u.png", 3, 4).unwrap().accept(&mut trace);
    Table::new(grid(&[&["x", "y"]])).unwrap().accept(&mut trace);

    assert_eq!(trace.0, vec!["P(hi)", "I(u.png,3x4)", "T([[\"x\", \"y\"]])"]);
}

#[test]
fn should_export_each_element_as_html() {
    let mut html = HtmlExporter::new();

    Paragraph::new("Fish & Chips <3").accept(&mut html);
    Image::new("a\"b.png", 10, 20).unwrap().accept(&mut html);
    Table::new(grid(&[&["1", "2"], &["3", "4"]])).unwrap().accept(&mut html);

    assert_eq!(
        html.fragments(),
        &[
            "<p>Fish &amp; Chips &lt;3</p>".to_string(),
            "<img src=\"a&quot;b.png\" width=\"10\" height=\"20\"/>".to_string(),
            "<table><tr><td>1</td><td>2</td></tr><tr><td>3</td><td>4</td></tr></table>".to_string(),
        ]
    );
}

#[test]
fn should_count_words_in_paragraphs_and_table_cells_but_not_images() {
    let mut counter = WordCounter::new();
    Paragraph::new("  two   words ").accept(&mut counter);
    assert_eq!(counter.total(), 2);
    Paragraph::new("").accept(&mut counter);
    Image::new("x.png", 1, 1).unwrap().accept(&mut counter);
    assert_eq!(counter.total(), 2);
    Table::new(grid(&[&["a b", "c"], &["d", "e f g"]])).unwrap().accept(&mut counter);

    assert_eq!(counter.total(), 9);
}

#[test]
fn should_apply_a_visitor_to_every_element_in_order() {
    let mut trace = Trace::default();

    sample().accept_all(&mut trace);

    assert_eq!(trace.0, vec!["P(Hello big world)", "I(cat.png,640x480)", "T([[\"a b\", \"c\"], [\"d\", \"e f g\"]])", "P(The end)"]);
}

#[test]
fn should_support_a_new_operation_without_changing_any_element_type() {
    let mut images = ImageCounter::default();

    sample().accept_all(&mut images);

    assert_eq!(images.0, 1);
}

#[test]
fn should_build_whole_documents_from_the_visitors() {
    let doc = sample();

    assert_eq!(doc.len(), 4);
    assert_eq!(doc.total_words(), 3 + 7 + 2);
    assert_eq!(
        doc.to_html(),
        [
            "<p>Hello big world</p>",
            "<img src=\"cat.png\" width=\"640\" height=\"480\"/>",
            "<table><tr><td>a b</td><td>c</td></tr><tr><td>d</td><td>e f g</td></tr></table>",
            "<p>The end</p>"
        ]
        .join("\n")
    );
    assert_eq!(Document::new().to_html(), "");
    assert_eq!(Document::new().total_words(), 0);
}

#[test]
fn should_keep_working_with_element_types_it_has_never_seen() {
    struct Divider;
    impl Element for Divider {
        fn accept(&self, visitor: &mut dyn Visitor) {
            visitor.visit_paragraph("---");
        }
    }
    let mut doc = Document::new();
    doc.add(Box::new(Divider));
    doc.add(Box::new(Paragraph::new("after")));

    assert_eq!(doc.to_html(), "<p>---</p>\n<p>after</p>");
    assert_eq!(doc.total_words(), 2);
}

#[test]
fn should_reject_invalid_elements() {
    assert!(matches!(Image::new(" ", 1, 1), Err(DocError::InvalidImage)));
    assert!(matches!(Image::new("u", 0, 1), Err(DocError::InvalidImage)));
    assert!(matches!(Image::new("u", 1, 0), Err(DocError::InvalidImage)));
    assert!(matches!(Table::new(vec![]), Err(DocError::InvalidTable)));
    assert!(matches!(Table::new(grid(&[&["a"], &["b", "c"]])), Err(DocError::InvalidTable)));
    assert!(matches!(Table::new(vec![vec![]]), Err(DocError::InvalidTable)));
    assert!(Table::new(grid(&[&["ok"]])).is_ok());
}
