package solution;

import java.util.List;

/**
 * The originator. It can save its state into an opaque {@link Snapshot} (the memento) and restore from one,
 * without ever exposing its internals. All fields are private; no setters.
 *
 * <p>Initial state: hp 100, level 1, position (0, 0), empty inventory.
 */
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

    /** Items in pick-up order. Callers cannot modify the character through the returned list. */
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

    /**
     * Captures hp, level, position and inventory. The snapshot must be a DEEP, independent copy: later changes to the
     * character never alter it.
     */
    public Snapshot save() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Puts the character back into the saved state (the name is not part of the state). Null -> IllegalArgumentException. */
    public void restore(Snapshot snapshot) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * The memento: opaque to everybody except {@link GameCharacter}. It must have NO public methods, NO public
     * fields and NO public constructor - nobody else can read or forge a saved state. Add private fields to hold
     * the state; the enclosing class can read them.
     */
    public static final class Snapshot {
        private Snapshot() { }
    }
}
