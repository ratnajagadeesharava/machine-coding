package solution;

import contract.Scanner;
import java.util.List;

/** Needs a scanner. It must ask for a {@link Scanner}, not for something bigger. */
public final class ScanJob {
    private ScanJob() {}

    /** Scans {@code pages} pages in order. Null scanner or pages <= 0 -> IllegalArgumentException. */
    public static List<String> run(Scanner scanner, int pages) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
