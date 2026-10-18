package solution;

import contract.Aircraft;
import contract.Tower;
import java.util.List;
import java.util.Optional;

/**
 * The mediator. All coordination lives here: aircraft never talk to each other.
 *
 * <p>Notification rules (the tests check the exact calls):
 * <ul>
 *   <li>runway free on request: {@code onClearedToLand()}; otherwise the aircraft joins the queue and gets
 *       {@code onHold(position)};</li>
 *   <li>when the runway is freed ({@code landed}, or {@code cancel} of the aircraft on the runway) and somebody is
 *       waiting: the first in the queue moves to the runway and gets {@code onClearedToLand()}, THEN every aircraft
 *       still waiting gets {@code onHold(newPosition)};</li>
 *   <li>cancelling a waiting aircraft removes it and tells each aircraft BEHIND it its new position;</li>
 *   <li>an emergency moves the aircraft to the front: first it gets {@code onHold(1)}, then each aircraft it
 *       overtook gets {@code onHold(newPosition)}, in queue order.</li>
 * </ul>
 */
public class ControlTower implements Tower {

    @Override
    public void requestLanding(Aircraft aircraft) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void landed(Aircraft aircraft) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public boolean cancel(Aircraft aircraft) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void declareEmergency(Aircraft aircraft) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public Optional<String> onRunway() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public List<String> queue() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
