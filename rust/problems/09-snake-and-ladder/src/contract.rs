//! Thin test entry point. Your domain model lives behind it.

pub trait SnakeAndLadderApi {
    /// Applies the dice value for the current player. Returns the mover's new position, or -1 if refused.
    fn roll(&mut self, dice: i32) -> i32;

    /// `None` after the game is over.
    fn current_player(&self) -> Option<String>;

    /// 0 before entering the board; -1 for unknown players.
    fn position_of(&self, player: &str) -> i32;

    fn winner(&self) -> Option<String>;
}
