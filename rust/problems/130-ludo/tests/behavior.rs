#![allow(unused_imports, clippy::needless_range_loop)]
use p130_ludo::{contract::*, create};
use playground_support::sut;

/// Parses "6 4 1" into dice.
fn dice(s: &str) -> Vec<u8> {
    s.split_whitespace().map(|p| p.parse().unwrap()).collect()
}

macro_rules! game {
    ($players:expr, $dice:expr) => {
        match sut!(create($players, &dice($dice))) {
            Ok(g) => g,
            Err(e) => panic!("valid set-up rejected: {e:?}"),
        }
    };
}

type G = Box<dyn Ludo>;

/// Rolls once and, if a move exists, plays the given token. Returns the move description if one was played.
fn turn(g: &mut G, token: usize) -> Option<String> {
    g.roll().unwrap();
    if g.legal_moves().is_empty() {
        None
    } else {
        Some(g.move_token(token).unwrap())
    }
}

/// Plays a whole game: "lead" = most advanced legal token, "first" = lowest index, "rear" = least advanced.
/// Returns every move description that is neither "moved" nor "home".
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

// ---- set-up and turn order -------------------------------------------------------------------------------

#[test]
fn should_start_with_all_tokens_in_the_yard_and_red_to_move() {
    let g = game!(4, "1");

    assert_eq!(g.current_player(), "RED");
    assert!(!g.is_over());
    assert_eq!(g.winner(), None);
    assert!(g.legal_moves().is_empty());
    for c in ["RED", "GREEN", "YELLOW", "BLUE"] {
        for t in 0..4 {
            assert_eq!(g.token_progress(c, t), Ok(0));
            assert_eq!(g.token_cell(c, t), Ok(-1));
        }
    }
}

#[test]
fn should_reject_invalid_setup() {
    let _valid = game!(2, "1"); // a valid instance first: the factory must be wired
    assert!(matches!(
        create(1, &[1]),
        Some(Err(LudoError::InvalidArgument))
    ));
    assert!(matches!(
        create(5, &[1]),
        Some(Err(LudoError::InvalidArgument))
    ));
    assert!(matches!(
        create(2, &[3, 0, 2]),
        Some(Err(LudoError::InvalidArgument))
    ));
    assert!(matches!(
        create(2, &[3, 7, 2]),
        Some(Err(LudoError::InvalidArgument))
    ));
}

#[test]
fn should_use_the_first_n_colours_in_turn_order() {
    let mut three = game!(3, "1 1 1 1");
    let mut order = vec![];
    for _ in 0..4 {
        order.push(three.current_player());
        three.roll().unwrap();
    }
    assert_eq!(order, ["RED", "GREEN", "YELLOW", "RED"]);
    assert_eq!(
        three.token_progress("BLUE", 0),
        Err(LudoError::InvalidArgument)
    );

    let mut four = game!(4, "1 1 1 1 1");
    let mut order = vec![];
    for _ in 0..5 {
        order.push(four.current_player());
        four.roll().unwrap();
    }
    assert_eq!(order, ["RED", "GREEN", "YELLOW", "BLUE", "RED"]);
}

#[test]
fn should_pass_the_turn_when_no_token_can_move() {
    let mut g = game!(2, "3 4");

    assert_eq!(g.roll(), Ok(3));
    assert!(g.legal_moves().is_empty());
    assert_eq!(g.current_player(), "GREEN");
    assert_eq!(g.roll(), Ok(4));
    assert_eq!(g.current_player(), "RED");
}

// ---- leaving the yard and moving -------------------------------------------------------------------------

#[test]
fn should_leave_the_yard_only_on_a_six_onto_the_start_cell() {
    let mut g = game!(2, "3 6 6");

    g.roll().unwrap(); // RED: 3, nothing moves
    assert_eq!(g.roll(), Ok(6)); // GREEN
    assert_eq!(g.legal_moves(), [0, 1, 2, 3]);
    assert_eq!(g.move_token(2).unwrap(), "moved");
    assert_eq!(g.token_progress("GREEN", 2), Ok(1));
    assert_eq!(g.token_cell("GREEN", 2), Ok(13));
    assert_eq!(g.token_progress("GREEN", 0), Ok(0));

    g.roll().unwrap(); // GREEN again: 6
    g.move_token(0).unwrap();
    assert_eq!(g.token_cell("GREEN", 0), Ok(13));
}

