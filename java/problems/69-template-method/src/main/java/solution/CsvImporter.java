package solution;

import java.util.List;
import java.util.Optional;

/**
 * Imports CSV lines "name,email,city". The first line is a header and is skipped. A record is valid when it has
 * exactly 3 comma-separated fields (empty trailing fields count) and none of them is blank after trimming. Valid
 * records are transformed by trimming every field and joining them with "|" ("ann , a@x.io, Rome" -> "ann|a@x.io|Rome"),
 * and saved by appending to the sink list. Null lines or sink -> IllegalArgumentException.
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
