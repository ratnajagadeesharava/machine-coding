package solution;

import java.time.Clock;
import java.util.List;
import java.util.Optional;

/**
 * A support ticket whose behaviour depends on its current state. Design it with the State pattern: every state
 * (NEW, IN_PROGRESS, ON_HOLD, RESOLVED, CLOSED) is its own class in this package that knows which operations it
 * allows and what the next state is; the Ticket just delegates to its current state object. Put each state in its own
 * source file (the tests count the files and scan for state-switching code).
 *
 * <pre>
 *            assign       hold          resume        resolve        reopen             close          comment
 * NEW         -> IN_PROGRESS  illegal      illegal       illegal        illegal            illegal        ok
 * IN_PROGRESS reassign        -> ON_HOLD   illegal       -> RESOLVED    illegal            illegal        ok
 * ON_HOLD     reassign        illegal      -> IN_PROGRESS illegal       illegal            illegal        ok
 * RESOLVED    illegal         illegal      illegal       illegal        -> IN_PROGRESS *   -> CLOSED      ok
 * CLOSED      illegal         illegal      illegal       illegal        illegal            illegal        illegal
 * </pre>
 * (* a ticket can be reopened at most twice; the third reopen is an IllegalStateException.)
 *
 * <p>Order of checks for every operation: invalid ARGUMENT (blank text) -> IllegalArgumentException first, then
 * an operation not allowed in the current state -> IllegalStateException with the message
 * {@code "cannot <operation> a ticket in state <STATE>"} (e.g. {@code "cannot resolve a ticket in state NEW"}).
 */
public class Ticket {

    public Ticket(String id, Clock clock) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String id() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** "NEW", "IN_PROGRESS", "ON_HOLD", "RESOLVED" or "CLOSED". */
    public String state() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Optional<String> assignedTo() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** NEW -> IN_PROGRESS and records the agent; in IN_PROGRESS or ON_HOLD it only changes the agent. Blank agent -> IllegalArgumentException. */
    public void assign(String agent) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** IN_PROGRESS -> ON_HOLD; adds the comment {@code "HOLD: <reason>"}. Blank reason -> IllegalArgumentException. */
    public void hold(String reason) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** ON_HOLD -> IN_PROGRESS. */
    public void resume() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** IN_PROGRESS -> RESOLVED; adds the comment {@code "RESOLVED: <note>"} and remembers when. Blank note -> IllegalArgumentException. */
    public void resolve(String note) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** RESOLVED -> IN_PROGRESS; adds the comment {@code "REOPENED"}. At most twice per ticket. */
    public void reopen() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** RESOLVED -> CLOSED. */
    public void close() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Adds the text as a comment in every state except CLOSED. Blank text -> IllegalArgumentException. */
    public void comment(String text) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Comments in the order they were added. Callers cannot modify the list. */
    public List<String> comments() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * A RESOLVED ticket that has been resolved for 72 hours or more (by the clock given at construction) becomes
     * CLOSED and this returns true. In any other situation it changes nothing and returns false.
     */
    public boolean autoCloseIfStale() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
