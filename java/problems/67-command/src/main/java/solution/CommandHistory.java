package solution;

import contract.Command;

/** The invoker: runs commands and remembers them for undo / redo. */
public class CommandHistory {

    /**
     * Runs the command. If it succeeds it becomes the newest undoable command and the redo list is cleared. If it
     * throws, the exception propagates and the history is unchanged. Null -> IllegalArgumentException.
     */
    public void execute(Command command) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Undoes the newest command and makes it redoable. Returns false (doing nothing) if there is nothing to undo. */
    public boolean undo() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Executes the newest undone command again and makes it undoable. Returns false if there is nothing to redo. */
    public boolean redo() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public boolean canUndo() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public boolean canRedo() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
