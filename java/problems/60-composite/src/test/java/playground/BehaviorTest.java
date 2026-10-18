package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.FsNode;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.DirectoryNode;
import solution.FileNode;

class BehaviorTest {

    /** A node type that exists only in the test: folders must treat it like any other node. */
    private record Shortcut(String name) implements FsNode {
        public long size() { return 4; }
        public int fileCount() { return 1; }
        public List<String> paths() { return List.of(name); }
    }

    private static DirectoryNode sampleTree() {
        DirectoryNode root = new DirectoryNode("root");
        root.add(new FileNode("a.txt", 100));
        DirectoryNode docs = new DirectoryNode("docs");
        docs.add(new FileNode("b.txt", 250));
        DirectoryNode deep = new DirectoryNode("deep");
        deep.add(new FileNode("c.txt", 50));
        docs.add(deep);
        root.add(docs);
        root.add(new DirectoryNode("empty"));
        return root;
    }

    @Test
    void shouldTreatAFileAsASingleNode() {
        FsNode file = new FileNode("a.txt", 100);

        assertThat(file.name()).isEqualTo("a.txt");
        assertThat(file.size()).isEqualTo(100);
        assertThat(file.fileCount()).isEqualTo(1);
        assertThat(file.paths()).containsExactly("a.txt");
    }

    @Test
    void shouldSumSizesAndCountFilesRecursively() {
        FsNode root = sampleTree();

        assertThat(root.size()).isEqualTo(400);
        assertThat(root.fileCount()).isEqualTo(3);
        FsNode docs = ((DirectoryNode) root).find("docs").orElseThrow();
        assertThat(docs.size()).isEqualTo(300);
        assertThat(docs.fileCount()).isEqualTo(2);
    }

    @Test
    void shouldListEveryFileWithItsPathInInsertionOrder() {
        assertThat(sampleTree().paths()).containsExactly("root/a.txt", "root/docs/b.txt", "root/docs/deep/c.txt");
        assertThat(new DirectoryNode("nothing").paths()).isEmpty();
        assertThat(new DirectoryNode("nothing").size()).isZero();
    }

    @Test
    void shouldFindNodesByRelativePath() {
        DirectoryNode root = sampleTree();

        assertThat(root.find("a.txt").orElseThrow().size()).isEqualTo(100);
        assertThat(root.find("docs/deep/c.txt").orElseThrow().name()).isEqualTo("c.txt");
        assertThat(root.find("docs/deep").orElseThrow().fileCount()).isEqualTo(1);
        assertThat(root.find("docs/missing.txt")).isEmpty();
        assertThat(root.find("a.txt/inside")).isEmpty();
        assertThatThrownBy(() -> root.find(" ")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldAcceptNodeTypesItHasNeverSeen() {
        DirectoryNode root = sampleTree();

        root.add(new Shortcut("link"));

        assertThat(root.size()).isEqualTo(404);
        assertThat(root.fileCount()).isEqualTo(4);
        assertThat(root.paths()).endsWith("root/link");
        assertThat(root.find("link")).isPresent();
    }

    @Test
    void shouldRemoveChildrenByName() {
        DirectoryNode root = sampleTree();

        assertThat(root.remove("docs")).isTrue();
        assertThat(root.remove("docs")).isFalse();

        assertThat(root.size()).isEqualTo(100);
        assertThat(root.children()).extracting(FsNode::name).containsExactly("a.txt", "empty");
    }

    @Test
    void shouldRefuseDuplicateNamesAndCycles() {
        DirectoryNode root = sampleTree();
        DirectoryNode docs = (DirectoryNode) root.find("docs").orElseThrow();
        DirectoryNode deep = (DirectoryNode) root.find("docs/deep").orElseThrow();

        assertThatThrownBy(() -> root.add(new FileNode("a.txt", 1))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> root.add(root)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> deep.add(root)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> deep.add(docs)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> root.add(null)).isInstanceOf(IllegalArgumentException.class);

        assertThat(root.fileCount()).isEqualTo(3);
        assertThat(deep.children()).hasSize(1);
    }

    @Test
    void shouldValidateNamesAndSizes() {
        assertThatThrownBy(() -> new FileNode("", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new FileNode("a/b", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new FileNode("ok", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new DirectoryNode(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new DirectoryNode("x/y")).isInstanceOf(IllegalArgumentException.class);
        assertThat(new FileNode("empty", 0).size()).isZero();
    }

    @Test
    void shouldNotLetCallersChangeAFolderThroughItsChildrenList() {
        DirectoryNode root = sampleTree();

        assertThat(root.children()).hasSize(3);
        assertThatThrownBy(() -> root.children().clear()).isInstanceOf(UnsupportedOperationException.class);
        assertThat(root.children()).hasSize(3);
    }
}
