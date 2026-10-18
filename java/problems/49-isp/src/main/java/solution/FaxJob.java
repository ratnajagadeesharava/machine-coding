package solution;

import contract.FaxMachine;

/** Needs a fax machine. It must ask for a {@link FaxMachine}, not for something bigger. */
public final class FaxJob {
    private FaxJob() {}

    /** Sends one fax and reports whether it went out. Null machine -> IllegalArgumentException. */
    public static boolean send(FaxMachine machine, String number, String document) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
