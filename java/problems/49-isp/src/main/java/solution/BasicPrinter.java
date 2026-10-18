package solution;

import contract.Printer;

/** A printer and nothing else: it implements ONLY {@link Printer}. */
public class BasicPrinter implements Printer {

    /** Blank document -> IllegalArgumentException. Otherwise returns "PRINTED: " + document. */
    @Override
    public String print(String document) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
