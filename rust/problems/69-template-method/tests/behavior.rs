use p69_template_method::contract::*;
use p69_template_method::{CsvImporter, DataImporter};
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

fn report(read: usize, saved: usize, skipped: usize, failed: usize) -> ImportReport {
    ImportReport { read, saved, skipped, failed }
}

/// An implementor written in the test: records every step in order so the template can be checked.
#[derive(Default)]
struct Recording {
    events: Vec<String>,
    input: VecDeque<String>,
    invalid: HashSet<String>,
    fail_transform_on: Option<String>,
    fail_save_on: Option<String>,
    fail_read_on: Option<String>,
    fail_open: bool,
}

impl Recording {
    fn new(records: &[&str]) -> Self {
        Recording { input: records.iter().map(|s| s.to_string()).collect(), ..Default::default() }
    }
}

impl DataImporter for Recording {
    fn open(&mut self) -> Result<(), ImportError> {
        self.events.push("open".into());
        if self.fail_open { Err(ImportError("cannot open".into())) } else { Ok(()) }
    }
    fn read_record(&mut self) -> Result<Option<String>, ImportError> {
        self.events.push("read".into());
        match self.input.pop_front() {
            None => Ok(None),
            Some(next) if Some(&next) == self.fail_read_on.as_ref() => Err(ImportError("read failed".into())),
            Some(next) => Ok(Some(next)),
        }
    }
    fn validate(&mut self, record: &str) -> bool {
        self.events.push(format!("validate:{record}"));
        !self.invalid.contains(record)
    }
    fn transform(&mut self, record: &str) -> Result<String, ImportError> {
        self.events.push(format!("transform:{record}"));
        if self.fail_transform_on.as_deref() == Some(record) { Err(ImportError("transform failed".into())) } else { Ok(record.to_uppercase()) }
    }
    fn save(&mut self, record: &str) -> Result<(), ImportError> {
        self.events.push(format!("save:{record}"));
        if self.fail_save_on.as_deref() == Some(record) { Err(ImportError("save failed".into())) } else { Ok(()) }
    }
    fn close(&mut self) {
        self.events.push("close".into());
    }
}

#[test]
fn should_run_the_steps_in_the_fixed_order() {
    let mut importer = Recording::new(&["a", "b"]);

    let r = importer.run().unwrap();

    assert_eq!(
        importer.events,
        vec!["open", "read", "validate:a", "transform:a", "save:A", "read", "validate:b", "transform:b", "save:B", "read", "close"]
    );
    assert_eq!(r, report(2, 2, 0, 0));
}

#[test]
fn should_skip_invalid_records_without_transforming_or_saving_them() {
    let mut importer = Recording::new(&["a", "bad", "c"]);
    importer.invalid.insert("bad".into());

    let r = importer.run().unwrap();

    assert!(!importer.events.contains(&"transform:bad".to_string()));
    assert!(!importer.events.contains(&"save:BAD".to_string()));
    assert_eq!(r, report(3, 2, 1, 0));
}

#[test]
fn should_count_a_transform_failure_and_carry_on() {
    let mut importer = Recording::new(&["a", "boom", "c"]);
    importer.fail_transform_on = Some("boom".into());

    let r = importer.run().unwrap();

    assert!(!importer.events.contains(&"save:BOOM".to_string()));
    assert!(importer.events.contains(&"save:A".to_string()) && importer.events.contains(&"save:C".to_string()));
    assert_eq!(r, report(3, 2, 0, 1));
}

#[test]
fn should_count_a_save_failure_and_carry_on() {
    let mut importer = Recording::new(&["a", "b", "c"]);
    importer.fail_save_on = Some("B".into());

    let r = importer.run().unwrap();

    assert!(importer.events.contains(&"save:B".to_string()) && importer.events.contains(&"save:C".to_string()));
    assert_eq!(r, report(3, 2, 0, 1));
}

#[test]
fn should_always_close_when_reading_fails_and_return_the_failure() {
    let mut importer = Recording::new(&["a", "oops", "c"]);
    importer.fail_read_on = Some("oops".into());

    let result = importer.run();

    assert_eq!(result, Err(ImportError("read failed".into())));
    assert_eq!(importer.events.last().unwrap(), "close");
    assert_eq!(importer.events.iter().filter(|e| *e == "close").count(), 1);
    assert!(importer.events.contains(&"save:A".to_string()));
}

#[test]
fn should_not_close_what_never_opened() {
    let mut importer = Recording::new(&["a"]);
    importer.fail_open = true;

    assert_eq!(importer.run(), Err(ImportError("cannot open".into())));

    assert_eq!(importer.events, vec!["open"]);
}

#[test]
fn should_provide_sensible_defaults_for_the_optional_steps() {
    struct Minimal {
        input: VecDeque<String>,
        saved: Vec<String>,
    }
    impl DataImporter for Minimal {
        fn open(&mut self) -> Result<(), ImportError> {
            Ok(())
        }
        fn read_record(&mut self) -> Result<Option<String>, ImportError> {
            Ok(self.input.pop_front())
        }
        fn save(&mut self, record: &str) -> Result<(), ImportError> {
            self.saved.push(record.to_string());
            Ok(())
        }
    }
    let mut minimal = Minimal { input: ["x", "   ", "y", ""].iter().map(|s| s.to_string()).collect(), saved: vec![] };

    let r = minimal.run().unwrap();

    assert_eq!(minimal.saved, vec!["x", "y"]);
    assert_eq!(r, report(4, 2, 2, 0));
}

#[test]
fn should_import_csv_end_to_end() {
    let sink = Rc::new(RefCell::new(Vec::new()));
    let lines = ["name,email,city", "ann , a@x.io, Rome", "bob,b@x.io", "", "cy,c@x.io,", "dee,d@x.io,Oslo"];
    let mut importer = CsvImporter::new(lines.iter().map(|s| s.to_string()).collect(), sink.clone());

    let r = importer.run().unwrap();

    assert_eq!(*sink.borrow(), vec!["ann|a@x.io|Rome", "dee|d@x.io|Oslo"]);
    assert_eq!(r, report(5, 2, 3, 0));
    assert_eq!(CsvImporter::new(vec![], Rc::new(RefCell::new(vec![]))).run().unwrap(), report(0, 0, 0, 0));
    assert_eq!(
        CsvImporter::new(vec!["only,a,header".to_string()], Rc::new(RefCell::new(vec![]))).run().unwrap(),
        report(0, 0, 0, 0)
    );
}
