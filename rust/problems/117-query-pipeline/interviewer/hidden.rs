#![allow(unused_imports)]
use p117_query_pipeline::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const DICT: &[&str] = &["phone", "case", "cheap", "charger", "usb", "car", "cat"];
const SYN: &[(&str, &[&str])] = &[("phone", &["mobile", "cell"])];
const CORPUS: &[(&str, &str)] = &[
    ("d10", "usb usb cable"),
    ("d2", "usb cable"),
    ("d1", "Phone CASE"),
    ("d3", "mobile phone"),
];

struct FnStep {
    name: String,
    f: Box<dyn Fn(&str) -> Result<String, String> + Send + Sync>,
}

impl Step for FnStep {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn transform(&self, query: &str) -> Result<String, String> {
        (self.f)(query)
    }
}

fn step(name: &str, f: impl Fn(&str) -> String + Send + Sync + 'static) -> Arc<dyn Step> {
    Arc::new(FnStep {
        name: name.to_string(),
        f: Box::new(move |q| Ok(f(q))),
    })
}

fn v(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn pipeline(banned: &[&str]) -> Option<Arc<dyn QueryPipeline>> {
    create(DICT, SYN, banned, CORPUS)
}

#[test]
fn should_filter_banned_words_inside_synonym_groups_and_collapse_them() {
    let _ = sut!(pipeline(&[]));
    let q = |banned: &[&str], raw: &str| pipeline(banned).unwrap().process(raw).unwrap().query;
    assert_eq!(q(&["cell"], "phone"), "(phone OR mobile)");
    assert_eq!(q(&["mobile", "cell"], "phone"), "phone");
    assert_eq!(q(&["phone"], "phone"), "(mobile OR cell)");
    assert_eq!(q(&["phone", "mobile", "cell"], "phone case"), "case");
}

#[test]
fn should_break_score_ties_by_doc_id_as_text() {
    let p = sut!(pipeline(&[]));
    let r = p.process("usb").unwrap();

    assert_eq!(r.results, v(&["d10: [usb] [usb] cable", "d2: [usb] cable"]));
}

#[test]
fn should_highlight_every_occurrence_keeping_the_documents_own_case() {
    let p = sut!(pipeline(&[]));
    let r = p.process("phone case").unwrap();

    assert_eq!(r.results, v(&["d1: [Phone] [CASE]", "d3: [mobile] [phone]"]));
}

#[test]
fn should_not_correct_a_correction_again_or_touch_words_that_are_dictionary_words() {
    let p = sut!(pipeline(&[]));

    assert_eq!(p.process("phon").unwrap().query, "(phone OR mobile OR cell)");
    let r = p.process("car cat cap").unwrap();
    assert_eq!(r.query, "car cat cap");
    assert!(!r.trace.contains(&"spell".to_string()));
}

#[test]
fn should_place_steps_exactly_where_they_were_registered() {
    let p = sut!(pipeline(&[]));
    let add = |n: &'static str| step(n, move |q| format!("{q} {n}"));
    p.register_after("parse", add("a")).unwrap();
    p.register_after("parse", add("b")).unwrap();
    p.register(add("c")).unwrap();
    p.register(add("d")).unwrap();
    p.register_before("filter", add("e")).unwrap();
    p.register_after("synonyms", add("f")).unwrap();
    p.register_before("a", add("g")).unwrap();

    let r = p.process("zzzz").unwrap();

    assert_eq!(r.query, "zzzz b g a f e c d");
    assert_eq!(r.trace, v(&["b", "g", "a", "f", "e", "c", "d"]));
}

#[test]
fn should_filter_what_earlier_steps_add_but_not_what_later_steps_add() {
    let p = sut!(pipeline(&["spam"]));
    p.register_before("filter", step("early", |q| format!("{q} spam"))).unwrap();
    p.register(step("late", |q| format!("{q} spam"))).unwrap();

    let r = p.process("usb").unwrap();

    assert_eq!(r.query, "usb spam");
    assert_eq!(r.trace, v(&["early", "filter", "late", "rank", "highlight"]));
}

struct Failing(Arc<Mutex<Vec<String>>>);

impl Step for Failing {
    fn name(&self) -> String {
        "explode".to_string()
    }
    fn transform(&self, query: &str) -> Result<String, String> {
        self.0.lock().unwrap().push(query.to_string());
        Err("boom".to_string())
    }
}

#[test]
fn should_keep_going_after_a_step_fails_and_leave_the_query_as_it_was() {
    let p = sut!(pipeline(&[]));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let s2 = seen.clone();
    p.register_after("synonyms", Arc::new(Failing(seen.clone()))).unwrap();
    p.register(step("after", move |q| {
        s2.lock().unwrap().push(q.to_string());
        q.to_string()
    }))
    .unwrap();

    let r = p.process("phon").unwrap();

    let group = "(phone OR mobile OR cell)";
    assert_eq!(*seen.lock().unwrap(), v(&[group, group]));
    assert_eq!(r.trace, v(&["spell", "synonyms", "explode:failed", "rank", "highlight"]));
    assert_eq!(r.query, group);
}

#[test]
fn should_treat_every_step_as_optional_including_parse() {
    let p = sut!(pipeline(&[]));
    p.remove_step("parse").unwrap();
    p.disable_step("spell").unwrap();

    let r = p.process("USB").unwrap();

    assert_eq!(r.query, "USB");
    assert!(!r.trace.contains(&"parse".to_string()));
    assert!(!r.trace.contains(&"spell".to_string()));
    assert_eq!(p.process("usb").unwrap().results.len(), 2);
}

#[test]
fn should_give_every_concurrent_call_a_consistent_step_order_while_steps_change() {
    let p = sut!(pipeline(&["spam"]));
    let bad = AtomicUsize::new(0);

    run_concurrently(10, |t| {
        if t >= 8 {
            for i in 0..400 {
                let n = format!("t{t}-{i}");
                p.register_before("filter", step(&n, |q| format!("{q} spam"))).unwrap();
                p.disable_step(&n).unwrap();
                p.enable_step(&n).unwrap();
                p.remove_step(&n).unwrap();
            }
            return;
        }
        for _ in 0..3_000 {
            let r = p.process("  USB ").unwrap();
            let rank = r.trace.iter().position(|s| s == "rank");
            let parse_first = r.trace.first().map(|s| s == "parse").unwrap_or(false);
            let ok = parse_first
                && rank.is_some()
                && r.trace.get(rank.unwrap() + 1).map(|s| s == "highlight").unwrap_or(false)
                && r.query == "usb";
            if !ok {
                bad.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(bad.load(Ordering::SeqCst), 0);
}
