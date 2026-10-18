#![allow(unused_imports, clippy::needless_range_loop)]
use p07_chess::{contract::*, create};
use playground_support::sut;

fn play(g: &mut dyn ChessApi, moves: &[&str]) {
    for m in moves {
        assert_eq!(g.make_move(m), MoveResult::Ok, "move {m}");
    }
}

/// Leaf positions after `depth` plies, found by replaying move sequences on fresh games.
fn perft(f: &dyn ChessFactory, fen: Option<&str>, prefix: &[String], depth: u32) -> u64 {
    let mut g = match fen {
        Some(fen) => f.from_fen(fen),
        None => f.new_game(),
    };
    for m in prefix {
        assert_eq!(
            g.make_move(m),
            MoveResult::Ok,
            "replay failed on {prefix:?}"
        );
    }
    let moves = g.legal_moves();
    if depth == 1 {
        return moves.len() as u64;
    }
    moves
        .iter()
        .map(|m| {
            let mut next = prefix.to_vec();
            next.push(m.clone());
            perft(f, fen, &next, depth - 1)
        })
        .sum()
}

#[test]
fn should_start_with_white_to_move_and_standard_setup() {
    let f = sut!(create());
    let g = f.new_game();

    assert_eq!(g.turn(), "WHITE");
    assert_eq!(g.status(), Status::InProgress);
    assert_eq!(g.piece_at("e1").as_deref(), Some("wK"));
    assert_eq!(g.piece_at("d8").as_deref(), Some("bQ"));
    assert_eq!(g.piece_at("a2").as_deref(), Some("wP"));
    assert_eq!(g.piece_at("h7").as_deref(), Some("bP"));
    assert_eq!(g.piece_at("e4"), None);
    assert_eq!(g.piece_at("z9"), None);
}

#[test]
fn should_have_twenty_legal_moves_in_the_starting_position() {
    let f = sut!(create());
    let g = f.new_game();
    let moves = g.legal_moves();

    assert_eq!(moves.len(), 20);
    for m in ["e2e4", "g1f3", "a2a3"] {
        assert!(moves.iter().any(|x| x == m), "{m} missing");
    }
    assert!(!moves.iter().any(|x| x == "e2e5"));
}

#[test]
fn should_apply_legal_move_and_switch_turn() {
    let f = sut!(create());
    let mut g = f.new_game();

    assert_eq!(g.make_move("e2e4"), MoveResult::Ok);

    assert_eq!(g.turn(), "BLACK");
    assert_eq!(g.piece_at("e4").as_deref(), Some("wP"));
    assert_eq!(g.piece_at("e2"), None);
}

#[test]
fn should_reject_illegal_moves_without_changing_state() {
    let f = sut!(create());
    let mut g = f.new_game();

    assert_eq!(g.make_move("e2e5"), MoveResult::Illegal); // pawn cannot advance three
    assert_eq!(g.make_move("e7e5"), MoveResult::Illegal); // not black's turn
    assert_eq!(g.make_move("e3e4"), MoveResult::Illegal); // empty square
    assert_eq!(g.make_move("a1a3"), MoveResult::Illegal); // rook blocked by pawn
    assert_eq!(g.make_move("nonsense"), MoveResult::Illegal);
    assert_eq!(g.make_move(""), MoveResult::Illegal);

    assert_eq!(g.turn(), "WHITE");
    assert_eq!(g.legal_moves().len(), 20);
    assert_eq!(g.piece_at("a1").as_deref(), Some("wR"));
}

#[test]
fn should_not_allow_piece_to_move_through_others() {
    let f = sut!(create());
    let mut g = f.new_game();

    assert_eq!(g.make_move("f1c4"), MoveResult::Illegal); // bishop blocked by e2
    assert_eq!(g.make_move("d1d3"), MoveResult::Illegal);
    assert_eq!(g.make_move("e2e4"), MoveResult::Ok);
    assert_eq!(g.make_move("e7e5"), MoveResult::Ok);
    assert_eq!(g.make_move("f1c4"), MoveResult::Ok); // now open
}

#[test]
fn should_allow_knights_to_jump() {
    let f = sut!(create());
    let mut g = f.new_game();

    assert_eq!(g.make_move("b1c3"), MoveResult::Ok);
    assert_eq!(g.piece_at("c3").as_deref(), Some("wN"));
}

