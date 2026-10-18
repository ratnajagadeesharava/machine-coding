package contract;

import java.util.List;
import java.util.OptionalLong;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface WalletApi {

    /** Creates an empty wallet. False if the id is blank or already exists. */
    boolean createWallet(String walletId);

    /** APPLIED, DUPLICATE, INVALID_AMOUNT or UNKNOWN_WALLET. */
    String credit(String key, String walletId, long amount);

    /** APPLIED, DUPLICATE, INVALID_AMOUNT, UNKNOWN_WALLET or INSUFFICIENT_FUNDS. */
    String debit(String key, String walletId, long amount);

    /** APPLIED, DUPLICATE, INVALID_AMOUNT, UNKNOWN_WALLET or INSUFFICIENT_FUNDS. */
    String transfer(String key, String fromWalletId, String toWalletId, long amount);

    OptionalLong balance(String walletId);

    /** Signed entries such as "+100" and "-30", oldest first. Empty for unknown wallets. */
    List<String> statement(String walletId);
}
