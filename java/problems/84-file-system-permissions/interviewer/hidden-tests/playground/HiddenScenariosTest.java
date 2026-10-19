package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.FileSystemApi;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    @Test
    void shouldHandleADeepTree() {
        FileSystemApi fs = Candidate.of(Solution::create);
        fs.chmod("root", "/", 0777);
        StringBuilder path = new StringBuilder();
        for (int i = 0; i < 500; i++) {
            path.append("/d").append(i);
            fs.mkdir("root", path.toString());
        }
        fs.createFile("root", path + "/leaf", "deep");

        assertThat(fs.read("root", path + "/leaf")).isEqualTo("deep");
    }

    @Test
    void shouldPickTheFirstMatchingClassNotTheMostPermissiveOne() {
        FileSystemApi fs = Candidate.of(Solution::create);
        fs.addUser("alice", List.of("dev"));
        fs.addUser("bob", List.of("dev"));
        fs.chmod("root", "/", 0777);
        fs.createFile("alice", "/f", "x");
        fs.chmod("alice", "/f", 0407); // owner r--, group ---, other rwx

        assertThat(fs.read("alice", "/f")).isEqualTo("x");
        assertThatThrownBy(() -> fs.read("bob", "/f")).isInstanceOf(SecurityException.class);
    }
}
