#![allow(unused_imports, clippy::needless_range_loop)]
use p132_word_processor::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

const DEF: &str = "Calibri|11|-|-|-|black";

/// Deliberately naive reference: one char and one format per position.
struct Model {
    chars: Vec<char>,
    fmts: Vec<[String; 6]>, // font,size,B,I,U,color
    aligns: Vec<String>,
}

fn def() -> [String; 6] {
    ["Calibri", "11", "-", "-", "-", "black"].map(String::from)
}

fn fs(f: &[String; 6]) -> String {
    f.join("|")
}

impl Model {
    fn new() -> Self {
        Model { chars: vec![], fmts: vec![], aligns: vec!["LEFT".into()] }
    }
    fn newlines(&self, from: usize, to: usize) -> usize {
        self.chars[from..to].iter().filter(|c| **c == '\n').count()
    }
    fn insert_f(&mut self, pos: usize, text: &str, f: [String; 6]) {
        let t: Vec<char> = text.chars().collect();
        if t.is_empty() {
            return;
        }
        let k = t.iter().filter(|c| **c == '\n').count();
        if k > 0 {
            let line = self.newlines(0, pos);
            let a = self.aligns[line].clone();
            for _ in 0..k {
                self.aligns.insert(line + 1, a.clone());
            }
        }
        for (i, c) in t.iter().enumerate() {
            self.chars.insert(pos + i, *c);
            self.fmts.insert(pos + i, f.clone());
        }
    }
    fn insert(&mut self, pos: usize, text: &str) {
        let f = if pos == 0 { def() } else { self.fmts[pos - 1].clone() };
        self.insert_f(pos, text, f);
    }
    fn delete(&mut self, from: usize, to: usize) {
        let k = self.newlines(from, to);
        if k > 0 {
            let line = self.newlines(0, from);
            for _ in 0..k {
                self.aligns.remove(line + 1);
            }
        }
        for i in (from..to).rev() {
            self.chars.remove(i);
            self.fmts.remove(i);
        }
    }
    fn s(&self) -> String {
        self.chars.iter().collect()
    }
    fn runs(&self) -> Vec<String> {
        let mut out = vec![];
        let mut s = 0;
        for i in 1..=self.chars.len() {
            if i == self.chars.len() || fs(&self.fmts[i]) != fs(&self.fmts[s]) {
                out.push(format!("{s}-{i}:{}", fs(&self.fmts[s])));
                s = i;
            }
        }
        out
    }
    fn find(&self, p: &str, cs: bool) -> Vec<usize> {
        let norm = |s: &str| if cs { s.to_string() } else { s.to_lowercase() };
        let t: Vec<char> = norm(&self.s()).chars().collect();
        let q: Vec<char> = norm(p).chars().collect();
        let mut out = vec![];
        if t.len() >= q.len() {
            for i in 0..=t.len() - q.len() {
                if t[i..i + q.len()] == q[..] {
                    out.push(i);
                }
            }
        }
        out
    }
    fn replace_all(&mut self, p: &str, r: &str, cs: bool) -> usize {
        let plen = p.chars().count();
        let mut chosen = vec![];
        let mut next = 0;
        for m in self.find(p, cs) {
            if m >= next {
                chosen.push(m);
                next = m + plen;
            }
        }
        for &s in chosen.iter().rev() {
            let f = self.fmts[s].clone();
            self.delete(s, s + plen);
            self.insert_f(s, r, f);
        }
        chosen.len()
    }
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n
    }
}

fn assert_same(d: &Arc<dyn WordProcessor>, m: &Model) {
    assert_eq!(d.text(), m.s());
    assert_eq!(d.length(), m.chars.len());
    assert_eq!(d.runs(), m.runs());
    assert_eq!(d.line_count(), m.aligns.len());
    for l in 0..m.aligns.len() {
        assert_eq!(d.alignment(l), Ok(m.aligns[l].clone()), "line {l}");
    }
    let distinct: HashSet<String> = m.fmts.iter().map(fs).collect();
    assert_eq!(d.distinct_format_count(), distinct.len());
}

fn random_text(r: &mut Lcg, max: usize) -> String {
    let alphabet: Vec<char> = "abAB  \n".chars().collect();
    let n = r.next(max + 1);
    (0..n).map(|_| alphabet[r.next(alphabet.len())]).collect()
}

