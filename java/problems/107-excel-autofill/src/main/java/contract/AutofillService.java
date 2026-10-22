package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be safe to call from many threads at once. */
public interface AutofillService {

    /**
     * Recognises the series formed by {@code seeds} (cell texts, top to bottom) and returns the next {@code count}
     * values of that series, in order, as strings. The seeds themselves are not part of the result.
     *
     * <p>Numeric series: seeds are plain decimal numbers (optional '-', digits, optional '.digits', at most 15 digits).
     * Letter series: seeds are 1 to 10 ASCII letters, all of one case, ordered like spreadsheet column names
     * (A..Z, AA..AZ, BA..). Both are arithmetic: the step is the difference between the first two seeds, every
     * consecutive pair must have that step, and a single seed has step 1.
     *
     * <p>A letter series that decreases stops before "A"/"a", so the result can be shorter than {@code count}.
     * {@code count == 0} returns an empty list.
     *
     * @throws IllegalArgumentException if {@code seeds} is null, empty or contains null, if {@code count} is negative,
     *         or if the seeds are not one recognised series (mixed kinds, not arithmetic, malformed, mixed case).
     *         Seeds are validated even when {@code count} is 0.
     */
    List<String> autofill(List<String> seeds, int count);
}
