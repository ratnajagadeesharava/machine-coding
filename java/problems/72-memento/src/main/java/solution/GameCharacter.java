package solution;

import java.util.List;

/** A game character. Initial state: hp 100, level 1, position (0, 0), empty inventory. */
public class GameCharacter {

    public static final int MAX_HP = 100;
    public static final int MAX_ITEMS = 3;

    /** Blank name -> IllegalArgumentException. */
    public GameCharacter(String name) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String name() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int hp() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int level() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int x() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int y() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Items in pick-up order. */
    public List<String> inventory() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Negative amount -> IllegalArgumentException. hp never goes below 0. */
    public void takeDamage(int amount) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Negative amount -> IllegalArgumentException. hp never goes above {@link #MAX_HP}. */
    public void heal(int amount) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public void levelUp() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public void moveTo(int x, int y) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Blank item -> IllegalArgumentException; already carrying {@link #MAX_ITEMS} -> IllegalStateException. */
    public void pickUp(String item) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Removes one copy of the item. Returns whether the character was carrying it. */
    public boolean drop(String item) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Captures the character's state. */
    public Snapshot save() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Restores a saved state. */
    public void restore(Snapshot snapshot) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** A saved character state. */
    public static final class Snapshot {
        private Snapshot() { }
    }
}
