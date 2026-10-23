package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface QueryPipeline {

    /** Names of the built-in steps, in their fixed default order. */
    String PARSE = "parse";
    String SPELL = "spell";
    String SYNONYMS = "synonyms";
    String FILTER = "filter";
    String RANK = "rank";
    String HIGHLIGHT = "highlight";

    /**
     * A query-transforming step supplied by the caller (the tests implement it). {@code name()} is constant for the
     * life of the step. {@code transform} receives the current query text and returns the new one. Throwing, or
     * returning null, means the step failed.
     */
    interface Step {
        String name();

        String transform(String query);
    }

    /**
     * What one {@link #process(String)} call produced.
     *
     * @param query   the query text after the last query-transforming step that ran ("" when it became empty)
     * @param results the ranked, highlighted result lines, best first; never null, empty when nothing matched
     * @param trace   names of the steps that changed something, in execution order; a failed step appears as
     *                {@code "<name>:failed"}
     */
    record Result(String query, List<String> results, List<String> trace) {}

    /**
     * Runs the pipeline on a raw query. The query-transforming steps are parse, spell, synonyms and filter (plus
     * any registered steps) in their current order, then rank, then highlight. IllegalArgumentException when
     * {@code rawQuery} is null or blank (nothing is run). Each call is independent; the returned lists are never
     * affected by later calls.
     */
    Result process(String rawQuery);

    /**
     * Adds a step at the end of the query-transforming phase (after "filter" and after earlier registered steps,
     * before rank). IllegalArgumentException when the step is null, its name is null/blank, or a step with the
     * same name exists (built-in names are always taken, even after the built-in was removed). Nothing changes on
     * error.
     */
    void register(Step step);

    /**
     * Adds a step immediately before the named query-transforming step (parse, spell, synonyms, filter or a
     * registered step). IllegalArgumentException for the same reasons as {@link #register(Step)}, or when the
     * anchor is unknown, removed, or not a query-transforming step ("rank", "highlight"). Nothing changes on error.
     */
    void registerBefore(String stepName, Step step);

    /** Like {@link #registerBefore(String, Step)} but immediately after the named step. */
    void registerAfter(String stepName, Step step);

    /**
     * Permanently removes a step (built-in or registered); it no longer runs. Removing "rank" leaves results empty;
     * removing "highlight" leaves the result lines unhighlighted. IllegalArgumentException for an unknown name.
     */
    void removeStep(String name);

    /** Keeps the step but skips it (not run, not in the trace) until enabled. IllegalArgumentException for an unknown name. */
    void disableStep(String name);

    /** Undoes {@link #disableStep(String)}; a no-op for a step that is already enabled. IllegalArgumentException for an unknown name. */
    void enableStep(String name);
}