#[test]
fn should_move_the_chosen_token_by_the_rolled_value() {
    let mut g = game!(2, "6 6 3");

    turn(&mut g, 1);
    turn(&mut g, 0);
    assert_eq!(g.roll(), Ok(3));
    assert_eq!(g.move_token(1).unwrap(), "moved");

    assert_eq!(g.token_progress("RED", 1), Ok(4));
    assert_eq!(g.token_progress("RED", 0), Ok(1));
    assert_eq!(g.token_cell("RED", 1), Ok(3));
    assert_eq!(g.current_player(), "GREEN");
}

#[test]
fn should_list_legal_tokens_in_ascending_order() {
    let mut g = game!(2, "6 6 3 6");

    turn(&mut g, 2);
    turn(&mut g, 3);
    g.roll().unwrap(); // 3: only the two tokens on the track can move
    assert_eq!(g.legal_moves(), [2, 3]);
    g.move_token(3).unwrap();
    g.roll().unwrap(); // GREEN 6
    assert_eq!(g.legal_moves(), [0, 1, 2, 3]);
}

#[test]
fn should_give_an_extra_turn_after_a_six() {
    let mut g = game!(2, "6 2");

    g.roll().unwrap();
    g.move_token(0).unwrap();
    assert_eq!(g.current_player(), "RED");
    g.roll().unwrap();
    g.move_token(0).unwrap();
    assert_eq!(g.current_player(), "GREEN");
}

#[test]
fn should_forfeit_the_third_consecutive_six() {
    let mut g = game!(2, "6 6 6 2");

    turn(&mut g, 0);
    turn(&mut g, 0);
    assert_eq!(g.roll(), Ok(6));

    assert!(g.legal_moves().is_empty());
    assert_eq!(g.token_progress("RED", 0), Ok(7));
    assert_eq!(g.current_player(), "GREEN");
    g.roll().unwrap();
    assert_eq!(g.current_player(), "RED");
}

#[test]
fn should_restart_the_count_of_sixes_when_the_turn_passes() {
    let mut g = game!(2, "6 6 5 1 6 6 4");

    turn(&mut g, 0);
    turn(&mut g, 0);
    turn(&mut g, 0);
    assert_eq!(g.current_player(), "GREEN");
    g.roll().unwrap();
    turn(&mut g, 0);
    turn(&mut g, 0);
    assert_eq!(g.roll(), Ok(4));
    assert_eq!(g.legal_moves(), [0]);
    g.move_token(0).unwrap();

    assert_eq!(g.token_progress("RED", 0), Ok(28));
    assert_eq!(g.current_player(), "GREEN");
}

// ---- capturing -------------------------------------------------------------------------------------------

#[test]
fn should_capture_a_single_opponent_on_an_unsafe_cell() {
    let mut g = game!(2, "6 6 5 6 2 4 3");

    for _ in 0..5 {
        turn(&mut g, 0);
    }
    assert_eq!(g.token_cell("GREEN", 0), Ok(15));
    assert_eq!(g.current_player(), "RED");

    g.roll().unwrap(); // 4
    assert_eq!(g.move_token(0).unwrap(), "captured:GREEN:1");

    assert_eq!(g.token_progress("GREEN", 0), Ok(0));
    assert_eq!(g.token_progress("RED", 0), Ok(16));
    assert_eq!(g.token_cell("RED", 0), Ok(15));
}

#[test]
fn should_give_an_extra_turn_after_a_capture() {
    let mut g = game!(2, "6 6 5 6 2 4 3");
    for _ in 0..6 {
        turn(&mut g, 0);
    }

    assert_eq!(g.current_player(), "RED");
    assert_eq!(g.roll(), Ok(3));
    assert_eq!(g.move_token(0).unwrap(), "moved");
    assert_eq!(g.token_progress("RED", 0), Ok(19));
    assert_eq!(g.current_player(), "GREEN");
}

#[test]
fn should_send_a_captured_token_back_to_need_a_six_again() {
    let mut g = game!(2, "6 6 5 6 2 4 3 5 6");
    for _ in 0..7 {
        turn(&mut g, 0);
    }
    g.roll().unwrap(); // GREEN 5: its only token is in the yard
    assert!(g.legal_moves().is_empty());
    assert_eq!(g.current_player(), "RED");
}

#[test]
fn should_not_capture_on_the_cell_eight_after_a_start() {
    let mut g = game!(2, "6 4 6 4 5 4 6 6");
    for _ in 0..7 {
        turn(&mut g, 1);
    }
    assert_eq!(g.token_cell("GREEN", 1), Ok(21));

    assert_eq!(turn(&mut g, 1).unwrap(), "moved");

    assert_eq!(g.token_cell("RED", 1), Ok(21));
    assert_eq!(g.token_progress("GREEN", 1), Ok(9));
}

