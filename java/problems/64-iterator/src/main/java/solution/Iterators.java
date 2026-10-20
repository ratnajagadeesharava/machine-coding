package solution;

import java.util.Iterator;
import java.util.function.Predicate;

public final class Iterators {
    private Iterators() {}

    /**
     * A lazy filtering iterator: yields only elements accepted by {@code keep}, never pulling more from the source
     * than needed. Null source or predicate -> IllegalArgumentException.
     */
    public static <T> Iterator<T> filter(Iterator<T> source, Predicate<? super T> keep) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
