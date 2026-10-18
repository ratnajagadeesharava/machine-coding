package solution;

/** The caretaker for undo/redo of one character. It stores snapshots only. */
public class UndoHistory {

    /** Null character or {@code maxCheckpoints < 1} -> IllegalArgumentException. */
    public UndoHistory(GameCharacter character, int maxCheckpoints) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Remembers the character's CURRENT state as something to go back to, and forgets everything that could be
     * redone. When more than {@code maxCheckpoints} are remembered the oldest is dropped.
     */
    public void checkpoint() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Goes back to the newest checkpoint (which is consumed). The state the character had just before going back is
     * kept so that {@link #redo()} can return to it. False (doing nothing) when there is no checkpoint.
     */
    public boolean undo() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Returns to the state left by the latest undo; that undo's target becomes undoable again. False if nothing to redo. */
    public boolean redo() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int undoDepth() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int redoDepth() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
