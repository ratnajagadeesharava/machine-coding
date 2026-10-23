package solution;

/** Undo/redo for one character. */
public class UndoHistory {

    /** Null character or {@code maxCheckpoints < 1} -> IllegalArgumentException. */
    public UndoHistory(GameCharacter character, int maxCheckpoints) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Remembers the current state. */
    public void checkpoint() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Goes back one checkpoint. */
    public boolean undo() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Re-applies the latest undone state. */
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
