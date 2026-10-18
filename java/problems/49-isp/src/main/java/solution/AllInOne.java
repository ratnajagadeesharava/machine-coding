package solution;

import contract.FaxMachine;
import contract.Printer;
import contract.Scanner;

/**
 * Does everything. It implements the three small interfaces directly: no combined "Machine" interface.
 * print: "PRINTED: " + document (blank -> IllegalArgumentException). scan: "SCAN-1", "SCAN-2", ...
 * fax: a blank document -> IllegalArgumentException; the number must look like an optional '+' followed by
 * 7 to 15 digits, otherwise returns false; a good number returns true.
 */
public class AllInOne implements Printer, Scanner, FaxMachine {

    @Override
    public String print(String document) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String scan() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public boolean fax(String number, String document) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