#[test]
fn should_agree_with_a_naive_model_over_thousands_of_random_operations() {
    for seed in 1..=5u64 {
        let d = sut!(create());
        let mut m = Model::new();
        let mut r = Lcg(seed * 7919);
        let colors = ["red", "blue", "black"];
        let fonts = ["Arial", "Calibri"];
        for step in 0..1_500 {
            let len = m.chars.len();
            match r.next(8) {
                0 | 1 => {
                    let pos = r.next(len + 1);
                    let t = random_text(&mut r, 6);
                    d.insert(pos, &t).unwrap();
                    m.insert(pos, &t);
                }
                2 => {
                    let a = r.next(len + 1);
                    let b = len.min(a + r.next(8));
                    d.delete(a, b).unwrap();
                    m.delete(a, b);
                }
                3 | 4 => {
                    let a = r.next(len + 1);
                    let b = a + r.next(len - a + 1);
                    let color = colors[r.next(3)];
                    let font = fonts[r.next(2)];
                    let bold = r.next(2) == 0;
                    d.apply_format(
                        a,
                        b,
                        &[("color", color), ("font", font), ("bold", if bold { "true" } else { "false" })],
                    )
                    .unwrap();
                    for i in a..b {
                        m.fmts[i][5] = color.to_string();
                        m.fmts[i][0] = font.to_string();
                        m.fmts[i][2] = if bold { "B" } else { "-" }.to_string();
                    }
                }
                5 => {
                    let p = ["a", "ab", "aa", "b\n", "A"][r.next(5)];
                    let cs = r.next(2) == 0;
                    assert_eq!(d.find(p, cs), Ok(m.find(p, cs)));
                    let rep = random_text(&mut r, 3);
                    assert_eq!(d.replace_all(p, &rep, cs), Ok(m.replace_all(p, &rep, cs)));
                }
                6 => {
                    let line = r.next(m.aligns.len());
                    let a = ["LEFT", "CENTER", "RIGHT", "JUSTIFY"][r.next(4)];
                    d.set_alignment(line, a).unwrap();
                    m.aligns[line] = a.to_string();
                }
                _ => {
                    if len > 0 {
                        let p = r.next(len);
                        assert_eq!(d.format_at(p), Ok(fs(&m.fmts[p])));
                    }
                }
            }
            if step % 50 == 0 {
                assert_same(&d, &m);
            }
        }
        assert_same(&d, &m);
    }
}

#[test]
fn should_collapse_a_heavily_fragmented_document_back_to_one_run() {
    let d = sut!(create());
    d.insert(0, &"x".repeat(20_000)).unwrap();
    for i in (0..20_000).step_by(2) {
        d.apply_format(i, i + 1, &[("bold", "true")]).unwrap();
    }
    assert_eq!(d.runs().len(), 20_000);
    assert_eq!(d.distinct_format_count(), 2);

    for i in (0..20_000).step_by(2) {
        d.apply_format(i, i + 1, &[("bold", "false")]).unwrap();
    }

    assert_eq!(d.runs(), vec![format!("0-20000:{DEF}")]);
    assert_eq!(d.distinct_format_count(), 1);
}

#[test]
fn should_delete_across_several_runs_and_lines() {
    let d = sut!(create());
    d.insert(0, "aa\nbb\ncc\ndd").unwrap();
    d.apply_format(0, 2, &[("bold", "true")]).unwrap();
    d.apply_format(3, 5, &[("italic", "true")]).unwrap();
    d.apply_format(6, 8, &[("underline", "true")]).unwrap();
    d.set_alignment(0, "CENTER").unwrap();
    d.set_alignment(1, "RIGHT").unwrap();
    d.set_alignment(2, "JUSTIFY").unwrap();

    d.delete(1, 7).unwrap();

    assert_eq!(d.text(), "ac\ndd");
    assert_eq!(
        d.runs(),
        vec![
            "0-1:Calibri|11|B|-|-|black".to_string(),
            "1-2:Calibri|11|-|-|U|black".to_string(),
            format!("2-5:{DEF}"),
        ]
    );
    assert_eq!(d.line_count(), 2);
    assert_eq!(d.alignment(0), Ok("CENTER".to_string()));
    assert_eq!(d.alignment(1), Ok("LEFT".to_string()));
}

#[test]
fn should_inherit_alignment_when_replacements_contain_newlines() {
    let d = sut!(create());
    d.insert(0, "a,b\nc,d").unwrap();
    d.set_alignment(0, "CENTER").unwrap();
    d.set_alignment(1, "RIGHT").unwrap();

    assert_eq!(d.replace_all(",", "\n", true), Ok(2));

    assert_eq!(d.text(), "a\nb\nc\nd");
    assert_eq!(d.line_count(), 4);
    assert_eq!(d.alignment(0), Ok("CENTER".to_string()));
    assert_eq!(d.alignment(1), Ok("CENTER".to_string()));
    assert_eq!(d.alignment(2), Ok("RIGHT".to_string()));
    assert_eq!(d.alignment(3), Ok("RIGHT".to_string()));

    assert_eq!(d.replace_all("\n", " ", true), Ok(3));
    assert_eq!(d.text(), "a b c d");
    assert_eq!(d.line_count(), 1);
    assert_eq!(d.alignment(0), Ok("CENTER".to_string()));
}

