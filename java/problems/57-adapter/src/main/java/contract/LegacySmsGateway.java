package contract;

/**
 * Provided and NOT to be changed: the vendor's old SMS gateway (the "adaptee"). It speaks XML and integer codes.
 * Return codes: 0 = accepted, 1 = invalid number, 2 = message too long, 3 = unauthorized, anything else = unknown.
 */
public interface LegacySmsGateway {

    int submitXml(String xml);
}
