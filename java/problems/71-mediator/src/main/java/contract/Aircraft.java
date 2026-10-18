package contract;

/** Provided colleague interface: an aircraft that is told what to do by the tower. */
public interface Aircraft {

    /** Unique call sign. Two aircraft with the same id are the same aircraft. */
    String id();

    /** The runway is yours: land now. */
    void onClearedToLand();

    /** Hold and wait. {@code position} is your place in the landing queue (1 = next to land). */
    void onHold(int position);
}
