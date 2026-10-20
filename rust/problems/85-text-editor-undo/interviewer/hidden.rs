#![allow(unused_imports, clippy::needless_range_loop)]
use p85_text_editor_undo::{contract::*, create};
use playground_support::sut;

#[test]
fn should_match_an_undo_redo_model_over_a_thousand_random_operations() {
    let mut e = sut!(create());
    let mut s = 42u64;
    let mut next = move |n: usize| {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s % n as u64) as usize
    };
    let mut texts = Vec::new();
    for _ in 0..1_000 {
        let before = e.text();
        let len = e.text().len();
        e.move_cursor(next(len + 1)).unwrap(); // ends any group so every edit below is a new step
        if next(4) == 0 && len > 0 {
            let from = next(len);
            e.select(from, from + 1).unwrap();
            e.backspace();
        } else {
            e.type_text(&((b'a' + next(26) as u8) as char).to_string());
        }
        texts.push(before);
    }
    while let Some(before) = texts.pop() {
        assert!(e.undo());
        assert_eq!(e.text(), before);
    }
    assert!(!e.undo());
}

#[test]
fn should_handle_twenty_thousand_characters_of_typing() {
    let mut e = sut!(create());

    for _ in 0..20_000 {
        e.type_text("x");
    }

    assert_eq!(e.text().len(), 20_000);
    assert!(e.undo());
    assert_eq!(e.text(), "");
}
