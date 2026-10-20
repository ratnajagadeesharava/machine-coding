package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.ChatApi;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private ChatApi chat() {
        ChatApi c = Candidate.of(Solution::create);
        for (String u : List.of("ann", "bob", "cy", "dee")) {
            c.register(u);
        }
        return c;
    }

    private static List<String> texts(List<String> history) {
        List<String> out = new ArrayList<>();
        for (String h : history) {
            out.add(h.split("\\|", 3)[1] + ":" + h.split("\\|", 3)[2]);
        }
        return out;
    }

    @Test
    void shouldWaitForOfflineRecipientsAndDeliverOnConnect() {
        ChatApi c = chat();

        String id = c.sendDirect("ann", "bob", "hi");

        assertThat(c.statusOf(id)).isEqualTo("SENT");
        c.connect("bob");
        assertThat(c.statusOf(id)).isEqualTo("DELIVERED");
    }

    @Test
    void shouldDeliverImmediatelyToOnlineRecipientsAndThenMarkRead() {
        ChatApi c = chat();
        c.connect("bob");

        String id = c.sendDirect("ann", "bob", "hi");
        assertThat(c.statusOf(id)).isEqualTo("DELIVERED");
        assertThat(c.unreadCount("bob", "dm:ann:bob")).isEqualTo(1);
        c.read("bob", "dm:ann:bob");

        assertThat(c.statusOf(id)).isEqualTo("READ");
        assertThat(c.unreadCount("bob", "dm:ann:bob")).isZero();
    }

    @Test
    void shouldStopDeliveringAfterDisconnect() {
        ChatApi c = chat();
        c.connect("bob");
        c.disconnect("bob");

        String id = c.sendDirect("ann", "bob", "hi");

        assertThat(c.statusOf(id)).isEqualTo("SENT");
    }

    @Test
    void shouldKeepDirectHistoryInSendOrderForBothSides() {
        ChatApi c = chat();
        c.sendDirect("ann", "bob", "one");
        c.sendDirect("bob", "ann", "two");
        c.sendDirect("ann", "bob", "three");
        c.sendDirect("cy", "ann", "other chat");

        assertThat(texts(c.history("ann", "dm:ann:bob"))).containsExactly("ann:one", "bob:two", "ann:three");
        assertThat(texts(c.history("bob", "dm:ann:bob"))).containsExactly("ann:one", "bob:two", "ann:three");
        assertThat(texts(c.history("ann", "dm:ann:cy"))).containsExactly("cy:other chat");
        assertThat(c.history("dee", "dm:ann:bob")).isEmpty();
    }

    @Test
    void shouldAggregateGroupStatusOverAllRecipients() {
        ChatApi c = chat();
        c.createGroup("g", "ann", List.of("bob", "cy"));
        c.connect("bob");

        String id = c.sendGroup("ann", "g", "hello all");
        assertThat(c.statusOf(id)).isEqualTo("SENT"); // cy has not got it
        c.connect("cy");
        assertThat(c.statusOf(id)).isEqualTo("DELIVERED");
        c.read("bob", "group:g");
        assertThat(c.statusOf(id)).isEqualTo("DELIVERED"); // cy has not read it
        c.read("cy", "group:g");
        assertThat(c.statusOf(id)).isEqualTo("READ");
    }

    @Test
    void shouldNotCountTheSenderAsARecipient() {
        ChatApi c = chat();
        c.createGroup("g", "ann", List.of("bob"));
        c.connect("bob");

        String id = c.sendGroup("ann", "g", "x");

        assertThat(c.unreadCount("ann", "group:g")).isZero();
        assertThat(c.statusOf(id)).isEqualTo("DELIVERED");
    }

    @Test
    void shouldOnlyLetMembersSendAndOnlyTheAdminManageMembers() {
        ChatApi c = chat();
        c.createGroup("g", "ann", List.of("bob"));

        assertThatThrownBy(() -> c.sendGroup("cy", "g", "x")).isInstanceOf(SecurityException.class);
        assertThatThrownBy(() -> c.addMember("bob", "g", "cy")).isInstanceOf(SecurityException.class);
        assertThatThrownBy(() -> c.removeMember("bob", "g", "ann")).isInstanceOf(SecurityException.class);
        assertThatThrownBy(() -> c.removeMember("ann", "g", "ann")).isInstanceOf(SecurityException.class);
        assertThatThrownBy(() -> c.sendGroup("ann", "nope", "x")).isInstanceOf(IllegalArgumentException.class);
        c.addMember("ann", "g", "cy");
        c.sendGroup("cy", "g", "now I can");
    }

    @Test
    void shouldShowNewMembersOnlyMessagesSentAfterTheyJoined() {
        ChatApi c = chat();
        c.createGroup("g", "ann", List.of("bob"));
        c.sendGroup("ann", "g", "before");

        c.addMember("ann", "g", "cy");
        c.sendGroup("bob", "g", "after");

        assertThat(texts(c.history("cy", "group:g"))).containsExactly("bob:after");
        assertThat(texts(c.history("bob", "group:g"))).containsExactly("ann:before", "bob:after");
    }

    @Test
    void shouldStopReceivingAfterLeavingButKeepWhatWasAlreadyThere() {
        ChatApi c = chat();
        c.createGroup("g", "ann", List.of("bob", "cy"));
        c.sendGroup("ann", "g", "one");

        c.removeMember("cy", "g", "cy"); // leaves
        c.sendGroup("ann", "g", "two");

        assertThat(texts(c.history("cy", "group:g"))).containsExactly("ann:one");
        assertThat(texts(c.history("bob", "group:g"))).containsExactly("ann:one", "ann:two");
        assertThatThrownBy(() -> c.sendGroup("cy", "g", "x")).isInstanceOf(SecurityException.class);
        assertThat(c.unreadCount("cy", "group:g")).isEqualTo(1);
    }

    @Test
    void shouldIgnoreMessagesOfAMemberWhoLeftForTheStatus() {
        ChatApi c = chat();
        c.createGroup("g", "ann", List.of("bob", "cy"));
        c.connect("bob");
        String id = c.sendGroup("ann", "g", "x"); // cy never connects
        assertThat(c.statusOf(id)).isEqualTo("SENT");

        c.removeMember("ann", "g", "cy");
        c.read("bob", "group:g");

        assertThat(c.statusOf(id)).isEqualTo("READ");
    }

    @Test
    void shouldRejectUnknownUsers() {
        ChatApi c = chat();

        assertThatThrownBy(() -> c.sendDirect("ann", "ghost", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.sendDirect("ghost", "ann", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.connect("ghost")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.statusOf("m-nope")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldGiveEveryMemberTheSameOrderUnderConcurrentSenders() {
        ChatApi c = chat();
        c.createGroup("g", "ann", List.of("bob", "cy", "dee"));
        String[] users = {"ann", "bob", "cy", "dee"};

        Concurrently.run(8, t -> {
            for (int i = 0; i < 100; i++) {
                c.sendGroup(users[t % 4], "g", "t" + t + "-" + i);
            }
        });

        List<String> reference = c.history("ann", "group:g");
        assertThat(reference).hasSize(800);
        assertThat(new HashSet<>(reference)).hasSize(800);
        for (String u : users) {
            assertThat(c.history(u, "group:g")).isEqualTo(reference);
        }
        // per sender, messages keep their own order
        for (int t = 0; t < 8; t++) {
            int last = -1;
            for (String h : reference) {
                String text = h.split("\\|", 3)[2];
                if (text.startsWith("t" + t + "-")) {
                    int n = Integer.parseInt(text.substring(text.indexOf('-') + 1));
                    assertThat(n).isGreaterThan(last);
                    last = n;
                }
            }
        }
    }

    @Test
    void shouldDeliverEveryMessageExactlyOnceWhileUsersConnectAndDisconnect() {
        ChatApi c = chat();

        Concurrently.run(5, t -> {
            if (t == 4) {
                for (int i = 0; i < 300; i++) {
                    c.connect("bob");
                    c.disconnect("bob");
                }
                c.connect("bob");
                return;
            }
            for (int i = 0; i < 200; i++) {
                c.sendDirect("ann", "bob", "m" + t + "-" + i);
            }
        });

        Set<String> ids = new HashSet<>();
        for (String h : c.history("bob", "dm:ann:bob")) {
            ids.add(h.split("\\|", 3)[0]);
        }
        assertThat(ids).hasSize(800);
        for (String id : ids) {
            assertThat(c.statusOf(id)).isEqualTo("DELIVERED");
        }
        assertThat(c.unreadCount("bob", "dm:ann:bob")).isEqualTo(800);
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            ChatApi c = chat();
            c.createGroup("g", "ann", List.of("bob"));

            Concurrently.run(4, t -> {
                for (int i = 0; i < 20; i++) {
                    c.sendGroup(t % 2 == 0 ? "ann" : "bob", "g", "x" + t + i);
                }
            });

            assertThat(c.history("ann", "group:g")).as("run %d", run).hasSize(80).isEqualTo(c.history("bob", "group:g"));
        }
    }
}
