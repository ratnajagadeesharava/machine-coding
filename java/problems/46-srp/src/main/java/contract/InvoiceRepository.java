package contract;

import java.util.Optional;

/** Provided contract: knows storage, nothing about maths or layout. */
public interface InvoiceRepository {

    void save(String invoiceId, String text);

    Optional<String> find(String invoiceId);
}
