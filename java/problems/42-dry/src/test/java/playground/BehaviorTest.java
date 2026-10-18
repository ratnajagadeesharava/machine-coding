package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.PricingRules;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Documents;
import solution.StandardRules;

class BehaviorTest {

    private static Documents standard() {
        return new Documents(new StandardRules(1800, 10_000, 1000));
    }

    /** Rules written in the test: every document must use whatever rules it is given. */
    private static PricingRules rules(long discount, long tax) {
        return new PricingRules() {
            public long discount(long net) { return discount; }
            public long tax(long taxable) { return tax; }
        };
    }

    @Test
    void shouldFormatAQuote() {
        assertThat(standard().quote(List.of(5000L))).isEqualTo("QUOTE net=50.00 discount=0.00 tax=9.00 total=59.00");
    }

    @Test
    void shouldFormatAnInvoiceWithTheBulkDiscount() {
        assertThat(standard().invoice("INV-1", List.of(12_000L, 8_000L)))
                .isEqualTo("INVOICE #INV-1 net=200.00 discount=20.00 tax=32.40 total=212.40");
    }

    @Test
    void shouldFormatACreditNoteWithANegativeTotal() {
        assertThat(standard().creditNote("INV-9", List.of(5000L)))
                .isEqualTo("CREDIT NOTE #INV-9 net=50.00 discount=0.00 tax=9.00 total=-59.00");
    }

    @Test
    void shouldRoundTaxHalfUpToWholeCents() {
        assertThat(standard().quote(List.of(1005L))).isEqualTo("QUOTE net=10.05 discount=0.00 tax=1.81 total=11.86");
        assertThat(standard().quote(List.of(5L))).isEqualTo("QUOTE net=0.05 discount=0.00 tax=0.01 total=0.06");
    }

    @Test
    void shouldApplyTheDiscountOnlyFromTheThreshold() {
        assertThat(standard().quote(List.of(9_999L))).contains("discount=0.00");
        assertThat(standard().quote(List.of(10_000L))).contains("discount=10.00");
    }

    @Test
    void shouldUseTheSameInjectedRulesInEveryDocument() {
        Documents docs = new Documents(rules(100, 7));

        assertThat(docs.quote(List.of(1000L))).isEqualTo("QUOTE net=10.00 discount=1.00 tax=0.07 total=9.07");
        assertThat(docs.invoice("A", List.of(1000L))).isEqualTo("INVOICE #A net=10.00 discount=1.00 tax=0.07 total=9.07");
        assertThat(docs.creditNote("A", List.of(1000L)))
                .isEqualTo("CREDIT NOTE #A net=10.00 discount=1.00 tax=0.07 total=-9.07");
    }

    @Test
    void shouldChangeEveryDocumentWhenOnlyTheRateChanges() {
        Documents low = new Documents(new StandardRules(1000, 10_000, 0));
        Documents high = new Documents(new StandardRules(5000, 10_000, 0));

        assertThat(low.quote(List.of(1000L))).contains("tax=1.00");
        assertThat(low.invoice("X", List.of(1000L))).contains("tax=1.00");
        assertThat(low.creditNote("X", List.of(1000L))).contains("tax=1.00");
        assertThat(high.quote(List.of(1000L))).contains("tax=5.00");
        assertThat(high.invoice("X", List.of(1000L))).contains("tax=5.00");
        assertThat(high.creditNote("X", List.of(1000L))).contains("tax=5.00");
    }

    @Test
    void shouldValidateInputTheSameWayEverywhere() {
        Documents docs = standard();

        assertThatThrownBy(() -> docs.quote(List.of())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> docs.quote(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> docs.invoice("A", List.of(100L, -1L))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> docs.creditNote("A", List.of(-5L))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> docs.invoice(" ", List.of(100L))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> docs.creditNote("", List.of(100L))).isInstanceOf(IllegalArgumentException.class);
        assertThat(docs.quote(List.of(0L))).isEqualTo("QUOTE net=0.00 discount=0.00 tax=0.00 total=0.00");
    }
}
