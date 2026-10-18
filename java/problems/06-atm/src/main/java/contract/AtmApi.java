package contract;

import java.util.Map;
import java.util.Optional;
import java.util.OptionalLong;

/** Thin test entry point. Your domain model lives behind it. */
public interface AtmApi {

    void addAccount(String cardId, String pin, long balance);

    /** Adds notes of the given denomination (10, 20, 50 or 100) to the machine. */
    void loadCash(int denomination, int count);

    /** Returns a session token, or empty (unknown card, wrong PIN, locked card). */
    Optional<String> authenticate(String cardId, String pin);

    OptionalLong balance(String session);

    /** Notes dispensed (denomination to count), or empty if refused. */
    Optional<Map<Integer, Integer>> withdraw(String session, int amount);

    boolean deposit(String session, long amount);

    void endSession(String session);

    long cashInMachine();
}
