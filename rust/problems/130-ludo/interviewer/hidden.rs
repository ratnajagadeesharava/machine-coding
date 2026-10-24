#![allow(unused_imports, clippy::needless_range_loop)]
use p130_ludo::{contract::*, create};
use playground_support::sut;

fn dice(s: &str) -> Vec<u8> {
    s.split_whitespace().map(|p| p.parse().unwrap()).collect()
}

macro_rules! game {
    ($players:expr, $dice:expr) => {
        match sut!(create($players, &$dice)) {
            Ok(g) => g,
            Err(e) => panic!("valid set-up rejected: {e:?}"),
        }
    };
}

type G = Box<dyn Ludo>;

/// Plays `rolls` rolls; whenever a move exists the next pick is played. Returns the last move description.
fn replay(g: &mut G, rolls: usize, picks: &[usize]) -> Option<String> {
    let mut last = None;
    let mut next = 0;
    for _ in 0..rolls {
        g.roll().unwrap();
        if !g.legal_moves().is_empty() {
            last = Some(g.move_token(picks[next]).unwrap());
            next += 1;
        }
    }
    last
}

/// Deterministic pseudo-random dice: a 64-bit linear congruential generator, value = ((x >> 33) % 6) + 1.
fn lcg_dice(seed: u64, count: usize) -> Vec<u8> {
    let mut x = seed;
    (0..count)
        .map(|_| {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((x >> 33) % 6) as u8 + 1
        })
        .collect()
}

/// Plays a whole game: "lead" = most advanced legal token, "first" = lowest index, "rear" = least advanced.
fn play_out(g: &mut G, policies: &[&str]) -> Vec<String> {
    let colours = ["RED", "GREEN", "YELLOW", "BLUE"];
    let mut specials = vec![];
    while !g.is_over() {
        let who = g.current_player();
        let seat = colours.iter().position(|c| *c == who).unwrap();
        g.roll().unwrap();
        let legal = g.legal_moves();
        if legal.is_empty() {
            continue;
        }
        let mut pick = legal[0];
        for &t in &legal {
            let (a, b) = (
                g.token_progress(&who, t).unwrap(),
                g.token_progress(&who, pick).unwrap(),
            );
            if (policies[seat] == "lead" && a > b) || (policies[seat] == "rear" && a < b) {
                pick = t;
            }
        }
        let d = g.move_token(pick).unwrap();
        if d != "moved" && d != "home" {
            specials.push(d);
        }
    }
    specials
}

fn assert_progress(g: &G, colour: &str, expected: [u32; 4]) {
    for t in 0..4 {
        assert_eq!(
            g.token_progress(colour, t),
            Ok(expected[t]),
            "{colour} token {t}"
        );
    }
}

fn count_captures(specials: &[String]) -> usize {
    specials.iter().map(|s| s.split(',').count()).sum()
}

#[test]
fn should_capture_two_colours_on_the_same_cell_in_turn_order() {
    let mut g = game!(
        3,
        dice("6 4 6 1 4 6 6 4 1 4 6 6 6 2 2 6 1 2 6 3 6 1 6 2 1 1 4 4 5 3 6")
    );

    let last = replay(
        &mut g,
        31,
        &[
            3, 3, 2, 2, 3, 2, 3, 3, 3, 1, 3, 2, 2, 3, 0, 0, 3, 1, 3, 3, 0, 3, 2, 0, 2, 2, 0,
        ],
    );

    assert_eq!(last.as_deref(), Some("captured:RED:1,captured:YELLOW:1"));
    assert_progress(&g, "GREEN", [15, 0, 0, 0]);
    assert_progress(&g, "RED", [0, 0, 8, 0]);
    assert_progress(&g, "YELLOW", [0, 0, 7, 8]);
    assert_eq!(g.token_cell("GREEN", 0), Ok(27));
    assert_eq!(g.current_player(), "GREEN");
}

#[test]
fn should_keep_the_turn_when_a_six_cannot_move_any_token() {
    let mut g = game!(2, dice("6 5 1 3 1 3 1 6 6 6 1 6 6 5 1 6 6 3 1 6 6 4 1 6 5 1 2 1 5 1 6 6 4 1 6 6 6 1 6 5 1 6 5 1 6 3 1 4 1 6 2 1 6 6 3 1 3 1 6 2 1 3 1 3 1 6 6 3 1 6 5 1 6 6 5 1 6 1"));
    for _ in 0..76 {
        g.roll().unwrap();
        let legal = g.legal_moves();
        if !legal.is_empty() {
            let who = g.current_player();
            let mut pick = legal[0];
            for &t in &legal {
                if g.token_progress(&who, t).unwrap() > g.token_progress(&who, pick).unwrap() {
                    pick = t;
                }
            }
            g.move_token(pick).unwrap();
        }
    }
    assert_progress(&g, "RED", [56, 57, 56, 52]);
    assert_eq!(g.current_player(), "RED");

    assert_eq!(g.roll(), Ok(6));
    assert!(g.legal_moves().is_empty());
    assert_eq!(g.current_player(), "RED");

    assert_eq!(g.roll(), Ok(1));
    assert_eq!(g.legal_moves(), [0, 2, 3]);
}

