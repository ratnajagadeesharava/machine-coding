package contract;

/** Provided by the caller (external system). Do not modify. */
public interface DeliveryGateway {

    /** One delivery attempt. True when accepted, false when refused. */
    boolean deliver(String channel, String address, String body);
}
