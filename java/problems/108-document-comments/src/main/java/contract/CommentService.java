package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface CommentService {

    /**
     * Appends a comment to an existing document and returns its id: "1", "2", ... (one sequence for the whole service,
     * in accepted order; a rejected call uses no id).
     * IllegalArgumentException if documentId, reviewerId or text is blank (checked first) or the document is unknown.
     */
    String addComment(String documentId, String reviewerId, String text);

    /**
     * All comments of the document, oldest first, each as reviewerId + ": " + text (verbatim). Empty if there are none.
     * The list is a copy. IllegalArgumentException if documentId is blank or unknown.
     */
    List<String> comments(String documentId);
}