#[test]
fn should_count_a_six_that_captured_towards_the_three_six_limit() {
    let mut g = game!(2, dice("6 3 6 2 6 6 6 2"));
    let last = replay(&mut g, 6, &[1, 1, 3, 3, 1, 1]);
    assert_eq!(last.as_deref(), Some("captured:GREEN:1"));
    assert_eq!(g.current_player(), "RED");

    assert_eq!(g.roll(), Ok(6));

    assert!(g.legal_moves().is_empty());
    assert_progress(&g, "RED", [0, 16, 0, 0]);
    assert_eq!(g.current_player(), "GREEN");
    g.roll().unwrap();
    assert_eq!(g.current_player(), "RED");
}

#[test]
fn should_capture_after_a_blue_token_wraps_past_cell_51() {
    let mut g = game!(4, dice("4 3 4 6 4 6 1 3 5 6 4"));

    let last = replay(&mut g, 11, &[3, 3, 2, 2, 3, 3]);

    assert_eq!(last.as_deref(), Some("captured:RED:1"));
    assert_progress(&g, "BLUE", [0, 0, 0, 15]);
    assert_eq!(g.token_cell("BLUE", 3), Ok(1));
    assert_progress(&g, "RED", [0, 0, 0, 0]);
    assert_eq!(g.current_player(), "BLUE");
}

#[test]
fn should_reject_every_call_once_the_game_is_won_even_with_dice_left() {
    let mut d = dice("3 1 6 4 1 6 4 1 6 6 3 1 4 1 5 1 6 6 2 1 6 6 5 1 6 5 1 2 1 6 6 5 1 6 6 3 1 6 4 1 6 3 1 2 2 6 2 1 6 4 1 6 6 4 1 3 1 4 1 6 6 3 1 6 2 6 6 6 1 5 1 6 6 6 1 6 6");
    d.extend([6, 6, 6]);
    let mut g = game!(2, d);

    play_out(&mut g, &["lead", "first"]);

    assert!(g.is_over());
    assert_eq!(g.winner().as_deref(), Some("RED"));
    assert_eq!(g.roll(), Err(LudoError::GameOver));
    assert_eq!(g.move_token(0), Err(LudoError::GameOver));
    assert_progress(&g, "RED", [57, 57, 57, 57]);
}

#[test]
fn should_play_random_game_1() {
    let mut g = game!(2, lcg_dice(1, 190));

    let specials = play_out(&mut g, &["lead", "first"]);

    assert!(g.is_over());
    assert_eq!(g.winner().as_deref(), Some("RED"));
    assert_eq!(count_captures(&specials), 2);
    assert_progress(&g, "RED", [57, 57, 57, 57]);
    assert_progress(&g, "GREEN", [57, 57, 57, 48]);
}

#[test]
fn should_play_random_game_2() {
    let mut g = game!(3, lcg_dice(2, 271));

    let specials = play_out(&mut g, &["lead", "rear", "first"]);

    assert!(g.is_over());
    assert_eq!(g.winner().as_deref(), Some("RED"));
    assert_eq!(count_captures(&specials), 9);
    assert_progress(&g, "RED", [57, 57, 57, 57]);
    assert_progress(&g, "GREEN", [0, 41, 0, 37]);
    assert_progress(&g, "YELLOW", [57, 57, 49, 0]);
}

#[test]
fn should_play_random_game_3() {
    let mut g = game!(4, lcg_dice(3, 380));

    let specials = play_out(&mut g, &["rear", "lead", "first", "lead"]);

    assert!(g.is_over());
    assert_eq!(g.winner().as_deref(), Some("GREEN"));
    assert_eq!(count_captures(&specials), 14);
    assert_progress(&g, "RED", [53, 0, 0, 56]);
    assert_progress(&g, "GREEN", [57, 57, 57, 57]);
    assert_progress(&g, "YELLOW", [57, 57, 0, 0]);
    assert_progress(&g, "BLUE", [57, 57, 57, 0]);
}

#[test]
fn should_play_random_game_4() {
    let mut g = game!(4, lcg_dice(12345, 356));

    let specials = play_out(&mut g, &["first", "first", "first", "first"]);

    assert!(g.is_over());
    assert_eq!(g.winner().as_deref(), Some("RED"));
    assert_eq!(count_captures(&specials), 8);
    assert_progress(&g, "RED", [57, 57, 57, 57]);
    assert_progress(&g, "GREEN", [57, 57, 57, 0]);
    assert_progress(&g, "YELLOW", [57, 57, 14, 0]);
    assert_progress(&g, "BLUE", [57, 57, 51, 0]);
}

#[test]
fn should_play_random_game_5() {
    let mut g = game!(2, lcg_dice(777, 172));

    let specials = play_out(&mut g, &["rear", "rear"]);

    assert!(g.is_over());
    assert_eq!(g.winner().as_deref(), Some("GREEN"));
    assert_eq!(count_captures(&specials), 2);
    assert_progress(&g, "RED", [54, 55, 54, 55]);
    assert_progress(&g, "GREEN", [57, 57, 57, 57]);
}
