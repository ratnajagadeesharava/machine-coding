package contract;

/** Provided contract: something that can send faxes. Nothing more. */
public interface FaxMachine {

    /** @return true if the fax was sent. */
    boolean fax(String number, String document);
}
