//! Thin test entry point for one game of Ludo. Your design lives behind it. Single-threaded use.
//!
//! Colours are the strings `"RED"`, `"GREEN"`, `"YELLOW"`, `"BLUE"`; a game with N players uses the first N of
//! them, in that turn order. Tokens are numbered 0..=3. A token's *progress* is 0 in the yard, 1..=51 on the shared
//! track, 52..=56 in its home column and 57 once it is HOME. The full rules are in the README.

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LudoError {
    /// bad player count, a die outside 1..=6, unknown colour or a token index outside 0..=3 in a query
    InvalidArgument,
    /// the game has been won
    GameOver,
    /// `roll` while the previous roll still waits for a `move_token`
    RollPending,
    /// `move_token` while no roll is pending
    NoRollPending,
    /// `move_token` with a token that is not one of `legal_moves()` (including an index above 3)
    IllegalMove,
    /// all injected dice are used up (nothing changes)
    DiceExhausted,
}

pub trait Ludo: Send {
    /// Rolls the next injected die for the current player and returns it (1..=6).
    /// Errors: `GameOver`, `RollPending`, `DiceExhausted`.
    /// If the roll gives no legal move, or it is the third consecutive 6, the turn is resolved automatically
    /// before this method returns.
    fn roll(&mut self) -> Result<u8, LudoError>;

    /// Token indexes the current player may move with the pending roll, ascending. Empty if no roll is pending.
    fn legal_moves(&self) -> Vec<usize>;

    /// Moves the chosen token by the pending roll and returns `"moved"`, `"home"` or `"captured:<COLOUR>:<n>"`
    /// (several captured colours are joined with `","` in turn order).
    /// Errors: `GameOver`, `NoRollPending`, `IllegalMove`.
    fn move_token(&mut self, token: usize) -> Result<String, LudoError>;

    /// The colour whose turn it is (after the game is over: the winner).
    fn current_player(&self) -> String;

    /// Progress 0..=57 of the token. `InvalidArgument` for a colour not in this game or a token above 3.
    fn token_progress(&self, colour: &str, token: usize) -> Result<u32, LudoError>;

    /// Shared track cell 0..=51 the token stands on, or -1 if it is in the yard, in its home column or HOME.
    fn token_cell(&self, colour: &str, token: usize) -> Result<i32, LudoError>;

    /// The winning colour, `None` while the game is running.
    fn winner(&self) -> Option<String>;

    fn is_over(&self) -> bool;
}
