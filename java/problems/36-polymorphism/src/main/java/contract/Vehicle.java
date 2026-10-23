package contract;

/** Provided contract. */
public interface Vehicle {

    String plate();

    /** Toll in cents. {@code peak} is true during rush hour. */
    long toll(boolean peak);
}
