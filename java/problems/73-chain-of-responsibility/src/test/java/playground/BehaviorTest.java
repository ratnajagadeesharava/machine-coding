package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.ExpenseRequest;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import org.junit.jupiter.api.Test;
import solution.ApprovalHandler;
import solution.Director;
import solution.Manager;
import solution.PolicyGuard;
import solution.TeamLead;

class BehaviorTest {

    private static ExpenseRequest claim(long cents) {
        return new ExpenseRequest("ann", cents, "meals");
    }

    /** A handler written in the test: logs every request it sees and never decides. */
    private static final class Audit extends ApprovalHandler {
        final List<String> seen = new ArrayList<>();

        @Override protected Optional<String> decide(ExpenseRequest request) {
            seen.add(request.employee() + ":" + request.amountCents());
            return Optional.empty();
        }
    }

    private static ApprovalHandler fullChain() {
        TeamLead head = new TeamLead();
        head.setNext(new Manager()).setNext(new Director());
        return head;
    }

    @Test
    void shouldLetTheTeamLeadApproveSmallClaimsUpToTheLimit() {
        TeamLead lead = new TeamLead();

        assertThat(lead.handle(claim(1))).isEqualTo("APPROVED:team lead");
        assertThat(lead.handle(claim(50_000))).isEqualTo("APPROVED:team lead");
        assertThat(lead.handle(claim(50_001))).isEqualTo("REJECTED:no approver for 50001");
    }

    @Test
    void shouldPassClaimsUpTheChainUntilSomeoneCanApprove() {
        ApprovalHandler chain = fullChain();

        assertThat(chain.handle(claim(50_000))).isEqualTo("APPROVED:team lead");
        assertThat(chain.handle(claim(50_001))).isEqualTo("APPROVED:manager");
        assertThat(chain.handle(claim(500_000))).isEqualTo("APPROVED:manager");
        assertThat(chain.handle(claim(500_001))).isEqualTo("APPROVED:director");
        assertThat(chain.handle(claim(5_000_000))).isEqualTo("APPROVED:director");
        assertThat(chain.handle(claim(5_000_001))).isEqualTo("REJECTED:no approver for 5000001");
    }

    @Test
    void shouldSendTravelClaimsPastTheTeamLeadEvenWhenTheyAreSmall() {
        ApprovalHandler chain = fullChain();

        assertThat(chain.handle(new ExpenseRequest("ann", 10_000, "Travel"))).isEqualTo("APPROVED:manager");
        assertThat(chain.handle(new ExpenseRequest("ann", 10_000, "meals"))).isEqualTo("APPROVED:team lead");
    }

    @Test
    void shouldLetTheOrderOfTheChainChangeWhoAnswers() {
        Director first = new Director();
        first.setNext(new TeamLead());

        assertThat(first.handle(claim(100))).isEqualTo("APPROVED:director");
    }

    @Test
    void shouldStopAtTheGuardAndNeverAskTheRestOfTheChain() {
        PolicyGuard guard = new PolicyGuard();
        Audit audit = new Audit();
        guard.setNext(audit).setNext(new TeamLead());

        assertThat(guard.handle(new ExpenseRequest("ann", 100, "Gambling"))).isEqualTo("REJECTED:policy violation");
        assertThat(guard.handle(claim(0))).isEqualTo("REJECTED:policy violation");
        assertThat(guard.handle(claim(-5))).isEqualTo("REJECTED:policy violation");
        assertThat(audit.seen).isEmpty();
        assertThat(guard.handle(claim(100))).isEqualTo("APPROVED:team lead");
        assertThat(audit.seen).containsExactly("ann:100");
    }

    @Test
    void shouldAcceptAHandlerItHasNeverSeenAnywhereInTheChain() {
        Audit audit = new Audit();
        audit.setNext(new Manager()).setNext(new Director());

        assertThat(audit.handle(new ExpenseRequest("bob", 400_000, "gear"))).isEqualTo("APPROVED:manager");
        assertThat(audit.handle(new ExpenseRequest("cy", 9_000_000, "gear"))).isEqualTo("REJECTED:no approver for 9000000");
        assertThat(audit.seen).containsExactly("bob:400000", "cy:9000000");
    }

    @Test
    void shouldReturnTheNextHandlerFromSetNextSoChainsReadLeftToRight() {
        TeamLead lead = new TeamLead();
        Manager manager = new Manager();
        Director director = new Director();

        assertThat(lead.setNext(manager)).isSameAs(manager);
        assertThat(manager.setNext(director)).isSameAs(director);
        assertThat(lead.handle(claim(600_000))).isEqualTo("APPROVED:director");
    }

    @Test
    void shouldReplaceTheNextHandlerWhenLinkedAgain() {
        TeamLead lead = new TeamLead();
        lead.setNext(new Manager());
        assertThat(lead.handle(claim(60_000))).isEqualTo("APPROVED:manager");

        lead.setNext(new Director());

        assertThat(lead.handle(claim(60_000))).isEqualTo("APPROVED:director");
    }

    @Test
    void shouldRejectBadInput() {
        TeamLead lead = new TeamLead();

        assertThatThrownBy(() -> lead.handle(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> lead.setNext(null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(lead.handle(claim(10))).isEqualTo("APPROVED:team lead");
    }

    @Test
    void shouldFixHowTheChainIsWalkedAndLeaveOnlyTheDecisionOpen() throws Exception {
        assertThat(new Manager().handle(claim(10))).isEqualTo("APPROVED:manager");

        Method handle = ApprovalHandler.class.getMethod("handle", ExpenseRequest.class);
        Method decide = ApprovalHandler.class.getDeclaredMethod("decide", ExpenseRequest.class);
        assertThat(Modifier.isAbstract(ApprovalHandler.class.getModifiers())).isTrue();
        assertThat(Modifier.isFinal(handle.getModifiers())).as("handle must be final").isTrue();
        assertThat(Modifier.isAbstract(decide.getModifiers()) && Modifier.isProtected(decide.getModifiers()))
                .as("decide is protected abstract").isTrue();
    }
}
