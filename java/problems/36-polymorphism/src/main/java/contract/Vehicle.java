package contract;

/** Provided contract. Every vehicle knows its own toll. */
public interface Vehicle {

    String plate();

    /** Toll in cents. {@code peak} is true during rush hour. */
    long toll(boolean peak);
}
