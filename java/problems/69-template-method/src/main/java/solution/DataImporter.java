package solution;

import contract.ImportReport;
import java.util.Optional;

/**
 * The template. {@link #run()} is the fixed algorithm; subclasses fill in the steps.
 *
 * <pre>
 * open();                                   // if this throws, run() throws and close() is NOT called
 * try {
 *   while (readRecord() is present) {       // an exception here propagates (after close())
 *     read++
 *     if (!validate(record)) { skipped++; continue; }
 *     try { saved record = transform(record); save(...); saved++ }
 *     catch (RuntimeException) { failed++ }   // transform OR save failed: count it and carry on
 *   }
 * } finally { close(); }
 * return new ImportReport(read, saved, skipped, failed);
 * </pre>
 */
public abstract class DataImporter {

    /** The algorithm. It must be final: subclasses change the steps, never the order. */
    public final ImportReport run() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    // ---- steps every importer must provide ----

    /** Prepare the source (open a file, connect...). */
    protected abstract void open();

    /** The next raw record, or empty when the source is exhausted. */
    protected abstract Optional<String> readRecord();

    /** Store one transformed record. May throw a RuntimeException. */
    protected abstract void save(String record);

    // ---- steps with a sensible default (hooks) ----

    /** Default: a record is valid when it is not blank. */
    protected boolean validate(String record) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Default: the record unchanged. May throw a RuntimeException. */
    protected String transform(String record) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Default: does nothing. */
    protected void close() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