#[test]
fn should_leave_the_document_untouched_when_any_operation_fails() {
    let d = sut!(create());
    d.insert(0, "one\ntwo").unwrap();
    d.apply_format(0, 3, &[("bold", "true")]).unwrap();
    d.set_alignment(1, "RIGHT").unwrap();
    let runs = d.runs();

    assert!(d.delete(3, 99).is_err());
    assert!(d.insert(99, "x\ny").is_err());
    assert!(d.apply_format(0, 7, &[("color", "red"), ("size", "-1")]).is_err());
    assert!(d.replace_all("", "x", true).is_err());
    assert!(d.set_alignment(5, "LEFT").is_err());
    assert!(d.set_alignment(0, "BOGUS").is_err());

    assert_eq!(d.text(), "one\ntwo");
    assert_eq!(d.runs(), runs);
    assert_eq!(d.line_count(), 2);
    assert_eq!(d.alignment(1), Ok("RIGHT".to_string()));
    assert_eq!(d.alignment(0), Ok("LEFT".to_string()));
}

#[test]
fn should_handle_patterns_longer_than_the_text_and_the_whole_text() {
    let d = sut!(create());

    assert_eq!(d.find("a", true), Ok(vec![]));
    assert_eq!(d.replace_all("a", "b", true), Ok(0));

    d.insert(0, "abc").unwrap();
    assert_eq!(d.find("abcd", true), Ok(vec![]));
    assert_eq!(d.replace_all("abcd", "x", true), Ok(0));
    assert_eq!(d.replace_all("abc", "xyz123", true), Ok(1));
    assert_eq!(d.text(), "xyz123");
    assert_eq!(d.runs(), vec![format!("0-6:{DEF}")]);
}

#[test]
fn should_replace_in_a_million_character_document_quickly() {
    let start = Instant::now();
    let d = sut!(create());
    d.insert(0, &"abcdefghij".repeat(100_000)).unwrap();
    for i in 0..10usize {
        d.apply_format(i * 100_000, i * 100_000 + 50_000, &[("color", &format!("c{i}"))]).unwrap();
    }
    assert_eq!(d.runs().len(), 20);

    assert_eq!(d.find("jab", true).unwrap().len(), 99_999);
    for i in 0..50usize {
        d.insert(i * 16_000 + 7, "NEEDLE").unwrap();
    }
    assert_eq!(d.runs().len(), 20);
    assert_eq!(d.find("NEEDLE", true).unwrap().len(), 50);
    assert_eq!(d.replace_all("needle", "N", false), Ok(50));

    assert_eq!(d.length(), 1_000_050);
    assert_eq!(d.runs().len(), 20);
    assert_eq!(d.distinct_format_count(), 11);
    assert_eq!(d.format_at(0), Ok("Calibri|11|-|-|-|c0".to_string()));
    assert!(start.elapsed().as_millis() < 30_000);
}

#[test]
fn should_keep_text_runs_and_lines_consistent_under_concurrent_editing() {
    let d = sut!(create());
    d.insert(0, "line one\nline two\nline three\n").unwrap();

    run_concurrently(8, |t| {
        for i in 0..500usize {
            match t % 4 {
                0 => d.insert(0, "ab\n").unwrap(),
                1 => d.insert(d.length(), "cd").unwrap(),
                2 => {
                    let n = d.length();
                    d.apply_format(n / 4, n / 2, &[("bold", if i % 2 == 0 { "true" } else { "false" })])
                        .unwrap();
                }
                _ => {
                    let runs = d.runs();
                    let text = d.text();
                    assert!(d.line_count() >= 4);
                    assert!(!runs.is_empty());
                    assert!(text.len() >= 29);
                }
            }
        }
    });

    assert_eq!(d.length(), 29 + 2 * 500 * 3 + 2 * 500 * 2);
    assert_eq!(d.line_count(), 3 + 1 + 2 * 500);
    let runs = d.runs();
    assert!(runs[0].starts_with("0-"));
    assert!(runs[runs.len() - 1].contains(&format!("-{}:", d.length())));
}
