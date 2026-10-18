use p58_facade::contract::*;
use p58_facade::PhotoUploadFacade;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

type Calls = Rc<RefCell<Vec<String>>>;

struct Scanner {
    calls: Calls,
    clean: Rc<Cell<bool>>,
}
impl VirusScanner for Scanner {
    fn is_clean(&self, content: &str) -> bool {
        self.calls.borrow_mut().push(format!("scan:{content}"));
        self.clean.get()
    }
}

struct Resizer(Calls);
impl ImageResizer for Resizer {
    fn resize(&self, content: &str, max_width: u32) -> String {
        self.0.borrow_mut().push(format!("resize:{max_width}"));
        format!("{content}@{max_width}")
    }
}

struct Storage {
    calls: Calls,
    fails: Rc<Cell<bool>>,
}
impl PhotoStorage for Storage {
    fn store(&self, file_name: &str, content: &str) -> Result<String, String> {
        self.calls.borrow_mut().push(format!("store:{file_name}:{content}"));
        if self.fails.get() {
            Err("disk full".to_string())
        } else {
            Ok(format!("https://cdn.test/{file_name}"))
        }
    }
    fn delete(&self, url: &str) {
        self.calls.borrow_mut().push(format!("delete:{url}"));
    }
}

struct Index {
    calls: Calls,
    fails: Rc<Cell<bool>>,
}
impl SearchIndex for Index {
    fn index(&self, url: &str, file_name: &str) -> Result<(), String> {
        self.calls.borrow_mut().push(format!("index:{url}:{file_name}"));
        if self.fails.get() {
            Err("cluster down".to_string())
        } else {
            Ok(())
        }
    }
}

struct Fixture {
    facade: PhotoUploadFacade,
    calls: Calls,
    clean: Rc<Cell<bool>>,
    storage_fails: Rc<Cell<bool>>,
    index_fails: Rc<Cell<bool>>,
}

fn fixture() -> Fixture {
    let calls: Calls = Rc::new(RefCell::new(Vec::new()));
    let clean = Rc::new(Cell::new(true));
    let storage_fails = Rc::new(Cell::new(false));
    let index_fails = Rc::new(Cell::new(false));
    let facade = PhotoUploadFacade::new(
        Box::new(Scanner { calls: calls.clone(), clean: clean.clone() }),
        Box::new(Resizer(calls.clone())),
        Box::new(Storage { calls: calls.clone(), fails: storage_fails.clone() }),
        Box::new(Index { calls: calls.clone(), fails: index_fails.clone() }),
    );
    Fixture { facade, calls, clean, storage_fails, index_fails }
}

fn result(status: &str, url: &str) -> UploadResult {
    UploadResult { status: status.into(), url: url.into() }
}

#[test]
fn should_run_the_whole_workflow_in_order_behind_one_call() {
    let f = fixture();

    let r = f.facade.upload("cat.jpg", "PIXELS");

    assert_eq!(r, result("OK", "https://cdn.test/cat.jpg"));
    assert_eq!(
        *f.calls.borrow(),
        vec!["scan:PIXELS", "resize:1024", "store:cat.jpg:PIXELS@1024", "index:https://cdn.test/cat.jpg:cat.jpg"]
    );
}

#[test]
fn should_store_the_resized_image_not_the_original() {
    let f = fixture();

    f.facade.upload("dog.PNG", "RAW");

    assert!(f.calls.borrow().contains(&"store:dog.PNG:RAW@1024".to_string()));
}

#[test]
fn should_reject_an_infected_file_after_scanning_only() {
    let f = fixture();
    f.clean.set(false);

    let r = f.facade.upload("bad.jpg", "VIRUS");

    assert_eq!(r, result("REJECTED_VIRUS", ""));
    assert_eq!(*f.calls.borrow(), vec!["scan:VIRUS"]);
}

#[test]
fn should_reject_invalid_input_without_calling_any_subsystem() {
    let f = fixture();

    assert_eq!(f.facade.upload("notes.txt", "x"), result("REJECTED_INVALID", ""));
    assert_eq!(f.facade.upload(" ", "x"), result("REJECTED_INVALID", ""));
    assert_eq!(f.facade.upload("a.jpg", ""), result("REJECTED_INVALID", ""));

    assert!(f.calls.borrow().is_empty());
}

#[test]
fn should_report_a_storage_failure_and_not_index_anything() {
    let f = fixture();
    f.storage_fails.set(true);

    let r = f.facade.upload("a.jpg", "x");

    assert_eq!(r, result("FAILED_STORAGE", ""));
    assert_eq!(*f.calls.borrow(), vec!["scan:x", "resize:1024", "store:a.jpg:x@1024"]);
}

#[test]
fn should_undo_the_storage_when_indexing_fails() {
    let f = fixture();
    f.index_fails.set(true);

    let r = f.facade.upload("a.jpg", "x");

    assert_eq!(r, result("FAILED_INDEXING", ""));
    let calls = f.calls.borrow();
    assert_eq!(calls[calls.len() - 2], "index:https://cdn.test/a.jpg:a.jpg");
    assert_eq!(calls[calls.len() - 1], "delete:https://cdn.test/a.jpg");
}

#[test]
fn should_be_reusable_for_many_uploads() {
    let f = fixture();

    assert_eq!(f.facade.upload("one.jpg", "1").status, "OK");
    f.clean.set(false);
    assert_eq!(f.facade.upload("two.jpg", "2").status, "REJECTED_VIRUS");
    f.clean.set(true);
    assert_eq!(f.facade.upload("three.png", "3").url, "https://cdn.test/three.png");
}
