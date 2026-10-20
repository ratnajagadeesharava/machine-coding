package solution;

import contract.Command;
import java.util.List;

/** Several commands as one. */
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
