//! Thin test entry point. Your domain model lives behind it.
//! One instance is one game in progress.

pub trait BoardGameApi {
    /// Returns OK, WIN, DRAW, INVALID_MOVE, NOT_YOUR_TURN or GAME_OVER.
    fn play(&mut self, player: &str, mv: &str) -> String;

    /// "X" (P1), "O" (P2) or "." (empty or out of the board). Connect4: row 0 is the bottom row.
    fn cell_at(&self, row: i32, col: i32) -> String;

    /// One line per row from the visual top to the bottom, joined with "\n".
    fn render(&self) -> String;

    /// "P1" or "P2"; `None` once the game is over.
    fn next_player(&self) -> Option<String>;
}
