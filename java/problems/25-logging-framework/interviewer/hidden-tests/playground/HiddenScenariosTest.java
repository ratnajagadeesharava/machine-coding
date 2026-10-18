package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.LoggingApi;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class HiddenScenariosTest {
    private final FakeClock clock = new FakeClock(42);
    private final List<String> lines = new ArrayList<>();

    private LoggingApi logging() {
        LoggingApi api = Candidate.of(() -> Solution.create(clock));
        api.addAppender("main", lines::add, "TRACE");
        return api;
    }

    @Test
    void aDeepLoggerShouldInheritThroughMissingIntermediateLoggers() {
        LoggingApi api = logging();
        api.setLevel("app", "ERROR");

        api.log("app.db.pool.conn", "WARN", "dropped");
        api.log("app.db.pool.conn", "ERROR", "kept");

        assertThat(lines).containsExactly("42 [ERROR] app.db.pool.conn - kept");
    }

    @Test
    void aSiblingWithACommonPrefixShouldNotInheritFromTheOtherSibling() {
        LoggingApi api = logging();
        api.setLevel("app.db", "ERROR");

        api.log("app.dbx", "INFO", "kept");
        api.log("app.db.pool", "INFO", "dropped");

        assertThat(lines).containsExactly("42 [INFO] app.dbx - kept");
    }

    @Test
    void adjacentPlaceholdersAndTemplatesWithoutPlaceholdersShouldWork() {
        LoggingApi api = logging();

        api.log("app", "INFO", "{}{}{}", "a", "b", "c");
        api.log("app", "INFO", "{}{}", "a");
        api.log("app", "INFO", "plain", "ignored");
        api.log("app", "INFO", "");

        assertThat(lines).containsExactly(
                "42 [INFO] app - abc", "42 [INFO] app - a{}", "42 [INFO] app - plain", "42 [INFO] app - ");
    }

    @Test
    void anArgumentContainingAPlaceholderShouldBeCopiedVerbatim() {
        LoggingApi api = logging();

        api.log("app", "INFO", "{} then {}", "{} {}", "end");

        assertThat(lines).containsExactly("42 [INFO] app - {} {} then end");
    }

    @Test
    void settingTheSameLevelTwiceShouldKeepTheLastAndTheRootNameShouldBeUsable() {
        LoggingApi api = logging();
        api.setLevel("root", "ERROR");
        api.setLevel("root", "TRACE");
        api.setLevel("app", "WARN");
        api.setLevel("app", "DEBUG");

        api.log("app.x", "DEBUG", "kept");
        api.log("other", "TRACE", "kept too");

        assertThat(lines).containsExactly("42 [DEBUG] app.x - kept", "42 [TRACE] other - kept too");
    }
}
