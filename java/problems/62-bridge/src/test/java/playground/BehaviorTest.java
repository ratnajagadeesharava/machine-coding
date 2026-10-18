package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Channel;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Alert;
import solution.DigestAlert;
import solution.EmailChannel;
import solution.InfoAlert;
import solution.SlackChannel;
import solution.SmsChannel;
import solution.UrgentAlert;

class BehaviorTest {

    /** A channel written in the test: records exactly what the alert asked it to deliver. */
    private static final class Recording implements Channel {
        final List<String> delivered = new ArrayList<>();

        public void deliver(String recipient, String title, String body) {
            delivered.add(recipient + "|" + title + "|" + body);
        }
    }

    @Test
    void shouldShapeInfoAlertsIndependentlyOfTheChannel() {
        Recording channel = new Recording();

        new InfoAlert(channel).send("ann", "Build finished");

        assertThat(channel.delivered).containsExactly("ann|Info|Build finished");
    }

    @Test
    void shouldShapeUrgentAlertsIndependentlyOfTheChannel() {
        Recording channel = new Recording();

        new UrgentAlert(channel).send("ann", "disk is full");

        assertThat(channel.delivered).containsExactly("ann|URGENT|DISK IS FULL");
    }

    @Test
    void shouldCollectDigestMessagesPerRecipientAndFlushThemTogether() {
        Recording channel = new Recording();
        DigestAlert digest = new DigestAlert(channel);

        digest.send("ann", "one");
        digest.send("bob", "other");
        digest.send("ann", "two");
        digest.send("ann", "three");

        assertThat(channel.delivered).isEmpty();
        assertThat(digest.pending("ann")).isEqualTo(3);
        assertThat(digest.flush("ann")).isTrue();
        assertThat(channel.delivered).containsExactly("ann|Digest (3)|one; two; three");
        assertThat(digest.pending("ann")).isZero();
        assertThat(digest.flush("ann")).isFalse();
        assertThat(digest.pending("bob")).isEqualTo(1);
        assertThat(channel.delivered).hasSize(1);
    }

    @Test
    void shouldFormatEachChannelIndependentlyOfTheAlert() {
        List<String> wire = new ArrayList<>();

        new EmailChannel(wire::add).deliver("ann@x.io", "Report", "All good");
        new SlackChannel(wire::add).deliver("ann", "Report", "All good");
        new SmsChannel(wire::add).deliver("+1555", "Report", "All good");

        assertThat(wire).containsExactly(
                "EMAIL to:ann@x.io subject:Report | All good",
                "SLACK @ann *Report* All good",
                "SMS +1555: Report: All good");
    }

    @Test
    void shouldCutLongSmsTextsTo40Characters() {
        List<String> wire = new ArrayList<>();
        SmsChannel sms = new SmsChannel(wire::add);

        sms.deliver("1", "T", "x".repeat(38));
        sms.deliver("1", "T", "x".repeat(37));

        assertThat(wire.get(0)).isEqualTo("SMS 1: T: " + "x".repeat(34) + "...");
        assertThat(wire.get(1)).isEqualTo("SMS 1: T: " + "x".repeat(37));
        assertThat(wire.get(0).substring("SMS 1: ".length())).hasSize(40);
    }

    @Test
    void shouldCombineAnyAlertWithAnyChannelWithoutAnExtraClassPerPair() {
        List<String> wire = new ArrayList<>();
        List<Alert> alerts = List.of(
                new InfoAlert(new EmailChannel(wire::add)),
                new UrgentAlert(new SlackChannel(wire::add)),
                new InfoAlert(new SlackChannel(wire::add)),
                new UrgentAlert(new EmailChannel(wire::add)));

        for (Alert alert : alerts) {
            alert.send("ann", "ping");
        }

        assertThat(wire).containsExactly(
                "EMAIL to:ann subject:Info | ping",
                "SLACK @ann *URGENT* PING",
                "SLACK @ann *Info* ping",
                "EMAIL to:ann subject:URGENT | PING");
    }

    @Test
    void shouldSwitchTheChannelAtRuntime() {
        List<String> wire = new ArrayList<>();
        Alert alert = new InfoAlert(new EmailChannel(wire::add));
        alert.send("ann", "first");

        alert.setChannel(new SlackChannel(wire::add));
        alert.send("ann", "second");

        assertThat(wire).containsExactly("EMAIL to:ann subject:Info | first", "SLACK @ann *Info* second");
        assertThat(alert.channel()).isInstanceOf(SlackChannel.class);
    }

    @Test
    void shouldWorkWithAChannelTypeItHasNeverSeen() {
        Recording pager = new Recording();
        DigestAlert digest = new DigestAlert(pager);

        new UrgentAlert(pager).send("oncall", "db down");
        digest.send("oncall", "a");
        digest.flush("oncall");

        assertThat(pager.delivered).containsExactly("oncall|URGENT|DB DOWN", "oncall|Digest (1)|a");
    }

    @Test
    void shouldRejectMissingChannelsAndWires() {
        assertThatThrownBy(() -> new InfoAlert(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new EmailChannel(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new SmsChannel(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new SlackChannel(null)).isInstanceOf(IllegalArgumentException.class);
        Alert alert = new UrgentAlert(new Recording());
        assertThatThrownBy(() -> alert.setChannel(null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(alert.channel()).isNotNull();
    }
}