#[test]
fn should_not_allow_pawn_to_capture_forward_or_move_diagonally_without_capture() {
    let f = sut!(create());
    let mut g = f.new_game();
    play(g.as_mut(), &["e2e4", "e7e5"]);

    assert_eq!(g.make_move("e4e5"), MoveResult::Illegal); // blocked head-on
    assert_eq!(g.make_move("e4d5"), MoveResult::Illegal); // diagonal without capture
}

#[test]
fn should_update_board_on_capture() {
    let f = sut!(create());
    let mut g = f.new_game();
    play(g.as_mut(), &["e2e4", "d7d5"]);

    assert_eq!(g.make_move("e4d5"), MoveResult::Ok);

    assert_eq!(g.piece_at("d5").as_deref(), Some("wP"));
    assert_eq!(g.piece_at("e4"), None);
}

#[test]
fn should_not_let_king_move_into_check() {
    let f = sut!(create());
    let mut g = f.from_fen("3rk3/8/8/8/8/8/8/4K3 w - - 0 1");

    assert_eq!(g.make_move("e1d1"), MoveResult::Illegal);
    assert_eq!(g.make_move("e1d2"), MoveResult::Illegal);
    assert_eq!(g.legal_moves(), vec!["e1e2", "e1f1", "e1f2"]);
}

#[test]
fn should_not_let_pinned_piece_expose_its_king() {
    let f = sut!(create());
    let mut g = f.from_fen("4r2k/8/8/8/8/8/4B3/4K3 w - - 0 1");

    assert_eq!(g.make_move("e2d3"), MoveResult::Illegal);
    assert!(g.legal_moves().iter().all(|m| !m.starts_with("e2")));
}

#[test]
fn should_report_check_and_only_allow_moves_that_resolve_it() {
    let f = sut!(create());
    let g = f.from_fen("4r2k/8/8/8/8/8/8/R3K3 w - - 0 1");

    assert_eq!(g.status(), Status::Check);
    assert_eq!(g.legal_moves(), vec!["e1d1", "e1d2", "e1f1", "e1f2"]);
}

#[test]
fn should_detect_checkmate() {
    let f = sut!(create());
    let mut g = f.new_game();

    play(g.as_mut(), &["f2f3", "e7e5", "g2g4", "d8h4"]);

    assert_eq!(g.status(), Status::Checkmate);
    assert!(g.legal_moves().is_empty());
    assert_eq!(g.make_move("a2a3"), MoveResult::GameOver);
}

#[test]
fn should_detect_stalemate() {
    let f = sut!(create());
    let mut g = f.from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");

    assert_eq!(g.status(), Status::Stalemate);
    assert!(g.legal_moves().is_empty());
    assert_eq!(g.make_move("h8g8"), MoveResult::GameOver);
}

#[test]
fn should_castle_kingside_and_queenside() {
    let f = sut!(create());
    let mut g = f.from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");

    assert_eq!(g.make_move("e1g1"), MoveResult::Ok);
    assert_eq!(g.piece_at("g1").as_deref(), Some("wK"));
    assert_eq!(g.piece_at("f1").as_deref(), Some("wR"));
    assert_eq!(g.piece_at("h1"), None);

    assert_eq!(g.make_move("e8c8"), MoveResult::Ok);
    assert_eq!(g.piece_at("c8").as_deref(), Some("bK"));
    assert_eq!(g.piece_at("d8").as_deref(), Some("bR"));
    assert_eq!(g.piece_at("a8"), None);
}

#[test]
fn should_not_castle_out_of_through_or_into_check() {
    let f = sut!(create());
    // in check
    assert_eq!(
        f.from_fen("4r2k/8/8/8/8/8/8/R3K2R w KQ - 0 1")
            .make_move("e1g1"),
        MoveResult::Illegal
    );
    // through an attacked square (f1)
    assert_eq!(
        f.from_fen("5r1k/8/8/8/8/8/8/R3K2R w KQ - 0 1")
            .make_move("e1g1"),
        MoveResult::Illegal
    );
    // into an attacked square (g1)
    assert_eq!(
        f.from_fen("6rk/8/8/8/8/8/8/R3K2R w KQ - 0 1")
            .make_move("e1g1"),
        MoveResult::Illegal
    );
    // the queenside is unaffected by an attack on g1/f1
    assert_eq!(
        f.from_fen("6rk/8/8/8/8/8/8/R3K2R w KQ - 0 1")
            .make_move("e1c1"),
        MoveResult::Ok
    );
}