#[test]
fn should_not_capture_on_a_start_cell() {
    let mut g = game!(2, "6 3 6 6 1 6 4");
    for t in [0, 0, 0, 3, 0, 0] {
        turn(&mut g, t);
    }

    assert_eq!(turn(&mut g, 0).unwrap(), "moved");

    assert_eq!(g.token_cell("RED", 0), Ok(13));
    assert_eq!(g.token_progress("GREEN", 3), Ok(1));
}

#[test]
fn should_not_capture_a_block_of_two_tokens() {
    let mut g = game!(2, "6 4 6 2 5 6 2 6");
    for t in [0, 0, 0, 0, 0, 1, 1] {
        turn(&mut g, t);
    }
    assert_eq!(g.token_cell("GREEN", 0), Ok(15));
    assert_eq!(g.token_cell("GREEN", 1), Ok(15));

    assert_eq!(turn(&mut g, 0).unwrap(), "moved");

    assert_eq!(g.token_cell("RED", 0), Ok(15));
    assert_eq!(g.token_progress("GREEN", 0), Ok(3));
    assert_eq!(g.token_progress("GREEN", 1), Ok(3));
}

// ---- the home stretch ------------------------------------------------------------------------------------

#[test]
fn should_wrap_around_the_shared_track() {
    let mut g = game!(2, "1 6 6 4 1 6 6 4 1 6 6 4");
    for _ in 0..12 {
        turn(&mut g, 0);
    }

    assert_eq!(g.token_progress("GREEN", 0), Ok(43));
    assert_eq!(g.token_cell("GREEN", 0), Ok(3));
}

#[test]
fn should_reach_home_only_by_exact_count_and_get_an_extra_turn() {
    let mut g = game!(2, "6 6 4 1 6 6 4 1 6 6 4 1 6 6 3 1 2");
    for _ in 0..12 {
        turn(&mut g, 0);
    }
    assert_eq!(g.token_progress("RED", 0), Ok(43));
    assert_eq!(g.token_cell("RED", 0), Ok(42));
    turn(&mut g, 0);
    turn(&mut g, 0);
    assert_eq!(g.token_progress("RED", 0), Ok(55));
    assert_eq!(g.token_cell("RED", 0), Ok(-1));

    assert_eq!(g.roll(), Ok(3)); // 55 + 3 would pass HOME
    assert!(g.legal_moves().is_empty());
    assert_eq!(g.current_player(), "GREEN");
    g.roll().unwrap();
    assert_eq!(g.current_player(), "RED");

    assert_eq!(g.roll(), Ok(2));
    assert_eq!(g.move_token(0).unwrap(), "home");
    assert_eq!(g.token_progress("RED", 0), Ok(57));
    assert_eq!(g.current_player(), "RED");
    assert!(!g.is_over());
}

// ---- invalid calls ---------------------------------------------------------------------------------------

#[test]
fn should_reject_move_before_roll() {
    let mut g = game!(2, "6");

    assert_eq!(g.move_token(0), Err(LudoError::NoRollPending));
}

#[test]
fn should_reject_rolling_twice_while_a_move_is_pending() {
    let mut g = game!(2, "6 5");
    g.roll().unwrap();

    assert_eq!(g.roll(), Err(LudoError::RollPending));
    assert_eq!(g.legal_moves(), [0, 1, 2, 3]);
    g.move_token(0).unwrap();
    assert_eq!(g.roll(), Ok(5)); // the rejected roll consumed nothing
}

#[test]
fn should_reject_an_illegal_or_unknown_token_and_keep_the_roll() {
    let mut g = game!(2, "6 3");
    turn(&mut g, 0);
    g.roll().unwrap(); // 3: only token 0 can move

    assert_eq!(g.move_token(1), Err(LudoError::IllegalMove));
    assert_eq!(g.move_token(4), Err(LudoError::IllegalMove));
    assert_eq!(g.legal_moves(), [0]);
    assert_eq!(g.move_token(0).unwrap(), "moved");
    assert_eq!(g.token_progress("RED", 0), Ok(4));
}

#[test]
fn should_reject_move_after_an_automatically_resolved_roll() {
    let mut g = game!(2, "3");
    g.roll().unwrap();

    assert_eq!(g.move_token(0), Err(LudoError::NoRollPending));
}

#[test]
fn should_report_dice_exhausted_without_changing_the_game() {
    let mut g = game!(2, "6");
    g.roll().unwrap();
    g.move_token(0).unwrap();

    assert_eq!(g.roll(), Err(LudoError::DiceExhausted));
    assert_eq!(g.current_player(), "RED");
    assert_eq!(g.token_progress("RED", 0), Ok(1));
    assert!(!g.is_over());
}

