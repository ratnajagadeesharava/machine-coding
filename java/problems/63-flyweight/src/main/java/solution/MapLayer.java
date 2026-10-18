package solution;

import java.util.List;

/** A layer of markers. Styles always come from the factory it was given. */
public class MapLayer {

    public MapLayer(StyleFactory factory) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Creates a marker whose style is {@code factory.get(shape, color)}. Invalid arguments -> IllegalArgumentException. */
    public void addMarker(double lat, double lng, String label, String shape, String color) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int markerCount() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Markers in insertion order. Callers cannot modify the layer through the returned list. */
    public List<Marker> markers() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** One line per marker, in order: {@code "<shape>/<color> @ <lat>,<lng> <label>"}, e.g. {@code "pin/red @ 12.5,-3.25 Cafe"}. */
    public List<String> render() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
