package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.FaxMachine;
import contract.Printer;
import contract.Scanner;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.AllInOne;
import solution.BasicPrinter;
import solution.FaxJob;
import solution.FlatbedScanner;
import solution.PrintJob;
import solution.ScanJob;

class BehaviorTest {

    @Test
    void shouldPrintWithABasicPrinter() {
        Printer printer = new BasicPrinter();

        assertThat(printer.print("report")).isEqualTo("PRINTED: report");
        assertThatThrownBy(() -> printer.print(" ")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNumberScannedPagesFromOne() {
        Scanner scanner = new FlatbedScanner();

        assertThat(scanner.scan()).isEqualTo("SCAN-1");
        assertThat(scanner.scan()).isEqualTo("SCAN-2");
    }

    @Test
    void shouldLetAnAllInOneBeUsedAsEachOfTheThreeDevices() {
        AllInOne machine = new AllInOne();
        Printer asPrinter = machine;
        Scanner asScanner = machine;
        FaxMachine asFax = machine;

        assertThat(asPrinter.print("a")).isEqualTo("PRINTED: a");
        assertThat(asScanner.scan()).isEqualTo("SCAN-1");
        assertThat(asFax.fax("+4915112345678", "a")).isTrue();
        assertThat(asFax.fax("12", "a")).isFalse();
        assertThat(asFax.fax("not-a-number", "a")).isFalse();
        assertThat(asFax.fax("5551234", "a")).isTrue();
        assertThatThrownBy(() -> asFax.fax("5551234", "")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRunAPrintJobOnAnyPrinterEvenOneThatCanDoNothingElse() {
        List<String> seen = new ArrayList<>();
        Printer printerOnly = doc -> {
            seen.add(doc);
            return "fake:" + doc;
        };

        List<String> out = PrintJob.run(printerOnly, List.of("a", "b"));

        assertThat(out).containsExactly("fake:a", "fake:b");
        assertThat(seen).containsExactly("a", "b");
        assertThat(PrintJob.run(new BasicPrinter(), List.of("x"))).containsExactly("PRINTED: x");
    }

    @Test
    void shouldRunAScanJobOnAnyScannerEvenOneThatCanDoNothingElse() {
        int[] counter = {0};
        Scanner scannerOnly = () -> "page" + (++counter[0]);

        assertThat(ScanJob.run(scannerOnly, 3)).containsExactly("page1", "page2", "page3");
        assertThat(ScanJob.run(new AllInOne(), 2)).containsExactly("SCAN-1", "SCAN-2");
    }

    @Test
    void shouldRunAFaxJobOnAnyFaxMachineEvenOneThatCanDoNothingElse() {
        List<String> sent = new ArrayList<>();
        FaxMachine faxOnly = (number, doc) -> sent.add(number + ":" + doc);

        assertThat(FaxJob.send(faxOnly, "5551234", "contract")).isTrue();
        assertThat(sent).containsExactly("5551234:contract");
        assertThat(FaxJob.send(new AllInOne(), "bad", "contract")).isFalse();
    }

    @Test
    void shouldKeepTheInterfacesSmallAndNeverForceADeviceToImplementWhatItCannotDo() {
        assertThat(new BasicPrinter().print("x")).isEqualTo("PRINTED: x");

        assertThat(BasicPrinter.class.getInterfaces()).containsExactly(Printer.class);
        assertThat(FlatbedScanner.class.getInterfaces()).containsExactly(Scanner.class);
        assertThat(AllInOne.class.getInterfaces()).containsExactlyInAnyOrder(Printer.class, Scanner.class, FaxMachine.class);
        assertThat(Scanner.class.isAssignableFrom(BasicPrinter.class)).isFalse();
        assertThat(FaxMachine.class.isAssignableFrom(BasicPrinter.class)).isFalse();
        assertThat(Printer.class.isAssignableFrom(FlatbedScanner.class)).isFalse();
    }

    @Test
    void shouldValidateJobArguments() {
        assertThat(PrintJob.run(new BasicPrinter(), null)).isEmpty();
        assertThatThrownBy(() -> PrintJob.run(null, List.of("a"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> ScanJob.run(new FlatbedScanner(), 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> ScanJob.run(null, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> FaxJob.send(null, "5551234", "a")).isInstanceOf(IllegalArgumentException.class);
    }
}
