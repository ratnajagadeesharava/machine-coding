package contract;

import java.util.List;
import java.util.Map;

/** Thin test entry point. Your domain model lives behind it. */
public interface NotificationApi {

    void registerUser(String userId, Map<String, String> addressesByChannel);

    /** Ignored for unregistered users. Everything is enabled by default. */
    void setPreference(String userId, String channel, String category, boolean enabled);

    void defineTemplate(String templateId, String body);

    /**
     * Returns UNKNOWN_USER, INVALID, SUPPRESSED, FAILED or DELIVERED:&lt;channel&gt;.
     * Channels are tried in the given order.
     */
    String send(String userId, String category, String templateId, Map<String, String> params, List<String> channels);
}
