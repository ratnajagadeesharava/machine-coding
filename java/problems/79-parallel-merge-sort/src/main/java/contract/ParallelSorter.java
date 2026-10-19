package contract;

/** Thin test entry point. Your design lives behind it. May be called from several threads on different arrays. */
public interface ParallelSorter {

    /**
     * Sorts {@code data} in place, ascending, splitting the work over at most {@code parallelism} worker threads
     * (the caller may take part). {@code parallelism == 1} must not need any extra thread.
     * IllegalArgumentException if parallelism < 1. Any length (including 0 and 1) and any int values are valid.
     */
    void sort(int[] data, int parallelism);
}
