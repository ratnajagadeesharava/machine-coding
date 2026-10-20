package contract;

/** Provided command interface: an undoable edit. */
public interface Command {

    /** Does the work. May throw (e.g. IllegalArgumentException) - then it must have changed nothing. */
    void execute();

    /** Reverses the most recent successful {@link #execute()}. */
    void undo();
}
