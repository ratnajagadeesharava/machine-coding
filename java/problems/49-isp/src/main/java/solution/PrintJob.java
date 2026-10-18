package solution;

import contract.Printer;
import java.util.List;

/** Needs a printer. It must ask for a {@link Printer}, not for something bigger. */
public final class PrintJob {
    private PrintJob() {}

    /** Prints each document in order and returns the outputs. Null printer -> IllegalArgumentException; null docs -> empty list. */
    public static List<String> run(Printer printer, List<String> documents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
