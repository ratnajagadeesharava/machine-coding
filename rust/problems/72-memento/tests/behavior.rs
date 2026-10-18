use p72_memento::contract::GameError;
use p72_memento::{GameCharacter, SaveSlots, UndoHistory, MAX_ITEMS};

const SOURCE: &str = include_str!("../src/lib.rs");

fn hero() -> GameCharacter {
    GameCharacter::new("Aria").unwrap()
}

fn assert_state(c: &GameCharacter, hp: u32, level: u32, pos: (i32, i32), items: &[&str]) {
    assert_eq!(c.hp(), hp, "hp");
    assert_eq!(c.level(), level, "level");
    assert_eq!(c.position(), pos, "position");
    let items: Vec<String> = items.iter().map(|s| s.to_string()).collect();
    assert_eq!(c.inventory(), items.as_slice(), "inventory");
}

#[test]
fn should_start_fresh_and_change_through_its_own_methods() {
    let mut c = hero();

    assert_state(&c, 100, 1, (0, 0), &[]);
    c.take_damage(30);
    c.heal(5);
    c.level_up();
    c.move_to(3, -4);
    c.pick_up("sword").unwrap();
    c.pick_up("potion").unwrap();
    assert!(!c.drop_item("shield"));
    assert!(c.drop_item("potion"));

    assert_state(&c, 75, 2, (3, -4), &["sword"]);
    assert_eq!(c.name(), "Aria");
    c.heal(1000);
    c.take_damage(1000);
    assert_eq!(c.hp(), 0);
}

#[test]
fn should_restore_the_exact_saved_state() {
    let mut c = hero();
    c.take_damage(40);
    c.level_up();
    c.move_to(7, 8);
    c.pick_up("sword").unwrap();
    c.pick_up("map").unwrap();
    let saved = c.save();

    c.take_damage(50);
    c.level_up();
    c.move_to(-1, -1);
    c.drop_item("sword");
    c.pick_up("gem").unwrap();
    c.restore(&saved);

    assert_state(&c, 60, 2, (7, 8), &["sword", "map"]);
}

#[test]
fn should_keep_a_snapshot_independent_of_later_changes_and_reusable() {
    let mut c = hero();
    c.pick_up("sword").unwrap();
    let saved = c.save();

    c.pick_up("shield").unwrap();
    c.take_damage(10);
    c.restore(&saved);
    assert_state(&c, 100, 1, (0, 0), &["sword"]);
    c.pick_up("bow").unwrap();
    c.restore(&saved);

    assert_state(&c, 100, 1, (0, 0), &["sword"]);
}

#[test]
fn should_allow_restoring_into_another_character_without_changing_its_name() {
    let mut aria = hero();
    aria.level_up();
    aria.pick_up("sword").unwrap();
    let mut twin = GameCharacter::new("Bryn").unwrap();

    twin.restore(&aria.save());

    assert_state(&twin, 100, 2, (0, 0), &["sword"]);
    assert_eq!(twin.name(), "Bryn");
    twin.pick_up("bow").unwrap();
    assert_eq!(aria.inventory(), ["sword".to_string()].as_slice());
}

#[test]
fn should_store_named_save_slots() {
    let mut c = hero();
    let mut slots = SaveSlots::new();
    slots.save("before-boss", &c).unwrap();
    c.take_damage(90);
    c.move_to(5, 5);
    slots.save("after-boss", &c).unwrap();
    c.heal(100);

    assert_eq!(slots.slot_count(), 2);
    assert!(slots.load("after-boss", &mut c));
    assert_state(&c, 10, 1, (5, 5), &[]);
    assert!(slots.load("before-boss", &mut c));
    assert_state(&c, 100, 1, (0, 0), &[]);
    assert!(!slots.load("nope", &mut c));
    assert_state(&c, 100, 1, (0, 0), &[]);

    c.level_up();
    slots.save("before-boss", &c).unwrap();
    assert_eq!(slots.slot_count(), 2);
    c.take_damage(1);
    slots.load("before-boss", &mut c);
    assert_state(&c, 100, 2, (0, 0), &[]);
}

#[test]
fn should_undo_and_redo_across_checkpoints() {
    let mut c = hero();
    let mut history = UndoHistory::new(10).unwrap();
    history.checkpoint(&c);
    c.take_damage(30);
    history.checkpoint(&c);
    c.level_up();
    c.take_damage(20);
    assert_state(&c, 50, 2, (0, 0), &[]);

    assert!(history.undo(&mut c));
    assert_state(&c, 70, 1, (0, 0), &[]);
    assert!(history.undo(&mut c));
    assert_state(&c, 100, 1, (0, 0), &[]);
    assert!(!history.undo(&mut c));
    assert!(history.redo(&mut c));
    assert_state(&c, 70, 1, (0, 0), &[]);
    assert!(history.redo(&mut c));
    assert_state(&c, 50, 2, (0, 0), &[]);
    assert!(!history.redo(&mut c));
    assert_eq!(history.undo_depth(), 2);
    assert_eq!(history.redo_depth(), 0);
}

#[test]
fn should_forget_the_redo_future_when_a_new_checkpoint_is_taken() {
    let mut c = hero();
    let mut history = UndoHistory::new(10).unwrap();
    history.checkpoint(&c);
    c.take_damage(10);
    history.undo(&mut c);
    assert_eq!(history.redo_depth(), 1);

    history.checkpoint(&c);

    assert_eq!(history.redo_depth(), 0);
    assert!(!history.redo(&mut c));
    assert_eq!(history.undo_depth(), 1);
}

#[test]
fn should_drop_the_oldest_checkpoint_when_the_limit_is_exceeded() {
    let mut c = hero();
    let mut history = UndoHistory::new(2).unwrap();
    for _ in 0..3 {
        history.checkpoint(&c);
        c.take_damage(10);
    }
    assert_state(&c, 70, 1, (0, 0), &[]);

    assert_eq!(history.undo_depth(), 2);
    assert!(history.undo(&mut c));
    assert_state(&c, 80, 1, (0, 0), &[]);
    assert!(history.undo(&mut c));
    assert_state(&c, 90, 1, (0, 0), &[]);
    assert!(!history.undo(&mut c));
}

#[test]
fn should_keep_the_memento_opaque() {
    assert_eq!(hero().hp(), 100);

    let lines: Vec<&str> = SOURCE.lines().map(str::trim).collect();
    let public_fields = lines.iter().filter(|l| l.starts_with("pub ") && l.ends_with(',') && !l.contains('(')).count();
    assert_eq!(public_fields, 0, "no struct in this file may have a pub field");
    let start = lines.iter().position(|l| l.starts_with("pub struct Snapshot")).expect("Snapshot must exist");
    let after: Vec<&&str> = lines[start..].iter().take_while(|l| !l.starts_with("pub struct GameCharacter")).collect();
    assert!(!after.iter().any(|l| l.starts_with("impl Snapshot")), "Snapshot must have no methods of its own");
}

#[test]
fn should_reject_invalid_input() {
    let mut c = hero();
    for item in ["a", "b", "c"] {
        c.pick_up(item).unwrap();
    }
    let mut slots = SaveSlots::new();

    assert!(matches!(GameCharacter::new(" "), Err(GameError::InvalidInput)));
    assert_eq!(c.pick_up("d"), Err(GameError::InventoryFull));
    assert_eq!(c.pick_up(" "), Err(GameError::InvalidInput));
    assert_eq!(slots.save(" ", &c), Err(GameError::InvalidInput));
    assert!(matches!(UndoHistory::new(0), Err(GameError::InvalidInput)));
    assert_eq!(c.inventory().len(), MAX_ITEMS);
}
