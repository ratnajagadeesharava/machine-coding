package solution;

import contract.FaxMachine;
import contract.Printer;
import contract.Scanner;

/** All-in-one office machine. */
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
