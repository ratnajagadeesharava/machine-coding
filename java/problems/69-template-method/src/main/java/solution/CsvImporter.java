package solution;

import java.util.List;
import java.util.Optional;

/**
 * Imports CSV lines "name,email,city" into a sink list. Null lines or sink -> IllegalArgumentException.
 */
public class CsvImporter extends DataImporter {

    public CsvImporter(List<String> lines, List<String> sink) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    protected void open() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    protected Optional<String> readRecord() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    protected void save(String record) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    protected boolean validate(String record) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    protected String transform(String record) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
