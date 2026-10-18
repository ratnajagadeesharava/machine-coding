package solution;

/**
 * One pin on the map. It holds only EXTRINSIC (per-pin) state - position and label - plus a reference to its
 * shared style. It must never hold a bitmap of its own.
 */
public class Marker {

    /** lat must be in -90..90 and lng in -180..180 (IllegalArgumentException otherwise); style must not be null. */
    public Marker(double lat, double lng, String label, MarkerStyle style) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public double lat() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public double lng() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String label() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public MarkerStyle style() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
