package solution;

/** The flyweight factory: hands out ONE shared {@link MarkerStyle} per distinct (shape, colour). Thread-safe. */
public class StyleFactory {

    /**
     * The shared style for this shape and colour (compared exactly, case-sensitive). Blank shape or colour ->
     * IllegalArgumentException. Many threads may ask at once; each distinct pair is still created only once.
     */
    public MarkerStyle get(String shape, String color) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** How many MarkerStyle objects this factory has created so far. */
    public int instancesCreated() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
