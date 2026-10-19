package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Backend;
import contract.Gateway;
import contract.Response;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {

    private final FakeClock clock = new FakeClock(1_000_000);

    private Gateway gateway() {
        return Candidate.of(() -> Solution.create(clock));
    }

    private static Backend echo(String name) {
        return path -> name + path;
    }

    private static Backend broken(AtomicBoolean down, String name) {
        return path -> {
            if (down.get()) {
                throw new IllegalStateException(name + " is down");
            }
            return name + path;
        };
    }

    private Gateway simple() {
        Gateway g = gateway();
        g.registerBackend("a", echo("a"));
        g.registerBackend("b", echo("b"));
        g.registerBackend("c", echo("c"));
        g.addRoute("/api", List.of("a", "b", "c"));
        g.addClient("key-1", "acme", 600);
        return g;
    }

    @Test
    void shouldAnswer401ForUnknownKeysAndNotConsumeTokens() {
        Gateway g = simple();

        assertThat(g.handle("nope", "/api/x")).isEqualTo(new Response(401, "unauthorized"));
        assertThat(g.handle(null, "/api/x").status()).isEqualTo(401);
    }

    @Test
    void shouldAnswer404WithoutConsumingATokenWhenNothingMatches() {
        Gateway g = gateway();
        g.registerBackend("a", echo("a"));
        g.addRoute("/api", List.of("a"));
        g.addClient("k", "tiny", 1);

        assertThat(g.handle("k", "/other")).isEqualTo(new Response(404, "no route"));
        assertThat(g.handle("k", "/api/x").status()).isEqualTo(200); // the 404 did not use the only token
    }

    @Test
    void shouldPickTheLongestMatchingPrefix() {
        Gateway g = gateway();
        g.registerBackend("general", echo("general"));
        g.registerBackend("users", echo("users"));
        g.addRoute("/api", List.of("general"));
        g.addRoute("/api/users", List.of("users"));
        g.addClient("k", "c", 100);

        assertThat(g.handle("k", "/api/users/42")).isEqualTo(new Response(200, "users/api/users/42"));
        assertThat(g.handle("k", "/api/orders")).isEqualTo(new Response(200, "general/api/orders"));
    }

    @Test
    void shouldRejectBadRoutes() {
        Gateway g = gateway();
        g.registerBackend("a", echo("a"));

        assertThatThrownBy(() -> g.addRoute("/x", List.of("ghost"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.addRoute("/x", List.of())).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRoundRobinOverTheBackends() {
        Gateway g = simple();

        List<String> bodies = new ArrayList<>();
        for (int i = 0; i < 6; i++) {
            bodies.add(g.handle("key-1", "/api/p").body());
        }

        assertThat(bodies).containsExactly("a/api/p", "b/api/p", "c/api/p", "a/api/p", "b/api/p", "c/api/p");
        assertThat(g.successCount("acme")).isEqualTo(6);
    }

    @Test
    void shouldFailOverToTheNextBackendWhenOneThrows() {
        Gateway g = gateway();
        AtomicBoolean aDown = new AtomicBoolean(true);
        g.registerBackend("a", broken(aDown, "a"));
        g.registerBackend("b", echo("b"));
        g.addRoute("/api", List.of("a", "b"));
        g.addClient("k", "c", 100);

        assertThat(g.handle("k", "/api/1")).isEqualTo(new Response(200, "b/api/1"));

        assertThat(g.callCount("a")).isEqualTo(1);
        assertThat(g.callCount("b")).isEqualTo(1);
    }

    @Test
    void shouldAnswer502WhenEveryBackendFails() {
        Gateway g = gateway();
        AtomicBoolean down = new AtomicBoolean(true);
        g.registerBackend("a", broken(down, "a"));
        g.registerBackend("b", broken(down, "b"));
        g.addRoute("/api", List.of("a", "b"));
        g.addClient("k", "c", 100);

        assertThat(g.handle("k", "/api/1")).isEqualTo(new Response(502, "bad gateway"));
        assertThat(g.successCount("c")).isZero();
    }

    @Test
    void shouldOpenTheCircuitAfterThreeConsecutiveFailuresAndStopCallingTheBackend() {
        Gateway g = gateway();
        AtomicBoolean aDown = new AtomicBoolean(true);
        g.registerBackend("a", broken(aDown, "a"));
        g.registerBackend("b", echo("b"));
        g.addRoute("/api", List.of("a", "b"));
        g.addClient("k", "c", 1_000);

        for (int i = 0; i < 20; i++) {
            assertThat(g.handle("k", "/api/x").status()).isEqualTo(200);
        }

        assertThat(g.callCount("a")).as("a is skipped once its circuit is open").isEqualTo(3);
    }

    @Test
    void shouldLetOneTrialThroughAfterThirtySecondsAndCloseOnSuccess() {
        Gateway g = gateway();
        AtomicBoolean aDown = new AtomicBoolean(true);
        g.registerBackend("a", broken(aDown, "a"));
        g.registerBackend("b", echo("b"));
        g.addRoute("/api", List.of("a", "b"));
        g.addClient("k", "c", 1_000);
        for (int i = 0; i < 10; i++) {
            g.handle("k", "/api/x");
        }
        assertThat(g.callCount("a")).isEqualTo(3);

        clock.advance(29_999);
        g.handle("k", "/api/x");
        g.handle("k", "/api/x");
        assertThat(g.callCount("a")).isEqualTo(3);

        clock.advance(1);
        aDown.set(false);
        g.handle("k", "/api/x"); // round-robin may start at b; two requests make sure a gets its trial
        g.handle("k", "/api/x");
        assertThat(g.callCount("a")).isGreaterThan(3);

        int before = g.callCount("a");
        for (int i = 0; i < 10; i++) {
            g.handle("k", "/api/x");
        }
        assertThat(g.callCount("a")).as("closed again: a takes its share").isGreaterThan(before + 3);
    }

    @Test
    void shouldReopenTheCircuitWhenTheTrialFails() {
        Gateway g = gateway();
        AtomicBoolean aDown = new AtomicBoolean(true);
        g.registerBackend("a", broken(aDown, "a"));
        g.registerBackend("b", echo("b"));
        g.addRoute("/api", List.of("a", "b"));
        g.addClient("k", "c", 1_000);
        for (int i = 0; i < 10; i++) {
            g.handle("k", "/api/x");
        }

        clock.advance(30_000);
        for (int i = 0; i < 4; i++) {
            g.handle("k", "/api/x"); // the trial fails
        }
        int afterTrial = g.callCount("a");
        for (int i = 0; i < 10; i++) {
            g.handle("k", "/api/x");
        }

        assertThat(afterTrial).isEqualTo(4);
        assertThat(g.callCount("a")).as("open again for 30 more seconds").isEqualTo(4);
    }

    @Test
    void shouldAnswer503WithoutCallingAnythingWhenEveryCircuitIsOpen() {
        Gateway g = gateway();
        AtomicBoolean down = new AtomicBoolean(true);
        g.registerBackend("a", broken(down, "a"));
        g.addRoute("/api", List.of("a"));
        g.addClient("k", "c", 1_000);
        for (int i = 0; i < 3; i++) {
            assertThat(g.handle("k", "/api/x").status()).isEqualTo(502);
        }

        Response r = g.handle("k", "/api/x");

        assertThat(r).isEqualTo(new Response(503, "service unavailable"));
        assertThat(g.callCount("a")).isEqualTo(3);
    }

    @Test
    void shouldResetTheFailureCountOnSuccess() {
        Gateway g = gateway();
        AtomicBoolean down = new AtomicBoolean(true);
        g.registerBackend("a", broken(down, "a"));
        g.addRoute("/api", List.of("a"));
        g.addClient("k", "c", 1_000);

        for (int round = 0; round < 5; round++) {
            down.set(true);
            g.handle("k", "/api/1");
            g.handle("k", "/api/2");
            down.set(false);
            assertThat(g.handle("k", "/api/3").status()).isEqualTo(200); // two failures then a success: never three in a row
        }
        assertThat(g.callCount("a")).isEqualTo(15);
    }

    @Test
    void shouldRateLimitPerClientWithATokenBucket() {
        Gateway g = gateway();
        g.registerBackend("a", echo("a"));
        g.addRoute("/api", List.of("a"));
        g.addClient("k1", "slow", 60);
        g.addClient("k2", "other", 60);

        for (int i = 0; i < 60; i++) {
            assertThat(g.handle("k1", "/api/x").status()).isEqualTo(200);
        }
        assertThat(g.handle("k1", "/api/x")).isEqualTo(new Response(429, "rate limited"));
        assertThat(g.handle("k2", "/api/x").status()).isEqualTo(200); // separate bucket

        clock.advance(1_000); // one token per second
        assertThat(g.handle("k1", "/api/x").status()).isEqualTo(200);
        assertThat(g.handle("k1", "/api/x").status()).isEqualTo(429);
        clock.advance(500);
        assertThat(g.handle("k1", "/api/x").status()).isEqualTo(429); // half a token is not enough
        clock.advance(500);
        assertThat(g.handle("k1", "/api/x").status()).isEqualTo(200);
    }

    @Test
    void shouldNeverHoldMoreTokensThanTheCapacity() {
        Gateway g = gateway();
        g.registerBackend("a", echo("a"));
        g.addRoute("/api", List.of("a"));
        g.addClient("k", "c", 10);

        clock.advance(3_600_000);
        int ok = 0;
        for (int i = 0; i < 30; i++) {
            if (g.handle("k", "/api/x").status() == 200) {
                ok++;
            }
        }

        assertThat(ok).isEqualTo(10);
    }

    @Test
    void shouldAdmitExactlyTheCapacityUnderConcurrentRequests() {
        Gateway g = gateway();
        g.registerBackend("a", echo("a"));
        g.addRoute("/api", List.of("a"));
        g.addClient("k", "c", 1_000);
        AtomicInteger ok = new AtomicInteger();
        AtomicInteger limited = new AtomicInteger();

        Concurrently.run(16, t -> {
            for (int i = 0; i < 100; i++) {
                int status = g.handle("k", "/api/x").status();
                (status == 200 ? ok : limited).incrementAndGet();
            }
        });

        assertThat(ok).hasValue(1_000);
        assertThat(limited).hasValue(600);
        assertThat(g.callCount("a")).isEqualTo(1_000);
    }

    @Test
    void shouldSpreadConcurrentRequestsEvenlyOverTheBackends() {
        Gateway g = simple(); // backends a, b, c
        g.registerBackend("d", echo("d"));
        g.addRoute("/all", List.of("a", "b", "c", "d"));
        g.addClient("k", "big", 100_000);

        Concurrently.run(16, t -> {
            for (int i = 0; i < 100; i++) {
                g.handle("k", "/all/x");
            }
        });

        for (String id : List.of("a", "b", "c", "d")) {
            assertThat(g.callCount(id)).as(id).isEqualTo(400);
        }
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            Gateway g = gateway();
            g.registerBackend("a", echo("a"));
            g.addRoute("/api", List.of("a"));
            g.addClient("k", "c", 50);
            AtomicInteger ok = new AtomicInteger();

            Concurrently.run(8, t -> {
                for (int i = 0; i < 20; i++) {
                    if (g.handle("k", "/api/x").status() == 200) {
                        ok.incrementAndGet();
                    }
                }
            });

            assertThat(ok).as("run %d", run).hasValue(50);
            assertThat(g.callCount("a")).as("run %d", run).isEqualTo(50);
        }
    }
}
