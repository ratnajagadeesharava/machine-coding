package contract;

import java.util.List;
import java.util.Optional;

/** Provided mediator interface: the only thing aircraft know about the rest of the airspace. */
public interface Tower {

    /**
     * Asks for the runway. Free -> the aircraft is cleared to land at once; otherwise it joins the end of the
     * queue and is told its position. Null, or an aircraft already on the runway or in the queue ->
     * IllegalArgumentException.
     */
    void requestLanding(Aircraft aircraft);

    /**
     * The aircraft on the runway has landed: the runway becomes free. Anyone else (or nobody on the runway) ->
     * IllegalStateException.
     */
    void landed(Aircraft aircraft);

    /**
     * The aircraft diverts and no longer wants to land. Works whether it is on the runway (like {@link #landed}) or
     * waiting in the queue. Returns false if the tower does not know the aircraft.
     */
    boolean cancel(Aircraft aircraft);

    /**
     * Moves a waiting aircraft to the front of the queue. Unknown aircraft, or one already cleared, ->
     * IllegalArgumentException. Already first in the queue -> nothing changes.
     */
    void declareEmergency(Aircraft aircraft);

    /** Id of the aircraft currently cleared to use the runway. */
    Optional<String> onRunway();

    /** Ids of the waiting aircraft in landing order. */
    List<String> queue();
}
