package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.LudoGame;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    static List<Integer> dice(String s) {
        List<Integer> out = new ArrayList<>();
        for (String p : s.trim().split("\\s+")) {
            out.add(Integer.parseInt(p));
        }
        return out;
    }

    private LudoGame game(int players, String dice) {
        return Candidate.of(() -> Solution.create(players, dice(dice)));
    }

    /** Rolls once and, if a move exists, plays the given token. Returns the move description or null. */
    private static String turn(LudoGame g, int token) {
        g.roll();
        return g.legalMoves().isEmpty() ? null : g.move(token);
    }

    /** Plays a whole game: "lead" = most advanced legal token, "first" = lowest index, "rear" = least advanced. */
    static List<String> playOut(LudoGame g, String... policies) {
        List<String> specials = new ArrayList<>();
        String[] colours = {"RED", "GREEN", "YELLOW", "BLUE"};
        while (!g.isOver()) {
            String who = g.currentPlayer();
            int seat = Arrays.asList(colours).indexOf(who);
            g.roll();
            List<Integer> legal = g.legalMoves();
            if (legal.isEmpty()) {
                continue;
            }
            int pick = legal.get(0);
            if (policies[seat].equals("lead")) {
                for (int t : legal) {
                    if (g.tokenProgress(who, t) > g.tokenProgress(who, pick)) {
                        pick = t;
                    }
                }
            } else if (policies[seat].equals("rear")) {
                for (int t : legal) {
                    if (g.tokenProgress(who, t) < g.tokenProgress(who, pick)) {
                        pick = t;
                    }
                }
            }
            String d = g.move(pick);
            if (!d.equals("moved") && !d.equals("home")) {
                specials.add(d);
            }
        }
        return specials;
    }

    // ---- set-up and turn order -------------------------------------------------------------------------------

    @Test
    void shouldStartWithAllTokensInTheYardAndRedToMove() {
        LudoGame g = game(4, "1");

        assertThat(g.currentPlayer()).isEqualTo("RED");
        assertThat(g.isOver()).isFalse();
        assertThat(g.winner()).isEmpty();
        assertThat(g.legalMoves()).isEmpty();
        for (String c : new String[] {"RED", "GREEN", "YELLOW", "BLUE"}) {
            for (int t = 0; t < 4; t++) {
                assertThat(g.tokenProgress(c, t)).isZero();
                assertThat(g.tokenCell(c, t)).isEqualTo(-1);
            }
        }
    }

    @Test
    void shouldRejectInvalidSetup() {
        game(2, "1"); // a valid instance first: the factory must be wired
        assertThatThrownBy(() -> Solution.create(1, dice("1"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(5, dice("1"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(2, null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(2, dice("3 0 2"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(2, dice("3 7 2"))).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldUseTheFirstNColoursInTurnOrder() {
        LudoGame three = game(3, "1 1 1 1");
        List<String> order = new ArrayList<>();
        for (int i = 0; i < 4; i++) {
            order.add(three.currentPlayer());
            three.roll();
        }
        assertThat(order).containsExactly("RED", "GREEN", "YELLOW", "RED");
        assertThatThrownBy(() -> three.tokenProgress("BLUE", 0)).isInstanceOf(IllegalArgumentException.class);

        LudoGame four = game(4, "1 1 1 1 1");
        order.clear();
        for (int i = 0; i < 5; i++) {
            order.add(four.currentPlayer());
            four.roll();
        }
        assertThat(order).containsExactly("RED", "GREEN", "YELLOW", "BLUE", "RED");
    }

    @Test
    void shouldPassTheTurnWhenNoTokenCanMove() {
        LudoGame g = game(2, "3 4");

        assertThat(g.roll()).isEqualTo(3);
        assertThat(g.legalMoves()).isEmpty();
        assertThat(g.currentPlayer()).isEqualTo("GREEN");
        assertThat(g.roll()).isEqualTo(4);
        assertThat(g.currentPlayer()).isEqualTo("RED");
    }

    // ---- leaving the yard and moving ---------------------------------------------------------------------------

    @Test
    void shouldLeaveTheYardOnlyOnASixOntoTheStartCell() {
        LudoGame g = game(2, "3 6 6");

        g.roll(); // RED: 3, nothing moves
        assertThat(g.roll()).isEqualTo(6); // GREEN
        assertThat(g.legalMoves()).containsExactly(0, 1, 2, 3);
        assertThat(g.move(2)).isEqualTo("moved");
        assertThat(g.tokenProgress("GREEN", 2)).isEqualTo(1);
        assertThat(g.tokenCell("GREEN", 2)).isEqualTo(13);
        assertThat(g.tokenProgress("GREEN", 0)).isZero();

        g.roll(); // GREEN again: 6
        g.move(0);
        assertThat(g.tokenCell("GREEN", 0)).isEqualTo(13);
    }

    @Test
    void shouldMoveTheChosenTokenByTheRolledValue() {
        LudoGame g = game(2, "6 6 3");

        turn(g, 1);
        turn(g, 0);
        assertThat(g.roll()).isEqualTo(3);
        assertThat(g.move(1)).isEqualTo("moved");

        assertThat(g.tokenProgress("RED", 1)).isEqualTo(4);
        assertThat(g.tokenProgress("RED", 0)).isEqualTo(1);
        assertThat(g.tokenCell("RED", 1)).isEqualTo(3);
        assertThat(g.currentPlayer()).isEqualTo("GREEN");
    }

    @Test
    void shouldListLegalTokensInAscendingOrder() {
        LudoGame g = game(2, "6 6 3 6");

        turn(g, 2);
        turn(g, 3);
        g.roll(); // 3: only the two tokens on the track can move
        assertThat(g.legalMoves()).containsExactly(2, 3);
        g.move(3);
        g.roll(); // GREEN 6
        assertThat(g.legalMoves()).containsExactly(0, 1, 2, 3);
    }

    @Test
    void shouldGiveAnExtraTurnAfterASix() {
        LudoGame g = game(2, "6 2");

        g.roll();
        g.move(0);
        assertThat(g.currentPlayer()).isEqualTo("RED");
        g.roll();
        g.move(0);
        assertThat(g.currentPlayer()).isEqualTo("GREEN");
    }

    @Test
    void shouldForfeitTheThirdConsecutiveSix() {
        LudoGame g = game(2, "6 6 6 2");

        turn(g, 0);
        turn(g, 0);
        assertThat(g.roll()).isEqualTo(6);

        assertThat(g.legalMoves()).isEmpty();
        assertThat(g.tokenProgress("RED", 0)).isEqualTo(7);
        assertThat(g.currentPlayer()).isEqualTo("GREEN");
        g.roll();
        assertThat(g.currentPlayer()).isEqualTo("RED");
    }

    @Test
    void shouldRestartTheCountOfSixesWhenTheTurnPasses() {
        LudoGame g = game(2, "6 6 5 1 6 6 4");

        turn(g, 0);
        turn(g, 0);
        turn(g, 0);
        assertThat(g.currentPlayer()).isEqualTo("GREEN");
        g.roll();
        turn(g, 0);
        turn(g, 0);
        assertThat(g.roll()).isEqualTo(4);
        assertThat(g.legalMoves()).containsExactly(0);
        g.move(0);

        assertThat(g.tokenProgress("RED", 0)).isEqualTo(28);
        assertThat(g.currentPlayer()).isEqualTo("GREEN");
    }

    // ---- capturing -----------------------------------------------------------------------------------------------

    @Test
    void shouldCaptureASingleOpponentOnAnUnsafeCell() {
        LudoGame g = game(2, "6 6 5 6 2 4 3");

        turn(g, 0);
        turn(g, 0);
        turn(g, 0); // RED token 0 at progress 12
        turn(g, 0); // GREEN leaves
        turn(g, 0); // GREEN to progress 3 = cell 15
        assertThat(g.tokenCell("GREEN", 0)).isEqualTo(15);
        assertThat(g.currentPlayer()).isEqualTo("RED");

        g.roll(); // 4
        assertThat(g.move(0)).isEqualTo("captured:GREEN:1");

        assertThat(g.tokenProgress("GREEN", 0)).isZero();
        assertThat(g.tokenProgress("RED", 0)).isEqualTo(16);
        assertThat(g.tokenCell("RED", 0)).isEqualTo(15);
    }

    @Test
    void shouldGiveAnExtraTurnAfterACapture() {
        LudoGame g = game(2, "6 6 5 6 2 4 3");
        for (int i = 0; i < 5; i++) {
            turn(g, 0);
        }
        turn(g, 0); // the capturing 4

        assertThat(g.currentPlayer()).isEqualTo("RED");
        assertThat(g.roll()).isEqualTo(3);
        assertThat(g.move(0)).isEqualTo("moved");
        assertThat(g.tokenProgress("RED", 0)).isEqualTo(19);
        assertThat(g.currentPlayer()).isEqualTo("GREEN");
    }

    @Test
    void shouldSendACapturedTokenBackToNeedASixAgain() {
        LudoGame g = game(2, "6 6 5 6 2 4 3 5 6");
        for (int i = 0; i < 6; i++) {
            turn(g, 0);
        }
        turn(g, 0); // RED's 3
        g.roll(); // GREEN 5: its only token is in the yard
        assertThat(g.legalMoves()).isEmpty();
        assertThat(g.currentPlayer()).isEqualTo("RED");
    }

    @Test
    void shouldNotCaptureOnTheCellEightAfterAStart() {
        LudoGame g = game(2, "6 4 6 4 5 4 6 6");
        for (int i = 0; i < 7; i++) {
            turn(g, 1);
        }
        assertThat(g.tokenCell("GREEN", 1)).isEqualTo(21);

        assertThat(turn(g, 1)).isEqualTo("moved");

        assertThat(g.tokenCell("RED", 1)).isEqualTo(21);
        assertThat(g.tokenProgress("GREEN", 1)).isEqualTo(9);
    }

    @Test
    void shouldNotCaptureOnAStartCell() {
        LudoGame g = game(2, "6 3 6 6 1 6 4");
        turn(g, 0);
        turn(g, 0);
        turn(g, 0);
        turn(g, 3);
        turn(g, 0);
        turn(g, 0);

        assertThat(turn(g, 0)).isEqualTo("moved");

        assertThat(g.tokenCell("RED", 0)).isEqualTo(13);
        assertThat(g.tokenProgress("GREEN", 3)).isEqualTo(1);
    }

    @Test
    void shouldNotCaptureABlockOfTwoTokens() {
        LudoGame g = game(2, "6 4 6 2 5 6 2 6");
        int[] picks = {0, 0, 0, 0, 0, 1, 1};
        for (int p : picks) {
            turn(g, p);
        }
        assertThat(g.tokenCell("GREEN", 0)).isEqualTo(15);
        assertThat(g.tokenCell("GREEN", 1)).isEqualTo(15);

        assertThat(turn(g, 0)).isEqualTo("moved");

        assertThat(g.tokenCell("RED", 0)).isEqualTo(15);
        assertThat(g.tokenProgress("GREEN", 0)).isEqualTo(3);
        assertThat(g.tokenProgress("GREEN", 1)).isEqualTo(3);
    }

    // ---- the home stretch ---------------------------------------------------------------------------------------

    @Test
    void shouldWrapAroundTheSharedTrack() {
        LudoGame g = game(2, "1 6 6 4 1 6 6 4 1 6 6 4");
        for (int i = 0; i < 12; i++) {
            turn(g, 0);
        }

        assertThat(g.tokenProgress("GREEN", 0)).isEqualTo(43);
        assertThat(g.tokenCell("GREEN", 0)).isEqualTo(3);
    }

    @Test
    void shouldReachHomeOnlyByExactCountAndGetAnExtraTurn() {
        LudoGame g = game(2, "6 6 4 1 6 6 4 1 6 6 4 1 6 6 3 1 2");
        for (int i = 0; i < 12; i++) {
            turn(g, 0);
        }
        assertThat(g.tokenProgress("RED", 0)).isEqualTo(43);
        assertThat(g.tokenCell("RED", 0)).isEqualTo(42);
        turn(g, 0);
        turn(g, 0);
        assertThat(g.tokenProgress("RED", 0)).isEqualTo(55);
        assertThat(g.tokenCell("RED", 0)).isEqualTo(-1);

        assertThat(g.roll()).isEqualTo(3); // 55 + 3 would pass HOME
        assertThat(g.legalMoves()).isEmpty();
        assertThat(g.currentPlayer()).isEqualTo("GREEN");
        g.roll();
        assertThat(g.currentPlayer()).isEqualTo("RED");

        assertThat(g.roll()).isEqualTo(2);
        assertThat(g.move(0)).isEqualTo("home");
        assertThat(g.tokenProgress("RED", 0)).isEqualTo(57);
        assertThat(g.currentPlayer()).isEqualTo("RED");
        assertThat(g.isOver()).isFalse();
    }

    // ---- invalid calls ---------------------------------------------------------------------------------------------

    @Test
    void shouldRejectMoveBeforeRoll() {
        LudoGame g = game(2, "6");

        assertThatThrownBy(() -> g.move(0)).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldRejectRollingTwiceWhileAMoveIsPending() {
        LudoGame g = game(2, "6 5");
        g.roll();

        assertThatThrownBy(g::roll).isInstanceOf(IllegalStateException.class);
        assertThat(g.legalMoves()).containsExactly(0, 1, 2, 3);
        g.move(0);
        assertThat(g.roll()).isEqualTo(5); // the rejected roll consumed nothing
    }

    @Test
    void shouldRejectAnIllegalOrUnknownTokenAndKeepTheRoll() {
        LudoGame g = game(2, "6 3");
        turn(g, 0);
        g.roll(); // 3: only token 0 can move

        assertThatThrownBy(() -> g.move(1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.move(4)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.move(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThat(g.legalMoves()).containsExactly(0);
        assertThat(g.move(0)).isEqualTo("moved");
        assertThat(g.tokenProgress("RED", 0)).isEqualTo(4);
    }

    @Test
    void shouldRejectMoveAfterAnAutomaticallyResolvedRoll() {
        LudoGame g = game(2, "3");
        g.roll();

        assertThatThrownBy(() -> g.move(0)).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldReportDiceExhaustedWithoutChangingTheGame() {
        LudoGame g = game(2, "6");
        g.roll();
        g.move(0);

        assertThatThrownBy(g::roll).isInstanceOf(IllegalStateException.class).hasMessageContaining("dice exhausted");
        assertThat(g.currentPlayer()).isEqualTo("RED");
        assertThat(g.tokenProgress("RED", 0)).isEqualTo(1);
        assertThat(g.isOver()).isFalse();
    }

    @Test
    void shouldRejectUnknownColoursAndTokensInQueries() {
        LudoGame g = game(2, "1");

        assertThatThrownBy(() -> g.tokenProgress("YELLOW", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.tokenProgress("PINK", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.tokenProgress("red", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.tokenProgress("RED", 4)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.tokenCell("RED", -1)).isInstanceOf(IllegalArgumentException.class);
    }

    // ---- whole games ---------------------------------------------------------------------------------------------

    private static final String GAME_1 = "3 1 6 4 1 6 4 1 6 6 3 1 4 1 5 1 6 6 2 1 6 6 5 1 6 5 1 2 1 6 6 5 1 6 6 3 1 6 4 1 6 3 1 2 2 6 2 1 6 4 1 6 6 4 1 3 1 4 1 6 6 3 1 6 2 6 6 6 1 5 1 6 6 6 1 6 6";
    private static final String GAME_2 = "4 6 6 6 4 2 6 4 2 4 1 4 4 5 6 2 4 4 6 6 5 4 6 3 3 3 4 6 4 6 6 3 6 5 2 6 6 6 3 3 4 5 1 5 5 5 2 4 5 5 6 3 3 6 3 5 3 4 4 3 3 6 6 5 2 4 4 6 3 6 5 6 5 6 6 3 6 4 2 6 5 3 4 5 5 5 6 5 5 3 6 6 6 5 6 3";
    private static final String GAME_3 = "4 1 4 2 1 5 6 5 3 6 4 3 6 4 6 6 4 5 3 4 3 3 6 4 5 5 3 2 6 6 4 5 5 3 6 4 3 4 6 4 6 6 4 5 2 1 6 6 4 4 4 2 4 3 4 3 6 4 2 4 5 6 6 4 2 2 2 6 6 6 2 3 3 6 6 6 6 1 1 5 6 3 2 2 2 6 6 4 3 6 2 6 3 6 4 1 3 6 2 5 4 6 4 2 4 5 6 6 6 4 4 4 6 4 3 3 4 4 2 5 6 2 6 3 4 1 5 6 3 5 4 1 5 3 4 6 2 6 3 6 6 4";

    private static void assertProgress(LudoGame g, String colour, int... expected) {
        for (int t = 0; t < 4; t++) {
            assertThat(g.tokenProgress(colour, t)).as(colour + " token " + t).isEqualTo(expected[t]);
        }
    }

    private static void assertFinished(LudoGame g, String winner) {
        assertThat(g.isOver()).isTrue();
        assertThat(g.winner()).contains(winner);
        assertThat(g.currentPlayer()).isEqualTo(winner);
        assertThat(g.legalMoves()).isEmpty();
        assertThatThrownBy(g::roll).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> g.move(0)).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldPlayFullGame1RedRunsAloneWithTwoForfeitedSixes() {
        LudoGame g = game(2, GAME_1);

        List<String> specials = playOut(g, "lead", "first");

        assertThat(specials).isEmpty();
        assertFinished(g, "RED");
        assertProgress(g, "RED", 57, 57, 57, 57);
        assertProgress(g, "GREEN", 0, 0, 0, 0);
    }

    @Test
    void shouldPlayFullGame2WithTwoCapturesAndASafeLanding() {
        LudoGame g = game(2, GAME_2);

        List<String> specials = playOut(g, "lead", "first");

        assertThat(specials).containsExactly("captured:GREEN:1", "captured:GREEN:1");
        assertFinished(g, "RED");
        assertProgress(g, "RED", 57, 57, 57, 57);
        assertProgress(g, "GREEN", 46, 0, 0, 0);
    }

    @Test
    void shouldPlayFullGame3WithFourPlayers() {
        LudoGame g = game(4, GAME_3);

        List<String> specials = playOut(g, "lead", "first", "lead", "first");

        assertThat(specials).containsExactly("captured:RED:1", "captured:GREEN:1", "captured:BLUE:1");
        assertFinished(g, "YELLOW");
        assertProgress(g, "YELLOW", 57, 57, 57, 57);
        assertProgress(g, "RED", 44, 0, 0, 0);
        assertProgress(g, "GREEN", 17, 0, 0, 0);
        assertProgress(g, "BLUE", 57, 0, 0, 0);
    }
}
