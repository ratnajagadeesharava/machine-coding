package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.InvoiceLine;
import contract.InvoiceRenderer;
import contract.InvoiceRepository;
import contract.InvoiceTotals;
import contract.TotalsCalculator;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.junit.jupiter.api.Test;
import solution.InvoiceService;
import solution.StandardCalculator;
import solution.TextRenderer;

class BehaviorTest {

    private static final List<InvoiceLine> LINES =
            List.of(new InvoiceLine("Widget", 2, 500), new InvoiceLine("Gadget", 1, 250));

    private final List<String> events = new ArrayList<>();

    private class FakeCalculator implements TotalsCalculator {
        final InvoiceTotals canned = new InvoiceTotals(111, 22, 133);
        boolean fail;

        public InvoiceTotals calculate(List<InvoiceLine> lines, int taxBps) {
            events.add("calculate:" + lines.size() + ":" + taxBps);
            if (fail) throw new IllegalArgumentException("bad lines");
            return canned;
        }
    }

    private class FakeRenderer implements InvoiceRenderer {
        InvoiceTotals seen;

        public String render(String invoiceId, List<InvoiceLine> lines, InvoiceTotals totals) {
            events.add("render:" + invoiceId);
            seen = totals;
            return "RENDERED";
        }
    }

    private class MemoryRepository implements InvoiceRepository {
        final Map<String, String> data = new HashMap<>();

        public void save(String invoiceId, String text) {
            events.add("save:" + invoiceId + ":" + text);
            data.put(invoiceId, text);
        }

        public Optional<String> find(String invoiceId) {
            return Optional.ofNullable(data.get(invoiceId));
        }
    }

    @Test
    void shouldCalculateTotalsWithoutKnowingAboutLayoutOrStorage() {
        InvoiceTotals totals = new StandardCalculator().calculate(LINES, 1800);

        assertThat(totals).isEqualTo(new InvoiceTotals(1250, 225, 1475));
        assertThat(new StandardCalculator().calculate(List.of(new InvoiceLine("x", 1, 1005)), 1800).tax()).isEqualTo(181);
    }

    @Test
    void shouldRejectInvalidInputInTheCalculator() {
        StandardCalculator calc = new StandardCalculator();

        assertThatThrownBy(() -> calc.calculate(List.of(), 1800)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> calc.calculate(List.of(new InvoiceLine("x", 0, 10)), 1800))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> calc.calculate(List.of(new InvoiceLine("x", 1, -10)), 1800))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> calc.calculate(LINES, -1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldLayOutAnInvoiceFromTotalsItIsGiven() {
        String text = new TextRenderer().render("A-1", LINES, new InvoiceTotals(1250, 225, 1475));

        assertThat(text).isEqualTo(String.join("\n",
                "INVOICE A-1",
                "Widget x2 @ 5.00 = 10.00",
                "Gadget x1 @ 2.50 = 2.50",
                "SUBTOTAL 12.50",
                "TAX 2.25",
                "TOTAL 14.75"));
    }

    @Test
    void shouldPrintTheGivenTotalsEvenWhenTheyDoNotMatchTheLines() {
        // A renderer that adds things up itself would print 12.50 here.
        String text = new TextRenderer().render("A-2", LINES, new InvoiceTotals(999, 1, 1000));

        assertThat(text).contains("SUBTOTAL 9.99").contains("TAX 0.01").contains("TOTAL 10.00");
    }

    @Test
    void shouldOnlyCoordinateWhenIssuingAnInvoice() {
        FakeCalculator calc = new FakeCalculator();
        FakeRenderer renderer = new FakeRenderer();
        MemoryRepository repo = new MemoryRepository();
        InvoiceService service = new InvoiceService(calc, renderer, repo);

        String result = service.issue("A-1", LINES, 1800);

        assertThat(result).isEqualTo("RENDERED");
        assertThat(renderer.seen).isEqualTo(calc.canned);
        assertThat(events).containsExactly("calculate:2:1800", "render:A-1", "save:A-1:RENDERED");
        assertThat(service.lookup("A-1")).hasValue("RENDERED");
        assertThat(service.lookup("nope")).isEmpty();
    }

    @Test
    void shouldTouchNeitherRendererNorRepositoryWhenTheCalculationFails() {
        FakeCalculator calc = new FakeCalculator();
        calc.fail = true;
        InvoiceService service = new InvoiceService(calc, new FakeRenderer(), new MemoryRepository());

        assertThatThrownBy(() -> service.issue("A-1", LINES, 1800)).isInstanceOf(IllegalArgumentException.class);

        assertThat(events).containsExactly("calculate:2:1800");
    }

    @Test
    void shouldRefuseABlankIdBeforeCallingAnyone() {
        InvoiceService service = new InvoiceService(new FakeCalculator(), new FakeRenderer(), new MemoryRepository());

        assertThatThrownBy(() -> service.issue(" ", LINES, 1800)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> service.issue(null, LINES, 1800)).isInstanceOf(IllegalArgumentException.class);

        assertThat(events).isEmpty();
    }

    @Test
    void shouldWorkEndToEndWithTheRealParts() {
        MemoryRepository repo = new MemoryRepository();
        InvoiceService service = new InvoiceService(new StandardCalculator(), new TextRenderer(), repo);

        String text = service.issue("A-1", LINES, 1800);

        assertThat(text).endsWith("SUBTOTAL 12.50\nTAX 2.25\nTOTAL 14.75");
        assertThat(repo.data).containsEntry("A-1", text);
    }
}
