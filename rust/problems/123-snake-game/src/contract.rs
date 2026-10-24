//! Thin test entry point. Your design lives behind it.
//! Single-threaded: one caller at a time (`&mut self`).
//!
//! Coordinates are (row, col) with (0,0) top-left; rows run 0..height-1, columns 0..width-1.
//! Cells are always rendered as the string "row,col" (no spaces).

#[derive(Debug, PartialEq, Eq)]
pub enum GameError {
    /// width/height below 1, a food outside the board (at creation), or a direction that is not exactly
    /// "U", "D", "L" or "R" (at any time, also after the game is over).
    InvalidArgument,
}

pub trait SnakeGame {
    /// Moves the head one cell. `direction` is exactly one of "U" (row-1), "D" (row+1), "L" (col-1), "R" (col+1).
    /// Anything else is `Err(InvalidArgument)` and changes nothing.
    ///
    /// Returns `Ok(score)` after the move, or `Ok(-1)` if the game is over after this move (a fatal move, a
    /// winning move, or any move made when the game was already over; moves after game over change nothing).
    fn make_move(&mut self, direction: &str) -> Result<i32, GameError>;

    /// Foods eaten so far (always `length() - 1`).
    fn score(&self) -> i32;

    /// True once the snake hit a wall or itself, reversed on itself, or filled the board.
    fn is_over(&self) -> bool;

    /// True only if the game ended because the snake fills every cell of the board.
    fn is_won(&self) -> bool;

    /// Number of cells the snake occupies (starts at 1).
    fn length(&self) -> usize;

    /// The head cell as "row,col".
    fn head(&self) -> String;

    /// Every snake cell as "row,col", head first, tail last.
    fn body(&self) -> Vec<String>;

    /// The cell of the food currently on the board as "row,col", or "none".
    fn food_position(&self) -> String;
}
