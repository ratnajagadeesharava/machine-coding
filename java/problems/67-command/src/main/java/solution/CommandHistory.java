package solution;

import contract.Command;

/** Runs commands with undo / redo support. */
public class CommandHistory {

    /** Runs the command. */
    public void execute(Command command) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Returns false if there is nothing to undo. */
    public boolean undo() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Returns false if there is nothing to redo. */
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