#[test]
fn should_not_castle_when_path_is_blocked_or_rights_are_lost() {
    let f = sut!(create());
    assert_eq!(
        f.from_fen("4k3/8/8/8/8/8/8/R2QK2R w KQ - 0 1")
            .make_move("e1c1"),
        MoveResult::Illegal
    );
    assert_eq!(
        f.from_fen("4k3/8/8/8/8/8/8/R3K2R w - - 0 1")
            .make_move("e1g1"),
        MoveResult::Illegal
    );

    let mut g = f.from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    play(g.as_mut(), &["e1e2", "e8e7", "e2e1", "e7e8"]);
    assert_eq!(g.make_move("e1g1"), MoveResult::Illegal);
}

#[test]
fn should_capture_en_passant_only_immediately_after_two_square_advance() {
    let f = sut!(create());
    let mut g = f.new_game();
    play(g.as_mut(), &["e2e4", "a7a6", "e4e5", "d7d5"]);

    assert_eq!(g.make_move("e5d6"), MoveResult::Ok);
    assert_eq!(g.piece_at("d6").as_deref(), Some("wP"));
    assert_eq!(g.piece_at("d5"), None);

    let mut late = f.new_game();
    play(
        late.as_mut(),
        &["e2e4", "a7a6", "e4e5", "d7d5", "h2h3", "a6a5"],
    );
    assert_eq!(late.make_move("e5d6"), MoveResult::Illegal);
}

#[test]
fn should_promote_pawn_and_require_a_piece_choice() {
    let f = sut!(create());
    let mut g = f.from_fen("8/P6k/8/8/8/8/8/K7 w - - 0 1");

    assert_eq!(g.make_move("a7a8"), MoveResult::Illegal);
    let moves = g.legal_moves();
    for m in ["a7a8q", "a7a8r", "a7a8b", "a7a8n"] {
        assert!(moves.iter().any(|x| x == m), "{m} missing");
    }
    assert_eq!(g.make_move("a7a8q"), MoveResult::Ok);
    assert_eq!(g.piece_at("a8").as_deref(), Some("wQ"));
    assert_eq!(g.piece_at("a7"), None);
}

#[test]
fn should_allow_underpromotion_and_reject_invalid_promotion_pieces() {
    let f = sut!(create());
    let mut g = f.from_fen("8/P6k/8/8/8/8/8/K7 w - - 0 1");

    assert_eq!(g.make_move("a7a8k"), MoveResult::Illegal);
    assert_eq!(g.make_move("a7a8p"), MoveResult::Illegal);
    assert_eq!(g.make_move("a7a8n"), MoveResult::Ok);
    assert_eq!(g.piece_at("a8").as_deref(), Some("wN"));
}

#[test]
fn should_count_legal_moves_correctly_in_the_perft_positions() {
    let f = sut!(create());
    assert_eq!(f.new_game().legal_moves().len(), 20);
    let cases = [
        (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            48,
        ),
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 14),
        (
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            6,
        ),
        (
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            44,
        ),
    ];
    for (fen, expected) in cases {
        assert_eq!(f.from_fen(fen).legal_moves().len(), expected, "{fen}");
    }
}

#[test]
fn should_match_known_perft_to_depth_three_from_the_starting_position() {
    let f = sut!(create());

    assert_eq!(perft(f.as_ref(), None, &[], 2), 400);
    assert_eq!(perft(f.as_ref(), None, &[], 3), 8902);
}

#[test]
fn should_match_known_perft_to_depth_two_in_tricky_positions() {
    let f = sut!(create());
    let cases = [
        (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            2039,
        ),
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 191),
        (
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            264,
        ),
        (
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            1486,
        ),
    ];
    for (fen, expected) in cases {
        assert_eq!(perft(f.as_ref(), Some(fen), &[], 2), expected, "{fen}");
    }
}
