#![allow(unused_imports, clippy::needless_range_loop)]
use p85_text_editor_undo::{contract::*, create};
use playground_support::sut;

fn state(e: &dyn Editor, text: &str, cursor: usize) {
    assert_eq!(e.text(), text);
    assert_eq!(e.cursor(), cursor);
}

#[test]
fn should_insert_at_the_cursor() {
    let mut e = sut!(create());

    e.type_text("hello");
    e.move_cursor(2).unwrap();
    e.type_text("XY");

    state(&*e, "heXYllo", 4);
}

#[test]
fn should_backspace_before_the_cursor_and_stop_at_the_start() {
    let mut e = sut!(create());
    e.type_text("abc");

    e.backspace();
    state(&*e, "ab", 2);
    e.move_cursor(0).unwrap();
    e.backspace();
    state(&*e, "ab", 0);
}

#[test]
fn should_replace_the_selection_when_typing() {
    let mut e = sut!(create());
    e.type_text("hello world");

    e.select(0, 5).unwrap();
    assert_eq!(e.selected_text(), "hello");
    e.type_text("bye");

    state(&*e, "bye world", 3);
    assert_eq!(e.selected_text(), "");
}

#[test]
fn should_delete_the_selection_on_backspace() {
    let mut e = sut!(create());
    e.type_text("hello world");

    e.select(5, 11).unwrap();
    e.backspace();

    state(&*e, "hello", 5);
}

#[test]
fn should_reject_out_of_range_positions() {
    let mut e = sut!(create());
    e.type_text("abc");

    assert_eq!(e.move_cursor(4), Err(EditorError::InvalidArgument));
    assert_eq!(e.select(2, 1), Err(EditorError::InvalidArgument));
    assert_eq!(e.select(0, 4), Err(EditorError::InvalidArgument));
    assert_eq!(e.replace_all("", "x"), Err(EditorError::InvalidArgument));
}

#[test]
fn should_copy_cut_and_paste() {
    let mut e = sut!(create());
    e.type_text("abcdef");

    e.select(1, 3).unwrap();
    e.copy();
    e.move_cursor(6).unwrap();
    e.paste();
    state(&*e, "abcdefbc", 8);

    e.select(0, 2).unwrap();
    e.cut();
    state(&*e, "cdefbc", 0);
    e.move_cursor(6).unwrap();
    e.paste();
    state(&*e, "cdefbcab", 8);
}

#[test]
fn should_group_consecutive_typing_into_one_undo_step() {
    let mut e = sut!(create());

    e.type_text("a");
    e.type_text("b");
    e.type_text("c");
    assert!(e.undo());

    state(&*e, "", 0);
    assert!(!e.undo());
}

#[test]
fn should_end_the_typing_group_when_the_cursor_moves() {
    let mut e = sut!(create());
    e.type_text("ab");
    e.move_cursor(0).unwrap();
    e.type_text("X");
    e.type_text("Y");

    e.undo();
    state(&*e, "ab", 0);
    e.undo();
    state(&*e, "", 0);
}

#[test]
fn should_group_consecutive_backspaces_but_not_typing_with_backspace() {
    let mut e = sut!(create());
    e.type_text("hello");
    e.backspace();
    e.backspace();
    e.type_text("p");

    e.undo(); // the "p"
    state(&*e, "hel", 3);
    e.undo(); // both backspaces together
    state(&*e, "hello", 5);
    e.undo(); // the typing
    state(&*e, "", 0);
}

#[test]
fn should_not_group_typing_over_a_selection() {
    let mut e = sut!(create());
    e.type_text("hello");
    e.select(0, 5).unwrap();
    e.type_text("a");
    e.select(0, 1).unwrap();
    e.type_text("b");

    e.undo();
    state(&*e, "a", 1);
    e.undo();
    state(&*e, "hello", 5);
}

#[test]
fn should_restore_the_cursor_on_undo_and_redo() {
    let mut e = sut!(create());
    e.type_text("hello");
    e.move_cursor(1).unwrap();
    e.type_text("XX");
    state(&*e, "hXXello", 3);

    e.undo();
    state(&*e, "hello", 1);
    e.redo();
    state(&*e, "hXXello", 3);
    assert!(!e.redo());
}

#[test]
fn should_clear_the_redo_list_on_a_new_edit() {
    let mut e = sut!(create());
    e.type_text("a");
    e.move_cursor(1).unwrap();
    e.type_text("b");
    e.undo();

    e.type_text("c");

    assert!(!e.redo());
    state(&*e, "ac", 2);
}

#[test]
fn should_undo_cut_and_paste_as_single_steps() {
    let mut e = sut!(create());
    e.type_text("abcdef");
    e.select(2, 4).unwrap();
    e.cut();
    e.paste();
    state(&*e, "abcdef", 4);

    e.undo();
    state(&*e, "abef", 2);
    e.undo();
    state(&*e, "abcdef", 4);
}

#[test]
fn should_replace_all_non_overlapping_occurrences_as_one_step() {
    let mut e = sut!(create());
    e.type_text("aaaa b aa");

    let count = e.replace_all("aa", "X").unwrap();

    assert_eq!(count, 3);
    assert_eq!(e.text(), "XX b X");
    e.undo();
    assert_eq!(e.text(), "aaaa b aa");
    e.redo();
    assert_eq!(e.text(), "XX b X");
    assert_eq!(e.replace_all("zzz", "y"), Ok(0));
    e.undo(); // the zero-count replace made no step: this undoes the previous replace
    assert_eq!(e.text(), "aaaa b aa");
}

#[test]
fn should_ignore_empty_typing_and_selection_only_operations_for_undo() {
    let mut e = sut!(create());
    e.type_text("abc");

    e.type_text("");
    e.select(0, 2).unwrap();
    e.copy();
    e.move_cursor(1).unwrap();

    e.undo();
    state(&*e, "", 0);
    assert!(!e.undo());
}

#[test]
fn should_survive_long_sessions_of_undo_and_redo() {
    let mut e = sut!(create());
    for _ in 0..200 {
        e.type_text("x");
        let len = e.text().len();
        e.move_cursor(len).unwrap(); // each move_cursor ends the group: 200 separate steps
    }

    for _ in 0..200 {
        assert!(e.undo());
    }
    assert_eq!(e.text(), "");
    for _ in 0..200 {
        assert!(e.redo());
    }
    assert_eq!(e.text().len(), 200);
}
