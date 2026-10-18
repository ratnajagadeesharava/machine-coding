use p67_command::contract::*;
use p67_command::{CommandHistory, DeleteCommand, InsertCommand, MacroCommand};
use std::cell::RefCell;
use std::rc::Rc;

/// A command written in the test: upper-cases the whole buffer, remembering the old text.
struct UppercaseCommand {
    before: String,
}

impl Command for UppercaseCommand {
    fn execute(&mut self, buffer: &mut TextBuffer) -> Result<(), EditError> {
        self.before = buffer.text().to_string();
        let len = buffer.len();
        buffer.delete(0, len)?;
        buffer.insert(0, &self.before.to_uppercase())
    }
    fn undo(&mut self, buffer: &mut TextBuffer) {
        let len = buffer.len();
        buffer.delete(0, len).unwrap();
        buffer.insert(0, &self.before).unwrap();
    }
}

fn typed(s: &str) -> TextBuffer {
    let mut b = TextBuffer::new();
    b.insert(0, s).unwrap();
    b
}

#[test]
fn should_insert_and_undo_the_insert() {
    let mut buffer = typed("Hello");
    let mut insert = InsertCommand::new(5, ", world");

    insert.execute(&mut buffer).unwrap();
    assert_eq!(buffer.text(), "Hello, world");
    insert.undo(&mut buffer);

    assert_eq!(buffer.text(), "Hello");
}

#[test]
fn should_remember_what_it_deleted_so_it_can_put_it_back() {
    let mut buffer = typed("Hello, world");
    let mut delete = DeleteCommand::new(5, 12);

    delete.execute(&mut buffer).unwrap();
    assert_eq!(buffer.text(), "Hello");
    delete.undo(&mut buffer);

    assert_eq!(buffer.text(), "Hello, world");
}

#[test]
fn should_run_a_macro_as_one_unit_and_undo_it_in_reverse() {
    let mut buffer = typed("abc");
    let mut macro_cmd = MacroCommand::new(vec![
        Box::new(InsertCommand::new(3, "def")),
        Box::new(DeleteCommand::new(0, 1)),
        Box::new(InsertCommand::new(0, "X")),
    ]);

    macro_cmd.execute(&mut buffer).unwrap();
    assert_eq!(buffer.text(), "Xbcdef");
    macro_cmd.undo(&mut buffer);

    assert_eq!(buffer.text(), "abc");
}

#[test]
fn should_roll_back_a_macro_that_fails_half_way() {
    let mut buffer = typed("abc");
    let mut macro_cmd = MacroCommand::new(vec![
        Box::new(InsertCommand::new(3, "def")),
        Box::new(DeleteCommand::new(0, 99)),
        Box::new(InsertCommand::new(0, "X")),
    ]);

    assert_eq!(macro_cmd.execute(&mut buffer), Err(EditError::OutOfRange));

    assert_eq!(buffer.text(), "abc");
}

#[test]
fn should_undo_and_redo_through_the_history_in_order() {
    let mut buffer = TextBuffer::new();
    let mut history = CommandHistory::new();
    history.execute(Box::new(InsertCommand::new(0, "one")), &mut buffer).unwrap();
    history.execute(Box::new(InsertCommand::new(3, " two")), &mut buffer).unwrap();
    history.execute(Box::new(InsertCommand::new(7, " three")), &mut buffer).unwrap();
    assert_eq!(buffer.text(), "one two three");

    assert!(history.undo(&mut buffer));
    assert!(history.undo(&mut buffer));
    assert_eq!(buffer.text(), "one");
    assert!(history.redo(&mut buffer));
    assert_eq!(buffer.text(), "one two");
    assert!(history.can_undo());
    assert!(history.can_redo());
}

#[test]
fn should_clear_the_redo_list_when_a_new_command_is_executed() {
    let mut buffer = TextBuffer::new();
    let mut history = CommandHistory::new();
    history.execute(Box::new(InsertCommand::new(0, "a")), &mut buffer).unwrap();
    history.execute(Box::new(InsertCommand::new(1, "b")), &mut buffer).unwrap();
    history.undo(&mut buffer);
    assert!(history.can_redo());

    history.execute(Box::new(InsertCommand::new(1, "c")), &mut buffer).unwrap();

    assert!(!history.can_redo());
    assert!(!history.redo(&mut buffer));
    assert_eq!(buffer.text(), "ac");
}

#[test]
fn should_return_false_when_there_is_nothing_to_undo_or_redo_and_not_record_failed_commands() {
    let mut buffer = TextBuffer::new();
    let mut history = CommandHistory::new();
    assert!(!history.undo(&mut buffer));
    assert!(!history.redo(&mut buffer));
    assert!(!history.can_undo());

    let result = history.execute(Box::new(DeleteCommand::new(0, 5)), &mut buffer);

    assert_eq!(result, Err(EditError::OutOfRange));
    assert!(!history.can_undo());
    assert!(buffer.is_empty());
}

#[test]
fn should_handle_a_command_it_has_never_seen() {
    let mut buffer = typed("quiet");
    let mut history = CommandHistory::new();

    history.execute(Box::new(UppercaseCommand { before: String::new() }), &mut buffer).unwrap();
    history.execute(Box::new(InsertCommand::new(5, "!")), &mut buffer).unwrap();
    assert_eq!(buffer.text(), "QUIET!");
    history.undo(&mut buffer);
    history.undo(&mut buffer);

    assert_eq!(buffer.text(), "quiet");
    history.redo(&mut buffer);
    assert_eq!(buffer.text(), "QUIET");
}

#[test]
fn should_log_commands_without_anything_knowing_what_they_do() {
    struct Logged(Rc<RefCell<Vec<&'static str>>>);
    impl Command for Logged {
        fn execute(&mut self, _b: &mut TextBuffer) -> Result<(), EditError> {
            self.0.borrow_mut().push("run");
            Ok(())
        }
        fn undo(&mut self, _b: &mut TextBuffer) {
            self.0.borrow_mut().push("undo");
        }
    }
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut buffer = TextBuffer::new();
    let mut history = CommandHistory::new();

    history.execute(Box::new(Logged(log.clone())), &mut buffer).unwrap();
    history.undo(&mut buffer);
    history.redo(&mut buffer);

    assert_eq!(*log.borrow(), vec!["run", "undo", "run"]);
}
