package solution;

import contract.Command;
import contract.TextBuffer;

/** Inserts text into a buffer. */
public class InsertCommand implements Command {

    public InsertCommand(TextBuffer buffer, int pos, String text) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void execute() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void undo() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