#[test]
fn should_reject_unknown_colours_and_tokens_in_queries() {
    let g = game!(2, "1");

    assert_eq!(
        g.token_progress("YELLOW", 0),
        Err(LudoError::InvalidArgument)
    );
    assert_eq!(g.token_progress("PINK", 0), Err(LudoError::InvalidArgument));
    assert_eq!(g.token_progress("red", 0), Err(LudoError::InvalidArgument));
    assert_eq!(g.token_progress("RED", 4), Err(LudoError::InvalidArgument));
    assert_eq!(g.token_cell("RED", 9), Err(LudoError::InvalidArgument));
}

// ---- whole games -----------------------------------------------------------------------------------------

const GAME_1: &str = "3 1 6 4 1 6 4 1 6 6 3 1 4 1 5 1 6 6 2 1 6 6 5 1 6 5 1 2 1 6 6 5 1 6 6 3 1 6 4 1 6 3 1 2 2 6 2 1 6 4 1 6 6 4 1 3 1 4 1 6 6 3 1 6 2 6 6 6 1 5 1 6 6 6 1 6 6";
const GAME_2: &str = "4 6 6 6 4 2 6 4 2 4 1 4 4 5 6 2 4 4 6 6 5 4 6 3 3 3 4 6 4 6 6 3 6 5 2 6 6 6 3 3 4 5 1 5 5 5 2 4 5 5 6 3 3 6 3 5 3 4 4 3 3 6 6 5 2 4 4 6 3 6 5 6 5 6 6 3 6 4 2 6 5 3 4 5 5 5 6 5 5 3 6 6 6 5 6 3";
const GAME_3: &str = "4 1 4 2 1 5 6 5 3 6 4 3 6 4 6 6 4 5 3 4 3 3 6 4 5 5 3 2 6 6 4 5 5 3 6 4 3 4 6 4 6 6 4 5 2 1 6 6 4 4 4 2 4 3 4 3 6 4 2 4 5 6 6 4 2 2 2 6 6 6 2 3 3 6 6 6 6 1 1 5 6 3 2 2 2 6 6 4 3 6 2 6 3 6 4 1 3 6 2 5 4 6 4 2 4 5 6 6 6 4 4 4 6 4 3 3 4 4 2 5 6 2 6 3 4 1 5 6 3 5 4 1 5 3 4 6 2 6 3 6 6 4";

fn assert_finished(g: &mut G, winner: &str) {
    assert!(g.is_over());
    assert_eq!(g.winner().as_deref(), Some(winner));
    assert_eq!(g.current_player(), winner);
    assert!(g.legal_moves().is_empty());
    assert_eq!(g.roll(), Err(LudoError::GameOver));
    assert_eq!(g.move_token(0), Err(LudoError::GameOver));
}

#[test]
fn should_play_full_game_1_red_runs_alone_with_two_forfeited_sixes() {
    let mut g = game!(2, GAME_1);

    let specials = play_out(&mut g, &["lead", "first"]);

    assert!(specials.is_empty());
    assert_finished(&mut g, "RED");
    assert_progress(&g, "RED", [57, 57, 57, 57]);
    assert_progress(&g, "GREEN", [0, 0, 0, 0]);
}

#[test]
fn should_play_full_game_2_with_two_captures_and_a_safe_landing() {
    let mut g = game!(2, GAME_2);

    let specials = play_out(&mut g, &["lead", "first"]);

    assert_eq!(specials, ["captured:GREEN:1", "captured:GREEN:1"]);
    assert_finished(&mut g, "RED");
    assert_progress(&g, "RED", [57, 57, 57, 57]);
    assert_progress(&g, "GREEN", [46, 0, 0, 0]);
}

#[test]
fn should_play_full_game_3_with_four_players() {
    let mut g = game!(4, GAME_3);

    let specials = play_out(&mut g, &["lead", "first", "lead", "first"]);

    assert_eq!(
        specials,
        ["captured:RED:1", "captured:GREEN:1", "captured:BLUE:1"]
    );
    assert_finished(&mut g, "YELLOW");
    assert_progress(&g, "YELLOW", [57, 57, 57, 57]);
    assert_progress(&g, "RED", [44, 0, 0, 0]);
    assert_progress(&g, "GREEN", [17, 0, 0, 0]);
    assert_progress(&g, "BLUE", [57, 0, 0, 0]);
}
