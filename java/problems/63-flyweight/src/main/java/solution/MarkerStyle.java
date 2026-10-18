package solution;

/**
 * The flyweight: the heavy, SHARED part of a map marker (its icon). Intrinsic state only: shape, colour and a
 * 4096-byte bitmap that is allocated once per style. It has no public constructor: only {@link StyleFactory}
 * creates styles, so they can be shared safely.
 */
public final class MarkerStyle {

    /** Package-private on purpose. Allocates the 4096-byte bitmap. */
    MarkerStyle(String shape, String color) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String shape() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String color() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Size of the icon bitmap in bytes (always 4096). */
    public int bitmapSize() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
