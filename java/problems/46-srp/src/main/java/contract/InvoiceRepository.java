package contract;

import java.util.Optional;

/** Provided contract. */
public interface InvoiceRepository {

    void save(String invoiceId, String text);

    Optional<String> find(String invoiceId);
}
