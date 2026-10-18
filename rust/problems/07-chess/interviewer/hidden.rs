#![allow(unused_imports, clippy::needless_range_loop)]
use p07_chess::{contract::*, create};
use playground_support::sut;

fn perft(f: &dyn ChessFactory, fen: &str, prefix: &[String], depth: u32) -> u64 {
    let mut g = f.from_fen(fen);
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
fn should_match_known_perft_to_depth_three_in_tricky_positions() {
    let f = sut!(create());
    assert_eq!(
        perft(
            f.as_ref(),
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            &[],
            3
        ),
        2812
    );
    assert_eq!(
        perft(
            f.as_ref(),
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            &[],
            3
        ),
        9467
    );
    assert_eq!(
        perft(
            f.as_ref(),
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            &[],
            3
        ),
        97862
    );
}

#[test]
fn should_not_allow_en_passant_that_exposes_the_king_along_the_rank() {
    let f = sut!(create());
    let mut g = f.from_fen("8/8/8/K2pP2r/8/8/8/7k w - d6 0 1");
    assert_eq!(g.make_move("e5d6"), MoveResult::Illegal);
}

#[test]
fn should_lose_castling_right_when_rook_is_captured_even_if_another_rook_returns_to_the_square() {
    let f = sut!(create());
    let mut g = f.from_fen("4k3/8/8/8/8/7R/6b1/R3K2R b KQ - 0 1");
    assert_eq!(g.make_move("g2h1"), MoveResult::Ok); // bishop takes the h1 rook
    assert_eq!(g.make_move("h3h1"), MoveResult::Ok); // another rook retakes on h1
    assert_eq!(g.make_move("e8e7"), MoveResult::Ok);
    assert_eq!(g.make_move("e1g1"), MoveResult::Illegal);
    assert!(g.legal_moves().iter().any(|m| m == "e1c1"));
}

#[test]
fn should_answer_double_check_with_king_move_only() {
    let f = sut!(create());
    let g = f.from_fen("4k3/8/8/1B6/8/8/8/4R1K1 b - - 0 1");
    assert_eq!(g.status(), Status::Check);
    assert!(g.legal_moves().iter().all(|m| m.starts_with("e8")));
}

#[test]
fn should_allow_promotion_with_capture() {
    let f = sut!(create());
    let mut g = f.from_fen("1n5k/P7/8/8/8/8/8/K7 w - - 0 1");
    let moves = g.legal_moves();
    for m in ["a7a8q", "a7b8q", "a7b8n"] {
        assert!(moves.iter().any(|x| x == m), "{m} missing");
    }
    assert_eq!(g.make_move("a7b8q"), MoveResult::Ok);
    assert_eq!(g.piece_at("b8").as_deref(), Some("wQ"));
}

#[test]
fn should_detect_back_rank_mate() {
    let f = sut!(create());
    let mut g = f.from_fen("6k1/5ppp/8/8/8/8/8/R3K3 w - - 0 1");
    assert_eq!(g.make_move("a1a8"), MoveResult::Ok);
    assert_eq!(g.status(), Status::Checkmate);
}
