#![allow(unused_imports)]
use p117_query_pipeline::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const DICT: &[&str] = &[
    "phone", "case", "cheap", "charger", "cable", "wireless", "usb", "deals", "laptop", "car", "cat",
];
const SYN: &[(&str, &[&str])] = &[
    ("phone", &["mobile", "cell"]),
    ("cheap", &["budget"]),
    ("laptop", &["notebook", "laptop", "notebook"]),
    ("mobile", &["handset"]),
];
const BANNED: &[&str] = &["free", "spam"];
const CORPUS: &[(&str, &str)] = &[
    ("d1", "cheap phone case"),
    ("d2", "wireless phone charger"),
    ("d3", "usb charger cable"),
    ("d4", "Budget mobile deals"),
    ("d5", "laptop usb cable"),
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

fn failing(name: &str) -> Arc<dyn Step> {
    Arc::new(FnStep {
        name: name.to_string(),
        f: Box::new(|_| Err("kaput".to_string())),
    })
}

fn appending(name: &str, word: &str) -> Arc<dyn Step> {
    let word = word.to_string();
    step(name, move |q| format!("{q} {word}"))
}

fn v(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn pipeline() -> Option<Arc<dyn QueryPipeline>> {
    create(DICT, SYN, BANNED, CORPUS)
}

#[test]
fn should_trim_collapse_and_lower_case_in_parse() {
    let p = sut!(pipeline());
    let r = p.process("  CASE \t  Charger  ").unwrap();

    assert_eq!(r.query, "case charger");
    assert_eq!(r.trace[0], "parse");
}

#[test]
fn should_not_trace_parse_when_the_query_is_already_clean() {
    let p = sut!(pipeline());
    let r = p.process("usb").unwrap();

    assert_eq!(r.query, "usb");
    assert!(!r.trace.contains(&"parse".to_string()));
}

#[test]
fn should_reject_blank_queries() {
    let p = sut!(pipeline());

    assert_eq!(p.process(""), Err(PipelineError::InvalidArgument));
    assert_eq!(p.process(" \t "), Err(PipelineError::InvalidArgument));
}

#[test]
fn should_correct_words_within_one_edit() {
    let p = sut!(pipeline());

    assert_eq!(p.process("chargr").unwrap().query, "charger"); // insertion
    assert_eq!(p.process("usbb").unwrap().query, "usb"); // deletion
    assert_eq!(p.process("wirelass").unwrap().query, "wireless"); // substitution
    assert!(p.process("chargr").unwrap().trace.contains(&"spell".to_string()));
}

#[test]
fn should_leave_dictionary_words_and_far_words_alone() {
    let p = sut!(pipeline());

    let in_dict = p.process("car").unwrap(); // one edit from "cat" but a dictionary word itself
    assert_eq!(in_dict.query, "car");
    assert!(!in_dict.trace.contains(&"spell".to_string()));
    assert_eq!(p.process("zzzz").unwrap().query, "zzzz");
    assert_eq!(p.process("cabel").unwrap().query, "cabel"); // a swap is two edits
}

#[test]
fn should_leave_ambiguous_misspellings_unchanged() {
    let p = sut!(pipeline());
    let r = p.process("cap").unwrap(); // one edit from both "car" and "cat"

    assert_eq!(r.query, "cap");
    assert!(!r.trace.contains(&"spell".to_string()));
}

#[test]
fn should_expand_synonyms_into_an_or_group_keeping_the_original_first() {
    let p = sut!(pipeline());

    assert_eq!(p.process("phone").unwrap().query, "(phone OR mobile OR cell)");
    assert_eq!(p.process("cheap case").unwrap().query, "(cheap OR budget) case");
    assert!(p.process("phone").unwrap().trace.contains(&"synonyms".to_string()));
}

#[test]
fn should_drop_duplicate_synonyms_and_not_expand_synonyms_of_synonyms() {
    let p = sut!(pipeline());

    assert_eq!(p.process("laptop").unwrap().query, "(laptop OR notebook)");
    assert!(!p.process("phone").unwrap().query.contains("handset"));
    assert_eq!(p.process("mobile").unwrap().query, "(mobile OR handset)");
}

#[test]
fn should_correct_spelling_before_expanding_synonyms() {
    let p = sut!(pipeline());

    assert_eq!(p.process("PHON").unwrap().query, "(phone OR mobile OR cell)");
}

#[test]
fn should_remove_banned_words_in_the_filter_step() {
    let p = sut!(pipeline());
    let r = p.process("free case spam").unwrap();

    assert_eq!(r.query, "case");
    assert!(r.trace.contains(&"filter".to_string()));
}

#[test]
fn should_return_an_empty_result_when_everything_is_filtered() {
    let p = sut!(pipeline());
    let r = p.process("Free  SPAM").unwrap();

    assert_eq!(r.query, "");
    assert!(r.results.is_empty());
    assert_eq!(r.trace, v(&["parse", "filter"]));
}

#[test]
fn should_rank_by_distinct_terms_matched_then_by_doc_id() {
    let p = sut!(pipeline());
    let r = p.process("phone charger").unwrap();

    assert_eq!(
        r.results,
        v(&[
            "d2: wireless [phone] [charger]",
            "d1: cheap [phone] case",
            "d3: usb [charger] cable",
            "d4: Budget [mobile] deals",
        ])
    );
}

#[test]
fn should_count_a_repeated_term_only_once() {
    let p = sut!(pipeline());
    let r = p.process("cable cable charger").unwrap();

    assert_eq!(
        r.results,
        v(&[
            "d3: usb [charger] [cable]",
            "d2: wireless phone [charger]",
            "d5: laptop usb [cable]",
        ])
    );
}

#[test]
fn should_count_an_or_group_as_one_term_matched_by_any_alternative() {
    let p = sut!(pipeline());
    let r = p.process("cheap").unwrap();

    assert_eq!(r.results, v(&["d1: [cheap] phone case", "d4: [Budget] mobile deals"]));
}

#[test]
fn should_return_no_results_and_no_rank_trace_when_nothing_matches() {
    let p = sut!(pipeline());
    let r = p.process("zzzz").unwrap();

    assert!(r.results.is_empty());
    assert!(!r.trace.contains(&"rank".to_string()));
    assert!(!r.trace.contains(&"highlight".to_string()));
}

#[test]
fn should_trace_every_step_that_changed_something_in_order() {
    let p = sut!(pipeline());
    let r = p.process("  Cheep   PHONE  ").unwrap();

    assert_eq!(r.query, "(cheap OR budget) (phone OR mobile OR cell)");
    assert_eq!(r.trace, v(&["parse", "spell", "synonyms", "rank", "highlight"]));
    assert_eq!(
        r.results,
        v(&[
            "d1: [cheap] [phone] case",
            "d4: [Budget] [mobile] deals",
            "d2: wireless [phone] charger",
        ])
    );
}

#[test]
fn should_run_a_registered_step_at_the_end_of_the_query_phase() {
    let p = sut!(pipeline());
    p.register(appending("add-usb", "usb")).unwrap();

    let r = p.process("free case").unwrap();

    assert_eq!(r.query, "case usb");
    assert_eq!(r.trace, v(&["filter", "add-usb", "rank", "highlight"]));
}

#[test]
fn should_run_a_step_registered_before_or_after_a_built_in_at_that_position() {
    let p = sut!(pipeline());
    p.register_before("synonyms", step("alias", |q| q.replace("handset", "phone")))
        .unwrap();
    p.register_after("filter", step("trailing", |q| format!("{q} case")))
        .unwrap();

    let r = p.process("handset").unwrap();

    assert_eq!(r.query, "(phone OR mobile OR cell) case");
    assert_eq!(r.trace, v(&["alias", "synonyms", "trailing", "rank", "highlight"]));
}

#[test]
fn should_hand_the_raw_query_to_a_step_registered_before_parse() {
    let p = sut!(pipeline());
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let s2 = seen.clone();
    p.register_before(
        "parse",
        step("spy", move |q| {
            s2.lock().unwrap().push(q.to_string());
            q.to_string()
        }),
    )
    .unwrap();

    let r = p.process("  Case ").unwrap();

    assert_eq!(*seen.lock().unwrap(), v(&["  Case "]));
    assert_eq!(r.query, "case");
    assert!(!r.trace.contains(&"spy".to_string()));
}

#[test]
fn should_keep_separate_pipeline_instances_independent() {
    let a = sut!(pipeline());
    let b = sut!(pipeline());
    a.register(appending("add-usb", "usb")).unwrap();
    a.remove_step("spell").unwrap();

    assert_eq!(b.process("chargr").unwrap().query, "charger");
    assert_eq!(a.process("chargr").unwrap().query, "chargr usb");
}

#[test]
fn should_not_run_a_removed_step_and_allow_reusing_its_name() {
    let p = sut!(pipeline());
    p.remove_step("spell").unwrap();
    assert_eq!(p.process("chargr").unwrap().query, "chargr");

    p.register(appending("tag", "usb")).unwrap();
    p.remove_step("tag").unwrap();
    assert_eq!(p.process("case").unwrap().query, "case");
    p.register(appending("tag", "cable")).unwrap();
    assert_eq!(p.process("case").unwrap().query, "case cable");
}

#[test]
fn should_skip_a_disabled_step_until_it_is_enabled_again() {
    let p = sut!(pipeline());
    p.disable_step("synonyms").unwrap();

    let off = p.process("phone").unwrap();
    assert_eq!(off.query, "phone");
    assert!(!off.trace.contains(&"synonyms".to_string()));

    p.enable_step("synonyms").unwrap();
    p.enable_step("synonyms").unwrap();
    assert_eq!(p.process("phone").unwrap().query, "(phone OR mobile OR cell)");
}

#[test]
fn should_leave_results_empty_without_rank_and_unhighlighted_without_highlight() {
    let p = sut!(pipeline());
    p.disable_step("highlight").unwrap();
    let plain = p.process("cheap").unwrap();
    assert_eq!(plain.results, v(&["d1: cheap phone case", "d4: Budget mobile deals"]));
    assert_eq!(plain.trace, v(&["synonyms", "rank"]));

    p.remove_step("rank").unwrap();
    let none = p.process("cheap").unwrap();
    assert!(none.results.is_empty());
    assert_eq!(none.query, "(cheap OR budget)");
}

#[test]
fn should_skip_a_failing_step_record_it_as_failed_and_continue() {
    let p = sut!(pipeline());
    p.register_after("parse", failing("boom")).unwrap();
    p.register_after("boom", failing("fails-too")).unwrap();

    let r = p.process("CHEEP").unwrap();

    assert_eq!(r.query, "(cheap OR budget)");
    assert_eq!(
        r.trace,
        v(&["parse", "boom:failed", "fails-too:failed", "spell", "synonyms", "rank", "highlight"])
    );
}

#[test]
fn should_stop_transforming_and_return_nothing_when_a_step_empties_the_query() {
    let p = sut!(pipeline());
    p.register_after("parse", step("wipe", |_| String::new())).unwrap();
    p.register(appending("never", "usb")).unwrap();

    let r = p.process("CASE").unwrap();

    assert_eq!(r.query, "");
    assert!(r.results.is_empty());
    assert_eq!(r.trace, v(&["parse", "wipe"]));
}

#[test]
fn should_reject_bad_registrations_and_leave_the_pipeline_unchanged() {
    let p = sut!(pipeline());
    p.register(appending("tag", "usb")).unwrap();
    let bad = Err(PipelineError::InvalidArgument);

    assert_eq!(p.register(appending(" ", "x")), bad);
    assert_eq!(p.register(appending("tag", "x")), bad);
    assert_eq!(p.register(appending("spell", "x")), bad);
    assert_eq!(p.register_before("nope", appending("a", "x")), bad);
    assert_eq!(p.register_after("rank", appending("a", "x")), bad);
    assert_eq!(p.register_before("highlight", appending("a", "x")), bad);

    assert_eq!(p.process("case").unwrap().query, "case usb");
}

#[test]
fn should_reject_unknown_step_names_when_removing_disabling_or_enabling() {
    let p = sut!(pipeline());
    let bad = Err(PipelineError::InvalidArgument);

    assert_eq!(p.remove_step("nope"), bad);
    assert_eq!(p.disable_step("nope"), bad);
    assert_eq!(p.enable_step("nope"), bad);
    p.remove_step("filter").unwrap();
    assert_eq!(p.remove_step("filter"), bad);
    assert_eq!(p.register_after("filter", appending("a", "x")), bad);
}

#[test]
fn should_return_independent_results_per_call() {
    let p = sut!(pipeline());
    let first = p.process("cheap").unwrap();
    let snapshot = first.clone();

    p.process("usb").unwrap();
    p.register(appending("tag", "usb")).unwrap();
    p.process("case").unwrap();

    assert_eq!(first, snapshot);
    assert_eq!(first.trace, v(&["synonyms", "rank", "highlight"]));
}

#[test]
fn should_process_concurrently_while_steps_are_registered_and_removed() {
    let p = sut!(pipeline());
    let bad = AtomicUsize::new(0);
    let expected = p.process("cheep phone").unwrap().results;

    run_concurrently(9, |t| {
        if t == 8 {
            for i in 0..300 {
                let n = format!("noop-{i}");
                p.register(step(&n, |q| q.to_string())).unwrap();
                p.disable_step(&n).unwrap();
                p.enable_step(&n).unwrap();
                p.remove_step(&n).unwrap();
            }
            return;
        }
        for _ in 0..2_000 {
            let r = p.process("cheep phone").unwrap();
            if r.results != expected || r.query != "(cheap OR budget) (phone OR mobile OR cell)" {
                bad.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(bad.load(Ordering::SeqCst), 0);
    assert_eq!(
        p.process("cheep phone").unwrap().trace,
        v(&["spell", "synonyms", "rank", "highlight"])
    );
}
