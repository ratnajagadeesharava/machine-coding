package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.DeliveryGateway;
import contract.NotificationApi;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    /** Accepts everything except the channels listed in `refused`; records channel and body. */
    static class Gateway implements DeliveryGateway {
        final List<String> log = new ArrayList<>();
        final List<String> refused;

        Gateway(String... refused) {
            this.refused = List.of(refused);
        }

        @Override
        public boolean deliver(String channel, String address, String body) {
            log.add(channel + "|" + body);
            return !refused.contains(channel);
        }
    }

    private NotificationApi system(Gateway gateway) {
        NotificationApi api = Candidate.of(() -> Solution.create(gateway));
        api.registerUser("u", Map.of("email", "e", "sms", "s", "push", "p"));
        return api;
    }

    @Test
    void parameterValuesShouldNeverBeExpandedAgain() {
        Gateway gw = new Gateway();
        NotificationApi api = system(gw);
        api.defineTemplate("t", "Hi {name}");

        api.send("u", "c", "t", Map.of("name", "{code}", "code", "1"), List.of("email"));

        assertThat(gw.log).containsExactly("email|Hi {code}");
    }

    @Test
    void bracesThatAreNotPlaceholdersShouldStayLiteral() {
        Gateway gw = new Gateway();
        NotificationApi api = system(gw);
        api.defineTemplate("t", "{} and {not valid} and {a}");

        String result = api.send("u", "c", "t", Map.of("a", "x"), List.of("email"));

        assertThat(result).isEqualTo("DELIVERED:email");
        assertThat(gw.log).containsExactly("email|{} and {not valid} and x");
    }

    @Test
    void skippedChannelBetweenFailingAndSucceedingOnesShouldCostNoCalls() {
        Gateway gw = new Gateway("email");
        NotificationApi api = system(gw);
        api.defineTemplate("t", "m");
        api.setPreference("u", "sms", "c", false);

        String result = api.send("u", "c", "t", Map.of(), List.of("email", "sms", "push"));

        assertThat(result).isEqualTo("DELIVERED:push");
        assertThat(gw.log).containsExactly("email|m", "email|m", "email|m", "push|m");
    }

    @Test
    void preferenceSetForUnregisteredUserShouldNotApplyToALaterRegistration() {
        Gateway gw = new Gateway();
        NotificationApi api = system(gw);
        api.defineTemplate("t", "m");
        api.setPreference("ghost", "email", "c", false);

        api.registerUser("ghost", Map.of("email", "g"));

        assertThat(api.send("ghost", "c", "t", Map.of(), List.of("email"))).isEqualTo("DELIVERED:email");
    }

    @Test
    void checksShouldRunInTheOrderUnknownUserInvalidSuppressed() {
        Gateway gw = new Gateway();
        NotificationApi api = system(gw);
        api.defineTemplate("t", "{x}");
        api.setPreference("u", "email", "c", false);
        api.registerUser("empty", Map.of());

        assertThat(api.send("nobody", "c", "missing", Map.of(), List.of())).isEqualTo("UNKNOWN_USER");
        assertThat(api.send("u", "c", "t", Map.of(), List.of("email"))).isEqualTo("INVALID");
        assertThat(api.send("u", "c", "t", Map.of("x", "1"), List.of("email"))).isEqualTo("SUPPRESSED");
        assertThat(api.send("empty", "c", "t", Map.of("x", "1"), List.of("email"))).isEqualTo("SUPPRESSED");
        assertThat(gw.log).isEmpty();
    }
}
