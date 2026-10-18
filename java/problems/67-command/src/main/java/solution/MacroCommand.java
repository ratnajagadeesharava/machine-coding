package solution;

import contract.Command;
import java.util.List;

/**
 * Several commands as one. execute() runs them in order; if one throws, the ones already run are undone in reverse
 * order (so the macro is all-or-nothing) and the exception is rethrown. undo() undoes them in reverse order.
 * Null or empty list -> IllegalArgumentException.
 */
public class MacroCommand implements Command {

    public MacroCommand(List<Command> commands) {
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
