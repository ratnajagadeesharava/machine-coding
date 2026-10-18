package solution;

import java.util.Iterator;
import java.util.function.Predicate;

public final class Iterators {
    private Iterators() {}

    /**
     * A lazy view of {@code source} that only yields elements accepted by {@code keep}. It pulls from the source only
     * as far as needed (never ahead of the element it is about to return), hasNext() is idempotent, and next() at
     * the end throws NoSuchElementException. Null source or predicate -> IllegalArgumentException.
     */
    public static <T> Iterator<T> filter(Iterator<T> source, Predicate<? super T> keep) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
