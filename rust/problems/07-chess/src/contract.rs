//! Thin test entry point. Your domain model lives behind it.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Ok,
    Illegal,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    InProgress,
    Check,
    Checkmate,
    Stalemate,
}

pub trait ChessApi {
    /// Applies a move in coordinate notation ("e2e4", "e7e8q").
    fn make_move(&mut self, mv: &str) -> MoveResult;

    /// "WHITE" or "BLACK".
    fn turn(&self) -> String;

    fn status(&self) -> Status;

    /// "wP", "bK", ... or `None` for an empty/invalid square.
    fn piece_at(&self, square: &str) -> Option<String>;

    /// All legal moves for the side to move, sorted alphabetically.
    fn legal_moves(&self) -> Vec<String>;
}

#[allow(clippy::wrong_self_convention)]
pub trait ChessFactory {
    fn new_game(&self) -> Box<dyn ChessApi>;

    /// Creates a game from a FEN string used to set up positions.
    fn from_fen(&self, fen: &str) -> Box<dyn ChessApi>;
}
