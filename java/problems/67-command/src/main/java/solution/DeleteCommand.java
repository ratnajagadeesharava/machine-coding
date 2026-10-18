package solution;

import contract.Command;
import contract.TextBuffer;

/**
 * Deletes [from, to). It remembers the text it removed so that undo can put it back at {@code from}.
 * Null buffer -> IllegalArgumentException. A bad range is reported by execute() (the buffer rejects it).
 */
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
