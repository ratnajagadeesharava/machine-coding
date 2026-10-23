package contract;

/**
 * Turns text into a vector. Implemented by the tests (deterministic fakes); your component only calls it.
 * Every vector from one embedder has the same length. Implementations may throw a RuntimeException.
 */
public interface Embedder {

    double[] embed(String text);
}
