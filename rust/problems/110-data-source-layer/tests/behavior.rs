#![allow(unused_imports, clippy::needless_range_loop)]
use p110_data_source_layer::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

const SOURCES: [&str; 3] = ["DATABASE", "API", "FILE"];

fn chars(n: usize) -> String {
    "x".repeat(n)
}

#[test]
fn should_start_on_database_and_empty() {
    let d = sut!(create());

    assert_eq!(d.current_source(), "DATABASE");
    assert_eq!(d.size(), 0);
    assert_eq!(d.get("anything"), Ok(None));
}

#[test]
fn should_store_and_return_a_value() {
    let d = sut!(create());
    d.put("user:1", "alice").unwrap();

    assert_eq!(d.get("user:1"), Ok(Some("alice".to_string())));
    assert_eq!(d.size(), 1);
}

#[test]
fn should_overwrite_without_growing() {
    let d = sut!(create());
    d.put("k", "a").unwrap();
    d.put("k", "b").unwrap();

    assert_eq!(d.get("k"), Ok(Some("b".to_string())));
    assert_eq!(d.size(), 1);
}

#[test]
fn should_accept_the_empty_string_as_a_value() {
    let d = sut!(create());
    d.put("k", "").unwrap();

    assert_eq!(d.get("k"), Ok(Some(String::new())));
    assert_eq!(d.size(), 1);
}

#[test]
fn should_report_whether_remove_deleted_something() {
    let d = sut!(create());
    d.put("k", "v").unwrap();

    assert_eq!(d.remove("k"), Ok(true));
    assert_eq!(d.remove("k"), Ok(false));
    assert_eq!(d.remove("never-there"), Ok(false));
    assert_eq!(d.get("k"), Ok(None));
    assert_eq!(d.size(), 0);
}

#[test]
fn should_reject_blank_keys_on_every_operation_in_every_source() {
    let d = sut!(create());
    for source in SOURCES {
        d.switch_source(source).unwrap();
        for bad in ["", "   ", "\t"] {
            assert_eq!(d.get(bad), Err(DataError::InvalidKey));
            assert_eq!(d.put(bad, "v"), Err(DataError::InvalidKey));
            assert_eq!(d.remove(bad), Err(DataError::InvalidKey));
        }
        assert_eq!(d.size(), 0);
    }
}

#[test]
fn should_reject_unknown_source_names_and_keep_the_current_one() {
    let d = sut!(create());
    d.switch_source("FILE").unwrap();

    for bad in ["", " ", "SQL", "api", "Database", " FILE", "FILE "] {
        assert_eq!(d.switch_source(bad), Err(DataError::UnknownSource));
        assert_eq!(d.current_source(), "FILE");
    }
}

#[test]
fn should_report_the_current_source_after_each_switch() {
    let d = sut!(create());

    for source in ["API", "FILE", "DATABASE"] {
        d.switch_source(source).unwrap();
        assert_eq!(d.current_source(), source);
    }
}

#[test]
fn should_keep_the_records_of_each_source_independent() {
    let d = sut!(create());
    d.put("k", "db").unwrap();
    d.put("only-db", "1").unwrap();

    d.switch_source("API").unwrap();
    assert_eq!(d.get("k"), Ok(None));
    assert_eq!(d.size(), 0);
    d.put("k", "api").unwrap();

    d.switch_source("FILE").unwrap();
    assert_eq!(d.get("k"), Ok(None));
    assert_eq!(d.get("only-db"), Ok(None));
    d.put("k", "file").unwrap();

    assert_eq!(d.get("k"), Ok(Some("file".to_string())));
    d.switch_source("API").unwrap();
    assert_eq!(d.get("k"), Ok(Some("api".to_string())));
    d.switch_source("DATABASE").unwrap();
    assert_eq!(d.get("k"), Ok(Some("db".to_string())));
    assert_eq!(d.get("only-db"), Ok(Some("1".to_string())));
}

#[test]
fn should_show_the_original_records_again_when_switching_back() {
    let d = sut!(create());
    d.put("a", "1").unwrap();
    d.put("b", "2").unwrap();

    d.switch_source("API").unwrap();
    d.put("c", "3").unwrap();
    d.switch_source("DATABASE").unwrap();

    assert_eq!(d.size(), 2);
    assert_eq!(d.get("a"), Ok(Some("1".to_string())));
    assert_eq!(d.get("b"), Ok(Some("2".to_string())));
    assert_eq!(d.get("c"), Ok(None));
}

#[test]
fn should_not_touch_records_when_switching_to_the_current_source() {
    let d = sut!(create());
    d.put("a", "1").unwrap();

    d.switch_source("DATABASE").unwrap();

    assert_eq!(d.get("a"), Ok(Some("1".to_string())));
    assert_eq!(d.size(), 1);
}

#[test]
fn should_remove_only_from_the_current_source() {
    let d = sut!(create());
    d.put("k", "db").unwrap();
    d.switch_source("FILE").unwrap();
    d.put("k", "file").unwrap();

    assert_eq!(d.remove("k"), Ok(true));
    assert_eq!(d.get("k"), Ok(None));

    d.switch_source("DATABASE").unwrap();
    assert_eq!(d.get("k"), Ok(Some("db".to_string())));
    assert_eq!(d.size(), 1);
}

