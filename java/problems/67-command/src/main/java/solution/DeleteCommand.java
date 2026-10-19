package solution;

import contract.Command;
import contract.TextBuffer;

/** Deletes a range of text. */
public class DeleteCommand implements Command {

    public DeleteCommand(TextBuffer buffer, int from, int to) {
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
