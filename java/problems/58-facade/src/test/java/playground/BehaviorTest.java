package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ImageResizer;
import contract.PhotoStorage;
import contract.SearchIndex;
import contract.UploadResult;
import contract.VirusScanner;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.PhotoUploadFacade;

class BehaviorTest {

    private final List<String> calls = new ArrayList<>();
    private boolean clean = true;
    private boolean storageFails;
    private boolean indexFails;

    private final VirusScanner scanner = content -> {
        calls.add("scan:" + content);
        return clean;
    };

    private final ImageResizer resizer = (content, maxWidth) -> {
        calls.add("resize:" + maxWidth);
        return content + "@" + maxWidth;
    };

    private final PhotoStorage storage = new PhotoStorage() {
        public String store(String fileName, String content) {
            calls.add("store:" + fileName + ":" + content);
            if (storageFails) throw new IllegalStateException("disk full");
            return "https://cdn.test/" + fileName;
        }

        public void delete(String url) {
            calls.add("delete:" + url);
        }
    };

    private final SearchIndex index = (url, fileName) -> {
        calls.add("index:" + url + ":" + fileName);
        if (indexFails) throw new IllegalStateException("cluster down");
    };

    private PhotoUploadFacade facade() {
        return new PhotoUploadFacade(scanner, resizer, storage, index);
    }

    @Test
    void shouldRunTheWholeWorkflowInOrderBehindOneCall() {
        UploadResult result = facade().upload("cat.jpg", "PIXELS");

        assertThat(result).isEqualTo(new UploadResult("OK", "https://cdn.test/cat.jpg"));
        assertThat(calls).containsExactly(
                "scan:PIXELS", "resize:1024", "store:cat.jpg:PIXELS@1024",
                "index:https://cdn.test/cat.jpg:cat.jpg");
    }

    @Test
    void shouldStoreTheResizedImageNotTheOriginal() {
        facade().upload("dog.PNG", "RAW");

        assertThat(calls).contains("store:dog.PNG:RAW@1024");
    }

    @Test
    void shouldRejectAnInfectedFileAfterScanningOnly() {
        clean = false;

        UploadResult result = facade().upload("bad.jpg", "VIRUS");

        assertThat(result).isEqualTo(new UploadResult("REJECTED_VIRUS", ""));
        assertThat(calls).containsExactly("scan:VIRUS");
    }

    @Test
    void shouldRejectInvalidInputWithoutCallingAnySubsystem() {
        PhotoUploadFacade facade = facade();

        assertThat(facade.upload("notes.txt", "x")).isEqualTo(new UploadResult("REJECTED_INVALID", ""));
        assertThat(facade.upload(" ", "x")).isEqualTo(new UploadResult("REJECTED_INVALID", ""));
        assertThat(facade.upload(null, "x")).isEqualTo(new UploadResult("REJECTED_INVALID", ""));
        assertThat(facade.upload("a.jpg", "")).isEqualTo(new UploadResult("REJECTED_INVALID", ""));
        assertThat(facade.upload("a.jpg", null)).isEqualTo(new UploadResult("REJECTED_INVALID", ""));

        assertThat(calls).isEmpty();
    }

    @Test
    void shouldReportAStorageFailureAndNotIndexAnything() {
        storageFails = true;

        UploadResult result = facade().upload("a.jpg", "x");

        assertThat(result).isEqualTo(new UploadResult("FAILED_STORAGE", ""));
        assertThat(calls).containsExactly("scan:x", "resize:1024", "store:a.jpg:x@1024");
    }

    @Test
    void shouldUndoTheStorageWhenIndexingFails() {
        indexFails = true;

        UploadResult result = facade().upload("a.jpg", "x");

        assertThat(result).isEqualTo(new UploadResult("FAILED_INDEXING", ""));
        assertThat(calls).endsWith("index:https://cdn.test/a.jpg:a.jpg", "delete:https://cdn.test/a.jpg");
    }

    @Test
    void shouldBeReusableForManyUploads() {
        PhotoUploadFacade facade = facade();

        assertThat(facade.upload("one.jpg", "1").status()).isEqualTo("OK");
        clean = false;
        assertThat(facade.upload("two.jpg", "2").status()).isEqualTo("REJECTED_VIRUS");
        clean = true;
        assertThat(facade.upload("three.png", "3").url()).isEqualTo("https://cdn.test/three.png");
    }
}
