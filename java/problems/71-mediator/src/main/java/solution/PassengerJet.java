package solution;

import contract.Aircraft;
import contract.Tower;

/** A passenger aircraft. */
public class PassengerJet implements Aircraft {

    /** Blank id or null tower -> IllegalArgumentException. Starts as "AIRBORNE". */
    public PassengerJet(String id, Tower tower) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String id() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Asks the tower for the runway. */
    public void requestLanding() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Tells the tower it has landed; status becomes "LANDED". */
    public void touchdown() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Tells the tower it is diverting; status becomes "DIVERTED". */
    public void divert() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** "AIRBORNE", "HOLDING #<n>", "CLEARED", "LANDED" or "DIVERTED". */
    public String status() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void onClearedToLand() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void onHold(int position) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
