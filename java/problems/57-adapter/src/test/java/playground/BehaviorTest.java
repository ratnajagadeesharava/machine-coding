package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Delivery;
import contract.LegacySmsGateway;
import contract.Notifier;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.AlertService;
import solution.LegacySmsAdapter;

class BehaviorTest {

    private static class FakeLegacy implements LegacySmsGateway {
        final List<String> received = new ArrayList<>();
        int code;

        public int submitXml(String xml) {
            received.add(xml);
            return code;
        }
    }

    @Test
    void shouldTranslateASendIntoTheLegacyXmlFormat() {
        FakeLegacy legacy = new FakeLegacy();
        Notifier notifier = new LegacySmsAdapter(legacy);

        Delivery result = notifier.send("+15551234", "Hello there");

        assertThat(result).isEqualTo(new Delivery(true, ""));
        assertThat(legacy.received).containsExactly("<sms><to>+15551234</to><body>Hello there</body></sms>");
    }

    @Test
    void shouldEscapeXmlSpecialCharacters() {
        FakeLegacy legacy = new FakeLegacy();

        new LegacySmsAdapter(legacy).send("1", "A&B <c> \"d\" 'e'");

        assertThat(legacy.received).containsExactly(
                "<sms><to>1</to><body>A&amp;B &lt;c&gt; &quot;d&quot; &apos;e&apos;</body></sms>");
    }

    @Test
    void shouldTranslateLegacyReturnCodesIntoDeliveries() {
        FakeLegacy legacy = new FakeLegacy();
        Notifier notifier = new LegacySmsAdapter(legacy);

        legacy.code = 0;
        assertThat(notifier.send("1", "m")).isEqualTo(new Delivery(true, ""));
        legacy.code = 1;
        assertThat(notifier.send("1", "m")).isEqualTo(new Delivery(false, "invalid number"));
        legacy.code = 2;
        assertThat(notifier.send("1", "m")).isEqualTo(new Delivery(false, "message too long"));
        legacy.code = 3;
        assertThat(notifier.send("1", "m")).isEqualTo(new Delivery(false, "unauthorized"));
        legacy.code = 42;
        assertThat(notifier.send("1", "m")).isEqualTo(new Delivery(false, "unknown error (code 42)"));
    }

    @Test
    void shouldRejectBlankInputWithoutBotheringTheLegacySystem() {
        FakeLegacy legacy = new FakeLegacy();
        Notifier notifier = new LegacySmsAdapter(legacy);

        assertThat(notifier.send(" ", "m")).isEqualTo(new Delivery(false, "invalid number"));
        assertThat(notifier.send(null, "m")).isEqualTo(new Delivery(false, "invalid number"));
        assertThat(notifier.send("1", "")).isEqualTo(new Delivery(false, "empty message"));
        assertThat(notifier.send("", "")).isEqualTo(new Delivery(false, "invalid number"));

        assertThat(legacy.received).isEmpty();
    }

    @Test
    void shouldCallTheLegacySystemExactlyOncePerSend() {
        FakeLegacy legacy = new FakeLegacy();
        Notifier notifier = new LegacySmsAdapter(legacy);

        notifier.send("1", "a");
        notifier.send("2", "b");

        assertThat(legacy.received).hasSize(2);
    }

    @Test
    void shouldWrapTheLegacyGatewayInsteadOfExtendingAnything() {
        assertThat(LegacySmsAdapter.class.getSuperclass()).isEqualTo(Object.class);
        assertThat(LegacySmsAdapter.class.getInterfaces()).containsExactly(Notifier.class);
        assertThat(new LegacySmsAdapter(new FakeLegacy()).send("1", "m").ok()).isTrue();
    }

    @Test
    void shouldLetApplicationCodeUseTheAdapterWithoutKnowingAboutXml() {
        FakeLegacy legacy = new FakeLegacy() {
            @Override
            public int submitXml(String xml) {
                super.submitXml(xml);
                return xml.contains("<to>BAD</to>") ? 1 : 0;
            }
        };
        AlertService alerts = new AlertService(new LegacySmsAdapter(legacy));

        List<String> failures = alerts.alertAll(List.of("111", "BAD", "222"), "Server down");

        assertThat(failures).containsExactly("BAD: invalid number");
        assertThat(legacy.received).hasSize(3);
    }

    @Test
    void shouldWorkWithAnyNotifierNotJustTheAdapter() {
        Notifier flaky = (phone, message) -> phone.startsWith("0") ? new Delivery(false, "blocked") : new Delivery(true, "");
        AlertService alerts = new AlertService(flaky);

        assertThat(alerts.alertAll(List.of("0100", "200", "0300"), "hi")).containsExactly("0100: blocked", "0300: blocked");
        assertThat(alerts.alertAll(List.of(), "hi")).isEmpty();
        assertThat(alerts.alertAll(null, "hi")).isEmpty();
    }
}
