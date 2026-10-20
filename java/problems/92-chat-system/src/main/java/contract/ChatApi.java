package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Chat ids: a direct chat between two users has the id {@code "dm:" + smallerName + ":" + largerName} (names
 * compared as strings); a group chat has the id {@code "group:" + groupId}. IllegalArgumentException for unknown
 * users/groups/messages; SecurityException for actions the user is not allowed to do.
 */
public interface ChatApi {

    void register(String user);

    /** Goes online. Everything waiting for this user is delivered now. */
    void connect(String user);

    /** Goes offline; messages sent from now on wait until the next connect. */
    void disconnect(String user);

    /** Sends a direct message; returns the message id. It is delivered immediately if the recipient is online. */
    String sendDirect(String from, String to, String text);

    /** The creator becomes the group's admin and is a member automatically; the other members are added. */
    void createGroup(String groupId, String admin, List<String> members);

    /** Only the admin. */
    void addMember(String actor, String groupId, String user);

    /** The admin may remove anybody; any member may remove themselves (leave). The admin cannot leave. */
    void removeMember(String actor, String groupId, String user);

    /** Sender must currently be a member (SecurityException otherwise). Returns the message id. */
    String sendGroup(String from, String groupId, String text);

    /**
     * The messages the user took part in (sent, or was a recipient of when they were sent) in this chat, in the order
     * they were sent, formatted "messageId|sender|text".
     */
    List<String> history(String user, String chatId);

    /** Marks every message addressed to the user in the chat as read (and delivered). */
    void read(String user, String chatId);

    /** Messages addressed to the user in the chat that they have not read yet. */
    int unreadCount(String user, String chatId);

    /**
     * Status from the sender's point of view: "SENT" (somebody it is addressed to has not got it yet), "DELIVERED"
     * (all recipients have it) or "READ" (all recipients have read it). A message nobody receives (a group of one)
     * is "READ". Recipients who have left the group since are ignored.
     */
    String statusOf(String messageId);
}
