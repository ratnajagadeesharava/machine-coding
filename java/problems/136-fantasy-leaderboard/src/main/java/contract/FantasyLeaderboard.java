package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface FantasyLeaderboard {

    /**
     * Registers a player. {@code role} is exactly one of BATTER, BOWLER, ALL_ROUNDER, KEEPER (case-sensitive).
     * IllegalArgumentException if the id or name is null/blank, the role is not one of the four, or the id exists.
     */
    void addPlayer(String playerId, String name, String role);

    /**
     * Creates a team. Valid only if: teamId and owner are non-blank and the teamId is new; playerIds holds exactly 11
     * distinct registered players; captainId and viceCaptainId are both in playerIds and are different; at least one
     * player is a KEEPER; at most six are BATTERs. Otherwise IllegalArgumentException and nothing is created.
     * The team's score immediately reflects the players' current match points, including events recorded earlier.
     */
    void createTeam(String teamId, String owner, List<String> playerIds, String captainId, String viceCaptainId);

    /**
     * Adds {@code count * points(eventType)} to the player's match points. Event types (exact, case-sensitive) and
     * points: RUN 1, FOUR_BONUS 1, SIX_BONUS 2, WICKET 25, CATCH 8, STUMPING 12, MAIDEN 12, DUCK -2.
     * IllegalArgumentException for an unknown player, an unknown event type or count &lt; 1.
     * Every team containing the player is updated before this call returns.
     */
    void recordEvent(String playerId, String eventType, int count);

    /**
     * Reverses {@code count} earlier events of that type for that player (subtracts {@code count * points}).
     * IllegalArgumentException for an unknown player, unknown event type, count &lt; 1, or when {@code count} is more
     * than the net number of events of that type currently recorded for the player (state is left unchanged).
     */
    void correctEvent(String playerId, String eventType, int count);

    /**
     * The first {@code k} entries of the leaderboard, each formatted {@code "rank:teamId:scoreHalfPoints"}, ordered by
     * score descending and, for equal scores, by team creation order (earliest first). Ranks use competition ranking.
     * Returns fewer than k entries when there are fewer teams; k = 0 gives an empty list.
     * IllegalArgumentException when k is negative. The result is a consistent snapshot: every event is reflected in
     * all teams or in none.
     */
    List<String> leaderboard(int k);

    /** Competition rank of the team: 1 + the number of teams with a strictly higher score. Unknown team: IllegalArgumentException. */
    int rankOf(String teamId);

    /** The team's score in half points (see README). Unknown team: IllegalArgumentException. */
    long teamScore(String teamId);

    /** The player's match points (whole points, may be negative). Unknown player: IllegalArgumentException. */
    long playerPoints(String playerId);
}
