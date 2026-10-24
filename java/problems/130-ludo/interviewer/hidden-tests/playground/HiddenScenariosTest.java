package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.LudoGame;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private static LudoGame game(int players, List<Integer> dice) {
        return Candidate.of(() -> Solution.create(players, dice));
    }

    /** Plays {@code rolls} rolls; whenever a move exists the next pick is played. Returns the last move description. */
    private static String replay(LudoGame g, int rolls, int... picks) {
        String last = null;
        int next = 0;
        for (int i = 0; i < rolls; i++) {
            g.roll();
            if (!g.legalMoves().isEmpty()) {
                last = g.move(picks[next++]);
            }
        }
        return last;
    }

    /** Deterministic pseudo-random dice: a 64-bit linear congruential generator, value = ((x >>> 33) % 6) + 1. */
    private static List<Integer> lcgDice(long seed, int count) {
        List<Integer> out = new ArrayList<>();
        long x = seed;
        for (int i = 0; i < count; i++) {
            x = x * 6364136223846793005L + 1442695040888963407L;
            out.add((int) ((x >>> 33) % 6) + 1);
        }
        return out;
    }

    private static void assertProgress(LudoGame g, String colour, int... expected) {
        for (int t = 0; t < 4; t++) {
            assertThat(g.tokenProgress(colour, t)).as(colour + " token " + t).isEqualTo(expected[t]);
        }
    }

    private static int countCaptures(List<String> specials) {
        int n = 0;
        for (String s : specials) {
            n += s.split(",").length;
        }
        return n;
    }

    @Test
    void shouldCaptureTwoColoursOnTheSameCellInTurnOrder() {
        LudoGame g = game(3, BehaviorTest.dice("6 4 6 1 4 6 6 4 1 4 6 6 6 2 2 6 1 2 6 3 6 1 6 2 1 1 4 4 5 3 6"));

        String last = replay(g, 31, 3, 3, 2, 2, 3, 2, 3, 3, 3, 1, 3, 2, 2, 3, 0, 0, 3, 1, 3, 3, 0, 3, 2, 0, 2, 2, 0);

        assertThat(last).isEqualTo("captured:RED:1,captured:YELLOW:1");
        assertProgress(g, "GREEN", 15, 0, 0, 0);
        assertProgress(g, "RED", 0, 0, 8, 0);
        assertProgress(g, "YELLOW", 0, 0, 7, 8);
        assertThat(g.tokenCell("GREEN", 0)).isEqualTo(27);
        assertThat(g.currentPlayer()).isEqualTo("GREEN");
    }

    @Test
    void shouldKeepTheTurnWhenASixCannotMoveAnyToken() {
        LudoGame g = game(2, BehaviorTest.dice("6 5 1 3 1 3 1 6 6 6 1 6 6 5 1 6 6 3 1 6 6 4 1 6 5 1 2 1 5 1 6 6 4 1 6 6 6 1 6 5 1 6 5 1 6 3 1 4 1 6 2 1 6 6 3 1 3 1 6 2 1 3 1 3 1 6 6 3 1 6 5 1 6 6 5 1 6 1"));
        for (int i = 0; i < 76; i++) {
            g.roll();
            List<Integer> legal = g.legalMoves();
            if (!legal.isEmpty()) {
                int pick = legal.get(0);
                for (int t : legal) {
                    if (g.tokenProgress(g.currentPlayer(), t) > g.tokenProgress(g.currentPlayer(), pick)) {
                        pick = t;
                    }
                }
                g.move(pick);
            }
        }
        assertProgress(g, "RED", 56, 57, 56, 52);
        assertThat(g.currentPlayer()).isEqualTo("RED");

        assertThat(g.roll()).isEqualTo(6);
        assertThat(g.legalMoves()).isEmpty();
        assertThat(g.currentPlayer()).isEqualTo("RED");

        assertThat(g.roll()).isEqualTo(1);
        assertThat(g.legalMoves()).containsExactly(0, 2, 3);
    }

    @Test
    void shouldCountASixThatCapturedTowardsTheThreeSixLimit() {
        LudoGame g = game(2, BehaviorTest.dice("6 3 6 2 6 6 6 2"));
        String last = replay(g, 6, 1, 1, 3, 3, 1, 1);
        assertThat(last).isEqualTo("captured:GREEN:1");
        assertThat(g.currentPlayer()).isEqualTo("RED");

        assertThat(g.roll()).isEqualTo(6);

        assertThat(g.legalMoves()).isEmpty();
        assertProgress(g, "RED", 0, 16, 0, 0);
        assertThat(g.currentPlayer()).isEqualTo("GREEN");
        g.roll();
        assertThat(g.currentPlayer()).isEqualTo("RED");
    }

    @Test
    void shouldCaptureAfterABlueTokenWrapsPastCell51() {
        LudoGame g = game(4, BehaviorTest.dice("4 3 4 6 4 6 1 3 5 6 4"));

        String last = replay(g, 11, 3, 3, 2, 2, 3, 3);

        assertThat(last).isEqualTo("captured:RED:1");
        assertProgress(g, "BLUE", 0, 0, 0, 15);
        assertThat(g.tokenCell("BLUE", 3)).isEqualTo(1);
        assertProgress(g, "RED", 0, 0, 0, 0);
        assertThat(g.currentPlayer()).isEqualTo("BLUE");
    }

    @Test
    void shouldRejectEveryCallOnceTheGameIsWonEvenWithDiceLeft() {
        List<Integer> dice = new ArrayList<>(BehaviorTest.dice("3 1 6 4 1 6 4 1 6 6 3 1 4 1 5 1 6 6 2 1 6 6 5 1 6 5 1 2 1 6 6 5 1 6 6 3 1 6 4 1 6 3 1 2 2 6 2 1 6 4 1 6 6 4 1 3 1 4 1 6 6 3 1 6 2 6 6 6 1 5 1 6 6 6 1 6 6"));
        dice.addAll(List.of(6, 6, 6));
        LudoGame g = game(2, dice);

        BehaviorTest.playOut(g, "lead", "first");

        assertThat(g.isOver()).isTrue();
        assertThat(g.winner()).contains("RED");
        assertThatThrownBy(g::roll).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> g.move(0)).isInstanceOf(IllegalStateException.class);
        assertProgress(g, "RED", 57, 57, 57, 57);
    }

    @Test
    void shouldPlayRandomGame1(){
        LudoGame g = game(2, lcgDice(1L, 190));

        List<String> specials = BehaviorTest.playOut(g, "lead", "first");

        assertThat(g.isOver()).isTrue();
        assertThat(g.winner()).contains("RED");
        assertThat(countCaptures(specials)).isEqualTo(2);
        assertProgress(g, "RED", 57, 57, 57, 57);
        assertProgress(g, "GREEN", 57, 57, 57, 48);
    }

    @Test
    void shouldPlayRandomGame2(){
        LudoGame g = game(3, lcgDice(2L, 271));

        List<String> specials = BehaviorTest.playOut(g, "lead", "rear", "first");

        assertThat(g.isOver()).isTrue();
        assertThat(g.winner()).contains("RED");
        assertThat(countCaptures(specials)).isEqualTo(9);
        assertProgress(g, "RED", 57, 57, 57, 57);
        assertProgress(g, "GREEN", 0, 41, 0, 37);
        assertProgress(g, "YELLOW", 57, 57, 49, 0);
    }

    @Test
    void shouldPlayRandomGame3(){
        LudoGame g = game(4, lcgDice(3L, 380));

        List<String> specials = BehaviorTest.playOut(g, "rear", "lead", "first", "lead");

        assertThat(g.isOver()).isTrue();
        assertThat(g.winner()).contains("GREEN");
        assertThat(countCaptures(specials)).isEqualTo(14);
        assertProgress(g, "RED", 53, 0, 0, 56);
        assertProgress(g, "GREEN", 57, 57, 57, 57);
        assertProgress(g, "YELLOW", 57, 57, 0, 0);
        assertProgress(g, "BLUE", 57, 57, 57, 0);
    }

    @Test
    void shouldPlayRandomGame4(){
        LudoGame g = game(4, lcgDice(12345L, 356));

        List<String> specials = BehaviorTest.playOut(g, "first", "first", "first", "first");

        assertThat(g.isOver()).isTrue();
        assertThat(g.winner()).contains("RED");
        assertThat(countCaptures(specials)).isEqualTo(8);
        assertProgress(g, "RED", 57, 57, 57, 57);
        assertProgress(g, "GREEN", 57, 57, 57, 0);
        assertProgress(g, "YELLOW", 57, 57, 14, 0);
        assertProgress(g, "BLUE", 57, 57, 51, 0);
    }

    @Test
    void shouldPlayRandomGame5(){
        LudoGame g = game(2, lcgDice(777L, 172));

        List<String> specials = BehaviorTest.playOut(g, "rear", "rear");

        assertThat(g.isOver()).isTrue();
        assertThat(g.winner()).contains("GREEN");
        assertThat(countCaptures(specials)).isEqualTo(2);
        assertProgress(g, "RED", 54, 55, 54, 55);
        assertProgress(g, "GREEN", 57, 57, 57, 57);
    }
}
