//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum LeaderboardError {
    /// the player id is not registered
    UnknownPlayer,
    /// the team id is not registered
    UnknownTeam,
    /// a player id or a team id is already taken
    AlreadyExists,
    /// blank id/name/owner, role or event type not in the fixed lists, or count < 1
    InvalidArgument,
    /// roster rules broken: not 11 distinct players, captain/vice-captain missing or equal, no KEEPER, more than 6 BATTERs
    InvalidRoster,
    /// `correct_event` asked to reverse more events of that type than are currently recorded for the player
    CorrectionExceedsRecorded,
}

pub trait FantasyLeaderboard: Send + Sync {
    /// Registers a player. `role` is exactly one of "BATTER", "BOWLER", "ALL_ROUNDER", "KEEPER" (case-sensitive).
    /// Errors: `InvalidArgument` (blank id or name, role not one of the four), `AlreadyExists` (id taken).
    /// Checked in that order.
    fn add_player(&self, player_id: &str, name: &str, role: &str) -> Result<(), LeaderboardError>;

    /// Creates a team. Valid only if: team_id and owner are non-blank and team_id is new; player_ids holds exactly 11
    /// distinct registered players; captain_id and vice_captain_id are both in player_ids and differ; at least one
    /// player is a KEEPER; at most six are BATTERs. Nothing is created on error.
    /// Errors, checked in this order: `InvalidArgument` (blank team_id or owner), `AlreadyExists` (team_id taken),
    /// `InvalidRoster` (not 11 entries, or a repeated player), `UnknownPlayer` (some player_ids entry is not
    /// registered), `InvalidRoster` (captain or vice-captain missing from the list or equal, no KEEPER, more than 6 BATTERs).
    /// The team's score immediately reflects the players' current match points, including earlier events.
    fn create_team(
        &self,
        team_id: &str,
        owner: &str,
        player_ids: &[&str],
        captain_id: &str,
        vice_captain_id: &str,
    ) -> Result<(), LeaderboardError>;

    /// Adds `count * points(event_type)` to the player's match points. Event types (exact, case-sensitive) and points:
    /// RUN 1, FOUR_BONUS 1, SIX_BONUS 2, WICKET 25, CATCH 8, STUMPING 12, MAIDEN 12, DUCK -2.
    /// Errors, checked in this order: `UnknownPlayer`, `InvalidArgument` (unknown event type, or count == 0).
    /// Every team containing the player is updated before this call returns.
    fn record_event(
        &self,
        player_id: &str,
        event_type: &str,
        count: u32,
    ) -> Result<(), LeaderboardError>;

    /// Reverses `count` earlier events of that type for that player (subtracts `count * points`).
    /// Errors, checked in this order: `UnknownPlayer`, `InvalidArgument` (unknown event type, or count == 0),
    /// `CorrectionExceedsRecorded` (count is more than the net number of events of that type currently recorded
    /// for the player; state is left unchanged).
    fn correct_event(
        &self,
        player_id: &str,
        event_type: &str,
        count: u32,
    ) -> Result<(), LeaderboardError>;

    /// The first `k` entries of the leaderboard, each formatted "rank:team_id:score_half_points", ordered by score
    /// descending and, for equal scores, by team creation order (earliest first). Ranks use competition ranking.
    /// Fewer than k entries when there are fewer teams; k == 0 gives an empty list. The result is a consistent
    /// snapshot: every event is reflected in all teams or in none.
    fn leaderboard(&self, k: usize) -> Vec<String>;

    /// Competition rank: 1 + the number of teams with a strictly higher score. Error: `UnknownTeam`.
    fn rank_of(&self, team_id: &str) -> Result<usize, LeaderboardError>;

    /// The team's score in half points (see README). Error: `UnknownTeam`.
    fn team_score(&self, team_id: &str) -> Result<i64, LeaderboardError>;

    /// The player's match points (whole points, may be negative). Error: `UnknownPlayer`.
    fn player_points(&self, player_id: &str) -> Result<i64, LeaderboardError>;
}
