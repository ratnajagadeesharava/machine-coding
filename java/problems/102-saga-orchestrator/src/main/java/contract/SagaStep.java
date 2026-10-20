package contract;

/**
 * One local transaction of a saga plus its compensating action (provided by the tests).
 * Both methods report failure by returning {@code false} OR by throwing: the orchestrator treats the two the same.
 * Both must be idempotent per saga id (they can be invoked again after a crash).
 */
public interface SagaStep {

    String name();

    boolean execute(String sagaId);

    boolean compensate(String sagaId);
}
