package solution;

import contract.ImageResizer;
import contract.PhotoStorage;
import contract.SearchIndex;
import contract.UploadResult;
import contract.VirusScanner;

/**
 * The facade: ONE simple method hides a four-step workflow across four subsystems. Callers never touch the
 * scanner, resizer, storage or index directly.
 */
public class PhotoUploadFacade {

    public PhotoUploadFacade(VirusScanner scanner, ImageResizer resizer, PhotoStorage storage, SearchIndex index) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Runs the whole workflow and reports one status:
     * <ol>
     *   <li>file name must be non-blank and end with ".jpg" or ".png" (any case), content non-empty, otherwise
     *       {@code ("REJECTED_INVALID", "")} and NO subsystem is called;</li>
     *   <li>{@code scanner.isClean(content)} false -> {@code ("REJECTED_VIRUS", "")}; nothing else is called;</li>
     *   <li>{@code resizer.resize(content, 1024)};</li>
     *   <li>{@code storage.store(fileName, resized)}; if it throws -> {@code ("FAILED_STORAGE", "")}, nothing is indexed;</li>
     *   <li>{@code index.index(url, fileName)}; if it throws -> the stored file is removed with
     *       {@code storage.delete(url)} and the result is {@code ("FAILED_INDEXING", "")};</li>
     *   <li>success -> {@code ("OK", url)}.</li>
     * </ol>
     */
    public UploadResult upload(String fileName, String content) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}
