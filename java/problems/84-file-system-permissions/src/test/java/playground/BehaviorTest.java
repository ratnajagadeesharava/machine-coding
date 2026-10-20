package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.FileSystemApi;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private FileSystemApi fs() {
        FileSystemApi fs = Candidate.of(Solution::create);
        fs.addUser("alice", List.of("dev", "ops"));
        fs.addUser("bob", List.of("dev"));
        fs.addUser("carol", List.of("hr"));
        return fs;
    }

    private static void denied(Runnable r) {
        assertThatThrownBy(r::run).isInstanceOf(SecurityException.class);
    }

    private static void invalid(Runnable r) {
        assertThatThrownBy(r::run).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldCreateReadAndListFilesAndDirectories() {
        FileSystemApi fs = fs();
        fs.mkdir("root", "/home");
        fs.chmod("root", "/home", 0777);

        fs.mkdir("alice", "/home/alice");
        fs.createFile("alice", "/home/alice/b.txt", "bee");
        fs.createFile("alice", "/home/alice/a.txt", "ay");

        assertThat(fs.list("alice", "/home/alice")).containsExactly("a.txt", "b.txt");
        assertThat(fs.read("alice", "/home/alice/a.txt")).isEqualTo("ay");
        assertThat(fs.list("alice", "/")).containsExactly("home");
    }

    @Test
    void shouldGiveNewEntriesTheCreatorsOwnershipAndDefaultModes() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);

        fs.mkdir("alice", "/d");
        fs.createFile("alice", "/f", "x");

        assertThat(fs.owner("/d")).isEqualTo("alice");
        assertThat(fs.group("/d")).isEqualTo("dev");
        assertThat(fs.mode("/d")).isEqualTo(0755);
        assertThat(fs.mode("/f")).isEqualTo(0644);
    }

    @Test
    void shouldRejectBadPathsMissingEntriesAndDuplicates() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);
        fs.createFile("alice", "/f", "x");

        invalid(() -> fs.mkdir("alice", "relative/path"));
        invalid(() -> fs.mkdir("alice", "/a/../b"));
        invalid(() -> fs.mkdir("alice", "/a//b"));
        invalid(() -> fs.mkdir("alice", "/missing/child"));
        invalid(() -> fs.read("alice", "/missing"));
        invalid(() -> fs.createFile("alice", "/f", "again"));
        invalid(() -> fs.mkdir("alice", "/f/sub"));
        invalid(() -> fs.list("alice", "/f"));
        invalid(() -> fs.read("alice", "/"));
        invalid(() -> fs.read("nobody", "/f"));
    }

    @Test
    void shouldUseOwnerBitsForTheOwnerGroupBitsForGroupMembersAndOtherBitsForTheRest() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);
        fs.createFile("alice", "/f", "secret");
        fs.chmod("alice", "/f", 0640); // rw- r-- ---

        fs.write("alice", "/f", "secret2");
        assertThat(fs.read("bob", "/f")).isEqualTo("secret2"); // bob shares group "dev"
        denied(() -> fs.write("bob", "/f", "x"));
        denied(() -> fs.read("carol", "/f"));
        denied(() -> fs.write("carol", "/f", "x"));
    }

    @Test
    void shouldNotFallBackToGroupBitsWhenTheOwnerBitsAreMoreRestrictive() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);
        fs.createFile("alice", "/f", "x");
        fs.chmod("alice", "/f", 0077); // owner has nothing, group and others everything

        denied(() -> fs.read("alice", "/f"));
        assertThat(fs.read("bob", "/f")).isEqualTo("x");
        assertThat(fs.read("carol", "/f")).isEqualTo("x");
    }

    @Test
    void shouldLetRootDoAnything() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);
        fs.createFile("alice", "/f", "x");
        fs.chmod("alice", "/f", 0000);

        assertThat(fs.read("root", "/f")).isEqualTo("x");
        fs.write("root", "/f", "y");
        fs.chown("root", "/f", "carol", "hr");
        assertThat(fs.owner("/f")).isEqualTo("carol");
        assertThat(fs.group("/f")).isEqualTo("hr");
    }

    @Test
    void shouldRequireExecuteOnEveryDirectoryOnThePath() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);
        fs.mkdir("alice", "/a");
        fs.mkdir("alice", "/a/b");
        fs.createFile("alice", "/a/b/f", "x");
        fs.chmod("alice", "/a", 0744); // others: r-- on /a (no x)

        assertThat(fs.read("alice", "/a/b/f")).isEqualTo("x");
        denied(() -> fs.read("carol", "/a/b/f"));
        assertThat(fs.list("carol", "/a")).containsExactly("b"); // r on /a is enough to list
        denied(() -> fs.list("carol", "/a/b"));
    }

    @Test
    void shouldRequireWriteAndExecuteOnTheParentToCreateOrDelete() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);
        fs.mkdir("alice", "/shared");
        fs.createFile("alice", "/shared/f", "x");

        denied(() -> fs.createFile("carol", "/shared/g", "x")); // others: r-x
        denied(() -> fs.delete("carol", "/shared/f"));
        denied(() -> fs.mkdir("bob", "/shared/sub")); // group dev has r-x, no w

        fs.chmod("alice", "/shared", 0775);
        fs.createFile("bob", "/shared/g", "x");
        fs.delete("bob", "/shared/f");
        assertThat(fs.list("alice", "/shared")).containsExactly("g");
    }

    @Test
    void shouldOnlyDeleteEmptyDirectories() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);
        fs.mkdir("alice", "/d");
        fs.createFile("alice", "/d/f", "x");

        invalid(() -> fs.delete("alice", "/d"));
        fs.delete("alice", "/d/f");
        fs.delete("alice", "/d");
        assertThat(fs.list("alice", "/")).isEmpty();
        invalid(() -> fs.delete("alice", "/"));
    }

    @Test
    void shouldRestrictChmodToOwnerAndChownToRoot() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);
        fs.createFile("alice", "/f", "x");

        denied(() -> fs.chmod("bob", "/f", 0777));
        denied(() -> fs.chown("alice", "/f", "bob", "dev"));
        fs.chmod("alice", "/f", 0600);
        assertThat(fs.mode("/f")).isEqualTo(0600);
        invalid(() -> fs.chmod("alice", "/f", 01000));
    }

    @Test
    void shouldMoveFilesAndDirectories() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);
        fs.mkdir("alice", "/src");
        fs.mkdir("alice", "/dst");
        fs.mkdir("alice", "/src/inner");
        fs.createFile("alice", "/src/inner/f", "x");

        fs.move("alice", "/src/inner", "/dst/moved");

        assertThat(fs.read("alice", "/dst/moved/f")).isEqualTo("x");
        assertThat(fs.list("alice", "/src")).isEmpty();
        invalid(() -> fs.move("alice", "/dst", "/dst/moved/loop")); // into its own subtree
        fs.createFile("alice", "/x", "1");
        invalid(() -> fs.move("alice", "/x", "/dst")); // target exists
        fs.mkdir("alice", "/ro");
        fs.createFile("alice", "/ro/f", "1");
        denied(() -> fs.move("carol", "/ro/f", "/y")); // carol cannot remove entries from /ro
    }

    @Test
    void shouldLetExactlyOneThreadCreateTheSamePath() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);

        List<Boolean> wins = Concurrently.collect(16, t -> {
            try {
                fs.createFile("alice", "/contested", "by " + t);
                return true;
            } catch (IllegalArgumentException e) {
                return false;
            }
        });

        assertThat(wins.stream().filter(w -> w).count()).isEqualTo(1);
    }

    @Test
    void shouldKeepAllEntriesWhenManyThreadsCreateFilesInTheSameDirectory() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);
        fs.mkdir("alice", "/d");
        fs.chmod("alice", "/d", 0777);

        Concurrently.run(8, t -> {
            for (int i = 0; i < 250; i++) {
                fs.createFile(t % 2 == 0 ? "alice" : "bob", "/d/f-" + t + "-" + i, "x");
            }
        });

        assertThat(fs.list("alice", "/d")).hasSize(2_000);
    }

    @Test
    void shouldSurviveConcurrentCreateAndDeleteCycles() {
        FileSystemApi fs = fs();
        fs.chmod("root", "/", 0777);
        AtomicInteger cycles = new AtomicInteger();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 300; i++) {
                String dir = "/w" + t;
                fs.mkdir("alice", dir);
                fs.createFile("alice", dir + "/f", "x");
                fs.move("alice", dir + "/f", dir + "/g");
                fs.delete("alice", dir + "/g");
                fs.delete("alice", dir);
                cycles.incrementAndGet();
            }
        });

        assertThat(cycles).hasValue(2_400);
        assertThat(fs.list("alice", "/")).isEmpty();
    }
}
