package contract;

/** Provided subsystem. */
public interface ImageResizer {

    /** Returns the image scaled down so that it is at most {@code maxWidth} pixels wide. */
    String resize(String content, int maxWidth);
}