#[test]
fn should_accept_api_values_up_to_thirty_two_characters_and_refuse_longer() {
    let d = sut!(create());
    d.switch_source("API").unwrap();

    d.put("ok", &chars(32)).unwrap();
    assert_eq!(d.get("ok"), Ok(Some(chars(32))));

    assert_eq!(d.put("too-long", &chars(33)), Err(DataError::ValueTooLong));
    assert_eq!(d.get("too-long"), Ok(None));
    assert_eq!(d.size(), 1);
}

#[test]
fn should_not_limit_value_length_on_database_and_file() {
    let d = sut!(create());

    d.put("big", &chars(10_000)).unwrap();
    assert_eq!(d.get("big"), Ok(Some(chars(10_000))));
    d.switch_source("FILE").unwrap();
    d.put("big", &chars(33)).unwrap();
    assert_eq!(d.get("big"), Ok(Some(chars(33))));
}

#[test]
fn should_treat_api_keys_case_insensitively_but_not_trim_them() {
    let d = sut!(create());
    d.switch_source("API").unwrap();

    d.put("Token", "t1").unwrap();
    assert_eq!(d.get("token"), Ok(Some("t1".to_string())));
    assert_eq!(d.get("TOKEN"), Ok(Some("t1".to_string())));
    d.put("tOKEN", "t2").unwrap();
    assert_eq!(d.size(), 1);
    assert_eq!(d.get("Token"), Ok(Some("t2".to_string())));

    d.put(" token", "spaced").unwrap();
    assert_eq!(d.size(), 2);
    assert_eq!(d.get("token"), Ok(Some("t2".to_string())));
    assert_eq!(d.remove("TokeN"), Ok(true));
    assert_eq!(d.get("token"), Ok(None));
}

#[test]
fn should_trim_file_keys_and_stay_case_sensitive() {
    let d = sut!(create());
    d.switch_source("FILE").unwrap();

    d.put("  report ", "v1").unwrap();
    assert_eq!(d.get("report"), Ok(Some("v1".to_string())));
    assert_eq!(d.get("\treport\n"), Ok(Some("v1".to_string())));
    d.put("report", "v2").unwrap();
    assert_eq!(d.size(), 1);

    d.put("Report", "other").unwrap();
    assert_eq!(d.size(), 2);
    assert_eq!(d.get("report"), Ok(Some("v2".to_string())));
    assert_eq!(d.get("Report"), Ok(Some("other".to_string())));
    assert_eq!(d.remove(" report  "), Ok(true));
    assert_eq!(d.get("report"), Ok(None));
}

#[test]
fn should_use_database_keys_exactly_as_given() {
    let d = sut!(create());
    d.put("a", "1").unwrap();
    d.put("A", "2").unwrap();
    d.put(" a", "3").unwrap();
    d.put("a ", "4").unwrap();

    assert_eq!(d.size(), 4);
    assert_eq!(d.get("a"), Ok(Some("1".to_string())));
    assert_eq!(d.get("A"), Ok(Some("2".to_string())));
    assert_eq!(d.get(" a"), Ok(Some("3".to_string())));
    assert_eq!(d.get("a "), Ok(Some("4".to_string())));
}

#[test]
fn should_leave_state_untouched_when_a_put_is_refused() {
    let d = sut!(create());
    d.switch_source("API").unwrap();
    d.put("k", "good").unwrap();

    assert_eq!(d.put("k", &chars(33)), Err(DataError::ValueTooLong));
    assert_eq!(d.put(" ", "v"), Err(DataError::InvalidKey));

    assert_eq!(d.get("k"), Ok(Some("good".to_string())));
    assert_eq!(d.size(), 1);
}

#[test]
fn should_count_size_per_source() {
    let d = sut!(create());
    d.put("a", "1").unwrap();
    d.put("b", "2").unwrap();
    d.put("c", "3").unwrap();
    d.switch_source("API").unwrap();
    d.put("a", "1").unwrap();
    d.switch_source("FILE").unwrap();

    assert_eq!(d.size(), 0);
    d.switch_source("API").unwrap();
    assert_eq!(d.size(), 1);
    d.switch_source("DATABASE").unwrap();
    assert_eq!(d.size(), 3);
}

#[test]
fn should_keep_writes_from_many_threads_on_a_single_source() {
    let d = sut!(create());
    d.switch_source("API").unwrap();

    run_concurrently(16, |t| {
        for i in 0..500 {
            d.put(&format!("t{t}-k{i}"), &format!("v{i}")).unwrap();
        }
    });

    assert_eq!(d.size(), 16 * 500);
    assert_eq!(d.get("t7-k123"), Ok(Some("v123".to_string())));
}

#[test]
fn should_never_fail_a_read_while_another_thread_switches_sources() {
    let d = sut!(create());
    for source in SOURCES {
        d.switch_source(source).unwrap();
        d.put("shared", source).unwrap();
    }
    d.switch_source("DATABASE").unwrap();
    let seen = Mutex::new(HashSet::new());
    let missing = AtomicUsize::new(0);

    run_concurrently(5, |t| {
        if t == 0 {
            for i in 0..3_000 {
                d.switch_source(SOURCES[i % 3]).unwrap();
            }
            return;
        }
        for _ in 0..20_000 {
            match d.get("shared").unwrap() {
                Some(v) => {
                    seen.lock().unwrap().insert(v);
                }
                None => {
                    missing.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    });

    assert_eq!(missing.load(Ordering::Relaxed), 0);
    assert!(seen
        .lock()
        .unwrap()
        .iter()
        .all(|v| SOURCES.contains(&v.as_str())));
}
