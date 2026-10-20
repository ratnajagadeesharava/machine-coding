package solution;

import contract.ImportReport;
import java.util.Optional;

/** Base class for data importers. */
public abstract class DataImporter {

    /** Runs one import and reports the counts. */
    public final ImportReport run() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Prepare the source. */
    protected abstract void open();

    /** The next raw record, or empty when the source is exhausted. */
    protected abstract Optional<String> readRecord();

    /** Store one record. */
    protected abstract void save(String record);

    /** Default: a record is valid when it is not blank. */
    protected boolean validate(String record) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Default: the record unchanged. */
    protected String transform(String record) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Default: does nothing. */
    protected void close() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
