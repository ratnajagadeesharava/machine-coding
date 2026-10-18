//! Thin test entry point. Your domain model lives behind it.

pub trait TicTacToeApi {
    /// OK, WIN, DRAW, GAME_OVER, NOT_YOUR_TURN, INVALID_PLAYER, OUT_OF_BOUNDS or OCCUPIED.
    fn play(&mut self, player: &str, row: i32, col: i32) -> String;

    /// "X" or "O"; `None` once the game is over.
    fn next_player(&self) -> Option<String>;

    fn winner(&self) -> Option<String>;

    /// "X", "O", or `None` for an empty/out-of-bounds cell.
    fn cell_at(&self, row: i32, col: i32) -> Option<String>;
}
