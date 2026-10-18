#![allow(unused_imports, clippy::needless_range_loop)]
use p09_snake_and_ladder::{contract::*, create};
use playground_support::sut;

const SNAKES: [(u32, u32); 2] = [(27, 5), (21, 9)];
const LADDERS: [(u32, u32); 2] = [(3, 22), (8, 26)];

fn game(players: &[&str]) -> Option<Box<dyn SnakeAndLadderApi>> {
    create(30, &SNAKES, &LADDERS, players)
}

#[test]
fn should_start_everyone_off_the_board_with_first_player_to_move() {
    let g = sut!(game(&["a", "b"]));

    assert_eq!(g.position_of("a"), 0);
    assert_eq!(g.position_of("b"), 0);
    assert_eq!(g.current_player().as_deref(), Some("a"));
    assert_eq!(g.winner(), None);
    assert_eq!(g.position_of("nobody"), -1);
}

#[test]
fn should_move_forward_and_pass_the_turn() {
    let mut g = sut!(game(&["a", "b"]));

    assert_eq!(g.roll(4), 4);

    assert_eq!(g.position_of("a"), 4);
    assert_eq!(g.current_player().as_deref(), Some("b"));
}

#[test]
fn should_rotate_turns_through_all_players() {
    let mut g = sut!(game(&["a", "b", "c"]));

    g.roll(1);
    assert_eq!(g.current_player().as_deref(), Some("b"));
    g.roll(2);
    assert_eq!(g.current_player().as_deref(), Some("c"));
    g.roll(4);
    assert_eq!(g.current_player().as_deref(), Some("a"));
    assert_eq!(g.position_of("b"), 2);
}

#[test]
fn should_climb_ladders() {
    let mut g = sut!(game(&["a", "b"]));

    assert_eq!(g.roll(3), 22);

    assert_eq!(g.position_of("a"), 22);
}

#[test]
fn should_slide_down_snakes() {
    let mut g = sut!(game(&["a", "b"]));
    g.roll(3); // a: 22
    g.roll(1); // b: 1

    assert_eq!(g.roll(5), 5); // a lands on 27 -> 5

    assert_eq!(g.position_of("a"), 5);
}

#[test]
fn should_not_chain_jumps() {
    let mut g = sut!(game(&["a"]));

    assert_eq!(g.roll(3), 22); // ladder 3 -> 22
    assert_eq!(g.roll(5), 5); // 22 + 5 = 27, a snake head -> 5
    assert_eq!(g.roll(3), 26); // 5 + 3 = 8, a ladder bottom -> 26
}

#[test]
fn should_stay_put_when_roll_would_overshoot_and_still_pass_the_turn() {
    let mut g = sut!(game(&["a", "b"]));
    // bring a to 29 using b as a bystander
    for (a, b) in [(6, 1), (6, 1)] {
        g.roll(a); // a 6, then 12
        g.roll(b); // b 1, then 2
    }
    g.roll(6); // a 18
    g.roll(1); // b 3 -> ladder -> 22
    g.roll(6); // a 24
    g.roll(1); // b 23
    g.roll(5); // a 29
    g.roll(1); // b 24
    assert_eq!(g.position_of("a"), 29);

    assert_eq!(g.roll(2), 29); // a would reach 31: stays
    assert_eq!(g.current_player().as_deref(), Some("b"));
}

#[test]
fn should_win_on_exact_roll_and_refuse_further_moves() {
    let mut g = sut!(game(&["solo"]));
    for d in [6, 6, 6, 6, 5] {
        g.roll(d);
    }
    assert_eq!(g.position_of("solo"), 29);

    assert_eq!(g.roll(1), 30);

    assert_eq!(g.winner().as_deref(), Some("solo"));
    assert_eq!(g.current_player(), None);
    assert_eq!(g.roll(1), -1);
    assert_eq!(g.position_of("solo"), 30);
}

#[test]
fn should_refuse_invalid_dice_without_consuming_the_turn() {
    let mut g = sut!(game(&["a", "b"]));

    assert_eq!(g.roll(0), -1);
    assert_eq!(g.roll(7), -1);
    assert_eq!(g.roll(-2), -1);

    assert_eq!(g.current_player().as_deref(), Some("a"));
    assert_eq!(g.position_of("a"), 0);
}

#[test]
fn should_declare_winner_who_reaches_the_end_first() {
    let mut g = sut!(game(&["a", "b"]));
    g.roll(4); // a 4
    g.roll(1); // b 1
    g.roll(4); // a 8 -> 26
    g.roll(1); // b 2
    assert_eq!(g.position_of("a"), 26);

    assert_eq!(g.roll(4), 30);

    assert_eq!(g.winner().as_deref(), Some("a"));
}
