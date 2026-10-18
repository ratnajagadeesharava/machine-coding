package contract;

/** Provided command interface: a request turned into an object, so it can be queued, logged and undone. */
public interface Command {

    /** Does the work. May throw (e.g. IllegalArgumentException) - then it must have changed nothing. */
    void execute();

    /** Reverses the most recent successful {@link #execute()}. */
    void undo();
}
