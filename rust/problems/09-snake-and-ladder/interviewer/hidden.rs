#![allow(unused_imports, clippy::needless_range_loop)]
use p09_snake_and_ladder::{contract::*, create};
use playground_support::sut;

#[test]
fn ladder_to_the_last_square_should_win() {
    let mut g = sut!(create(30, &[], &[(5, 30)], &["a", "b"]));

    assert_eq!(g.roll(5), 30);
    assert_eq!(g.winner().as_deref(), Some("a"));
    assert_eq!(g.roll(1), -1);
}

#[test]
fn jump_destination_should_not_trigger_another_jump() {
    // the snake at 20 ends on 12, which is the bottom of a ladder: sliding down must not climb again
    let mut g = sut!(create(30, &[(20, 12)], &[(12, 28)], &["a"]));

    for _ in 0..4 {
        g.roll(5); // 5, 10, 15, 20 -> snake -> 12
    }
    assert_eq!(g.position_of("a"), 12);
}

#[test]
fn should_advance_turn_even_when_overshooting() {
    let mut g = sut!(create(10, &[], &[], &["a", "b"]));
    g.roll(6); // a 6
    g.roll(6); // b 6
    assert_eq!(g.roll(6), 6); // a would reach 12: stays
    assert_eq!(g.current_player().as_deref(), Some("b"));
    assert_eq!(g.roll(4), 10);
    assert_eq!(g.winner().as_deref(), Some("b"));
}

#[test]
fn invalid_dice_after_game_over_should_still_be_refused() {
    let mut g = sut!(create(10, &[], &[(6, 10)], &["a"]));
    g.roll(6);
    assert_eq!(g.winner().as_deref(), Some("a"));
    assert_eq!(g.roll(0), -1);
    assert_eq!(g.roll(3), -1);
}
