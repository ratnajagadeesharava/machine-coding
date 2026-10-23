package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.DeliveryGateway;
import contract.NotificationApi;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    record Call(String channel, String address, String body) {}

    /** Records every attempt and refuses the first N attempts per channel when scripted to. */
    static class FakeGateway implements DeliveryGateway {
        final List<Call> calls = new ArrayList<>();
        private final Map<String, Integer> refusalsLeft = new HashMap<>();

        void refuseFirst(String channel, int attempts) {
            refusalsLeft.put(channel, attempts);
        }

        void alwaysRefuse(String channel) {
            refuseFirst(channel, Integer.MAX_VALUE);
        }

        @Override
        public boolean deliver(String channel, String address, String body) {
            calls.add(new Call(channel, address, body));
            int left = refusalsLeft.getOrDefault(channel, 0);
            if (left > 0) {
                refusalsLeft.put(channel, left - 1);
                return false;
            }
            return true;
        }

        List<String> channelsCalled() {
            return calls.stream().map(Call::channel).toList();
        }
    }

    private final FakeGateway gateway = new FakeGateway();

    private NotificationApi system() {
        NotificationApi api = Candidate.of(() -> Solution.create(gateway));
        api.registerUser("u1", Map.of("email", "a@x.com", "sms", "555-0100", "push", "device-1"));
        api.defineTemplate("otp", "Your code is {code}");
        return api;
    }

    @Test
    void shouldDeliverOnTheFirstChannelWithTheRenderedBody() {
        NotificationApi api = system();
        api.defineTemplate("welcome", "Hi {name}, your code is {code}.");

        String result = api.send("u1", "security", "welcome", Map.of("name", "Asha", "code", "42"), List.of("email", "sms"));

        assertThat(result).isEqualTo("DELIVERED:email");
        assertThat(gateway.calls).containsExactly(new Call("email", "a@x.com", "Hi Asha, your code is 42."));
    }

    @Test
    void shouldFollowTheCallersChannelOrder() {
        NotificationApi api = system();

        String result = api.send("u1", "security", "otp", Map.of("code", "1"), List.of("sms", "email"));

        assertThat(result).isEqualTo("DELIVERED:sms");
        assertThat(gateway.calls).containsExactly(new Call("sms", "555-0100", "Your code is 1"));
    }

    @Test
    void shouldRetryTheSameChannelUpToThreeAttempts() {
        NotificationApi api = system();
        gateway.refuseFirst("email", 2);

        String result = api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email", "sms"));

        assertThat(result).isEqualTo("DELIVERED:email");
        assertThat(gateway.channelsCalled()).containsExactly("email", "email", "email");
    }

    @Test
    void shouldFallBackToTheNextChannelAfterThreeRefusedAttempts() {
        NotificationApi api = system();
        gateway.alwaysRefuse("email");

        String result = api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email", "sms"));

        assertThat(result).isEqualTo("DELIVERED:sms");
        assertThat(gateway.channelsCalled()).containsExactly("email", "email", "email", "sms");
    }

    @Test
    void shouldReportFailedWhenEveryChannelRefusesAllAttempts() {
        NotificationApi api = system();
        gateway.alwaysRefuse("email");
        gateway.alwaysRefuse("sms");

        String result = api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email", "sms"));

        assertThat(result).isEqualTo("FAILED");
        assertThat(gateway.calls).hasSize(6);
    }

    @Test
    void shouldGiveEverySendAFreshSetOfAttempts() {
        NotificationApi api = system();
        gateway.refuseFirst("email", 3);

        assertThat(api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email"))).isEqualTo("FAILED");
        assertThat(gateway.calls).hasSize(3);

        assertThat(api.send("u1", "security", "otp", Map.of("code", "2"), List.of("email"))).isEqualTo("DELIVERED:email");
        assertThat(gateway.calls).hasSize(4);
    }

    @Test
    void shouldSkipChannelsTheUserHasNoAddressFor() {
        NotificationApi api = Candidate.of(() -> Solution.create(gateway));
        api.registerUser("u2", Map.of("sms", "555-0199"));
        api.defineTemplate("otp", "Your code is {code}");

        String result = api.send("u2", "security", "otp", Map.of("code", "1"), List.of("email", "sms"));

        assertThat(result).isEqualTo("DELIVERED:sms");
        assertThat(gateway.channelsCalled()).containsExactly("sms");
    }

    @Test
    void shouldSkipChannelsSwitchedOffForTheCategoryOnly() {
        NotificationApi api = system();
        api.setPreference("u1", "email", "marketing", false);

        assertThat(api.send("u1", "marketing", "otp", Map.of("code", "1"), List.of("email", "sms"))).isEqualTo("DELIVERED:sms");
        assertThat(api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email", "sms"))).isEqualTo("DELIVERED:email");
        assertThat(gateway.channelsCalled()).containsExactly("sms", "email");
    }

    @Test
    void shouldAllowSwitchingAChannelBackOn() {
        NotificationApi api = system();
        api.setPreference("u1", "email", "marketing", false);
        api.setPreference("u1", "email", "marketing", true);

        String result = api.send("u1", "marketing", "otp", Map.of("code", "1"), List.of("email"));

        assertThat(result).isEqualTo("DELIVERED:email");
    }

    @Test
    void shouldSuppressWithoutCallingTheGatewayWhenNothingIsEligible() {
        NotificationApi api = system();
        api.setPreference("u1", "email", "marketing", false);
        api.setPreference("u1", "sms", "marketing", false);

        assertThat(api.send("u1", "marketing", "otp", Map.of("code", "1"), List.of("email", "sms"))).isEqualTo("SUPPRESSED");
        assertThat(api.send("u1", "security", "otp", Map.of("code", "1"), List.of("fax"))).isEqualTo("SUPPRESSED");

        assertThat(gateway.calls).isEmpty();
    }

    @Test
    void shouldReturnInvalidWithoutCallingTheGateway() {
        NotificationApi api = system();

        assertThat(api.send("u1", "security", "otp", Map.of(), List.of("email"))).isEqualTo("INVALID");
        assertThat(api.send("u1", "security", "otp", Map.of("other", "x"), List.of("email"))).isEqualTo("INVALID");
        assertThat(api.send("u1", "security", "no-such-template", Map.of(), List.of("email"))).isEqualTo("INVALID");
        assertThat(api.send("u1", "security", "otp", Map.of("code", "1"), List.of())).isEqualTo("INVALID");

        assertThat(gateway.calls).isEmpty();
    }

    @Test
    void shouldReturnUnknownUserWithoutCallingTheGateway() {
        NotificationApi api = system();

        assertThat(api.send("ghost", "security", "otp", Map.of("code", "1"), List.of("email"))).isEqualTo("UNKNOWN_USER");

        assertThat(gateway.calls).isEmpty();
    }

    @Test
    void shouldReplaceEveryOccurrenceAndIgnoreExtraParameters() {
        NotificationApi api = system();
        api.defineTemplate("echo", "{a}-{a}-{b}");

        api.send("u1", "security", "echo", Map.of("a", "1", "b", "2", "unused", "9"), List.of("email"));

        assertThat(gateway.calls).extracting(Call::body).containsExactly("1-1-2");
    }

    @Test
    void shouldUseTheLatestTemplateDefinition() {
        NotificationApi api = system();
        api.defineTemplate("otp", "New code: {code}");

        api.send("u1", "security", "otp", Map.of("code", "7"), List.of("email"));

        assertThat(gateway.calls).extracting(Call::body).containsExactly("New code: 7");
    }

    @Test
    void shouldTryAChannelListedTwiceOnlyOnce() {
        NotificationApi api = system();
        gateway.alwaysRefuse("email");

        String result = api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email", "email", "sms"));

        assertThat(result).isEqualTo("DELIVERED:sms");
        assertThat(gateway.channelsCalled()).containsExactly("email", "email", "email", "sms");
    }

    @Test
    void shouldReplaceAddressesButKeepPreferencesWhenRegisteringAgain() {
        NotificationApi api = system();
        api.setPreference("u1", "email", "marketing", false);

        api.registerUser("u1", Map.of("email", "new@x.com"));

        assertThat(api.send("u1", "marketing", "otp", Map.of("code", "1"), List.of("email"))).isEqualTo("SUPPRESSED");
        assertThat(api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email", "sms"))).isEqualTo("DELIVERED:email");
        assertThat(gateway.calls).containsExactly(new Call("email", "new@x.com", "Your code is 1"));
    }

    @Test
    void shouldKeepPreferencesPerUser() {
        NotificationApi api = system();
        api.registerUser("u2", Map.of("email", "b@x.com"));
        api.setPreference("u1", "email", "marketing", false);

        assertThat(api.send("u2", "marketing", "otp", Map.of("code", "1"), List.of("email"))).isEqualTo("DELIVERED:email");
        assertThat(api.send("u1", "marketing", "otp", Map.of("code", "1"), List.of("email"))).isEqualTo("SUPPRESSED");
    }

    @Test
    void shouldSendOnlyOnceWhenADuplicatedChannelIsDelivered() {
        NotificationApi api = system();

        String result = api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email", "email", "sms"));

        assertThat(result).isEqualTo("DELIVERED:email");
        assertThat(gateway.calls).containsExactly(new Call("email", "a@x.com", "Your code is 1"));
    }

    @Test
    void shouldFailAfterThreeAttemptsWhenTheOnlyChannelIsListedTwice() {
        NotificationApi api = system();
        gateway.alwaysRefuse("email");

        String result = api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email", "email"));

        assertThat(result).isEqualTo("FAILED");
        assertThat(gateway.channelsCalled()).containsExactly("email", "email", "email");
    }

    @Test
    void shouldKeepTheFirstPositionOfADuplicatedChannel() {
        NotificationApi api = system();
        gateway.alwaysRefuse("sms");
        gateway.alwaysRefuse("email");

        String result = api.send("u1", "security", "otp", Map.of("code", "1"), List.of("sms", "email", "sms", "push"));

        assertThat(result).isEqualTo("DELIVERED:push");
        assertThat(gateway.channelsCalled())
                .containsExactly("sms", "sms", "sms", "email", "email", "email", "push");
    }

    @Test
    void shouldNotCallADisabledChannelListedTwice() {
        NotificationApi api = system();
        api.setPreference("u1", "email", "security", false);

        String result = api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email", "email", "sms"));

        assertThat(result).isEqualTo("DELIVERED:sms");
        assertThat(gateway.channelsCalled()).containsExactly("sms");
    }

    @Test
    void shouldTreatChannelNamesAsCaseSensitive() {
        NotificationApi api = system();

        assertThat(api.send("u1", "security", "otp", Map.of("code", "1"), List.of("EMAIL", "Sms"))).isEqualTo("SUPPRESSED");
        assertThat(api.send("u1", "security", "otp", Map.of("code", "1"), List.of("Email", "email"))).isEqualTo("DELIVERED:email");

        assertThat(gateway.channelsCalled()).containsExactly("email");
    }

    @Test
    void shouldDeliverOnAnyChannelNameTheUserHasAnAddressFor() {
        NotificationApi api = system();
        api.registerUser("u3", Map.of("whatsapp", "+1-555-0100", "carrier-pigeon", "roof-7"));

        String result = api.send("u3", "security", "otp", Map.of("code", "1"), List.of("telegram", "carrier-pigeon", "whatsapp"));

        assertThat(result).isEqualTo("DELIVERED:carrier-pigeon");
        assertThat(gateway.calls).containsExactly(new Call("carrier-pigeon", "roof-7", "Your code is 1"));
    }

    @Test
    void shouldHandTheSameAddressAndBodyToEveryRetry() {
        NotificationApi api = system();
        gateway.refuseFirst("email", 2);

        api.send("u1", "security", "otp", Map.of("code", "9"), List.of("email"));

        Call expected = new Call("email", "a@x.com", "Your code is 9");
        assertThat(gateway.calls).containsExactly(expected, expected, expected);
    }

    @Test
    void shouldStopCallingTheGatewayAfterTheFirstAcceptedAttempt() {
        NotificationApi api = system();
        gateway.refuseFirst("email", 1);

        String result = api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email", "sms", "push"));

        assertThat(result).isEqualTo("DELIVERED:email");
        assertThat(gateway.channelsCalled()).containsExactly("email", "email");
    }

    @Test
    void shouldReportFailedNotSuppressedWhenTheOnlyEligibleChannelRefuses() {
        NotificationApi api = system();
        api.setPreference("u1", "sms", "security", false);
        gateway.alwaysRefuse("email");

        String result = api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email", "sms", "fax"));

        assertThat(result).isEqualTo("FAILED");
        assertThat(gateway.channelsCalled()).containsExactly("email", "email", "email");
    }

    @Test
    void shouldForgetOldAddressesWhenRegisteringAgain() {
        NotificationApi api = system();

        api.registerUser("u1", Map.of("email", "new@x.com"));

        assertThat(api.send("u1", "security", "otp", Map.of("code", "1"), List.of("sms", "push"))).isEqualTo("SUPPRESSED");
        assertThat(gateway.calls).isEmpty();

        api.registerUser("u1", Map.of());
        assertThat(api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email"))).isEqualTo("SUPPRESSED");
        assertThat(gateway.calls).isEmpty();
    }

    @Test
    void shouldKeepPreferencesIndependentPerChannelAndCategory() {
        NotificationApi api = system();
        api.setPreference("u1", "email", "marketing", false);
        api.setPreference("u1", "sms", "security", false);

        assertThat(api.send("u1", "marketing", "otp", Map.of("code", "1"), List.of("email", "sms"))).isEqualTo("DELIVERED:sms");
        assertThat(api.send("u1", "security", "otp", Map.of("code", "1"), List.of("sms", "email"))).isEqualTo("DELIVERED:email");
        assertThat(api.send("u1", "updates", "otp", Map.of("code", "1"), List.of("sms", "email"))).isEqualTo("DELIVERED:sms");
        assertThat(gateway.channelsCalled()).containsExactly("sms", "email", "sms");
    }

    @Test
    void shouldApplyPreferenceChangesToTheNextSend() {
        NotificationApi api = system();
        assertThat(api.send("u1", "marketing", "otp", Map.of("code", "1"), List.of("email"))).isEqualTo("DELIVERED:email");

        api.setPreference("u1", "email", "marketing", false);
        assertThat(api.send("u1", "marketing", "otp", Map.of("code", "1"), List.of("email"))).isEqualTo("SUPPRESSED");

        api.setPreference("u1", "email", "marketing", true);
        assertThat(api.send("u1", "marketing", "otp", Map.of("code", "1"), List.of("email"))).isEqualTo("DELIVERED:email");
        assertThat(gateway.calls).hasSize(2);
    }

    @Test
    void shouldDeliverToEachUsersOwnAddressInSendOrder() {
        NotificationApi api = system();
        api.registerUser("u2", Map.of("email", "b@x.com"));
        api.registerUser("u3", Map.of("email", "c@x.com"));

        api.send("u3", "security", "otp", Map.of("code", "3"), List.of("email"));
        api.send("u1", "security", "otp", Map.of("code", "1"), List.of("email"));
        api.send("u2", "security", "otp", Map.of("code", "2"), List.of("email"));
        api.send("u3", "security", "otp", Map.of("code", "4"), List.of("email"));

        assertThat(gateway.calls).containsExactly(
                new Call("email", "c@x.com", "Your code is 3"),
                new Call("email", "a@x.com", "Your code is 1"),
                new Call("email", "b@x.com", "Your code is 2"),
                new Call("email", "c@x.com", "Your code is 4"));
    }

    @Test
    void shouldReturnInvalidWhenOnlySomeRequiredParametersAreGiven() {
        NotificationApi api = system();
        api.defineTemplate("two", "{a} and {b}");

        assertThat(api.send("u1", "security", "two", Map.of("a", "1"), List.of("email"))).isEqualTo("INVALID");

        assertThat(gateway.calls).isEmpty();
    }

    @Test
    void shouldAcceptEmptyStringAsAParameterValue() {
        NotificationApi api = system();
        api.defineTemplate("t", "[{a}]");

        String result = api.send("u1", "security", "t", Map.of("a", ""), List.of("email"));

        assertThat(result).isEqualTo("DELIVERED:email");
        assertThat(gateway.calls).extracting(Call::body).containsExactly("[]");
    }

    @Test
    void shouldRecognisePlaceholderNamesWithDigitsAndUnderscores() {
        NotificationApi api = system();
        api.defineTemplate("t", "{user_1}/{A_b2}/{9}");

        api.send("u1", "security", "t", Map.of("user_1", "x", "A_b2", "y", "9", "z"), List.of("email"));

        assertThat(gateway.calls).extracting(Call::body).containsExactly("x/y/z");
    }
}
