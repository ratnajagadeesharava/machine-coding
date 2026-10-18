package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Aircraft;
import contract.Tower;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.ControlTower;
import solution.PassengerJet;

class BehaviorTest {

    private final List<String> calls = new ArrayList<>();

    /** An aircraft written in the test: records every instruction it is given. */
    private Aircraft plane(String id) {
        return new Aircraft() {
            public String id() { return id; }
            public void onClearedToLand() { calls.add(id + ":CLEARED"); }
            public void onHold(int position) { calls.add(id + ":HOLD " + position); }
        };
    }

    @Test
    void shouldClearTheFirstAircraftAtOnceWhenTheRunwayIsFree() {
        ControlTower tower = new ControlTower();

        tower.requestLanding(plane("A"));

        assertThat(calls).containsExactly("A:CLEARED");
        assertThat(tower.onRunway()).hasValue("A");
        assertThat(tower.queue()).isEmpty();
    }

    @Test
    void shouldQueueLaterAircraftAndTellThemTheirPosition() {
        ControlTower tower = new ControlTower();
        tower.requestLanding(plane("A"));
        tower.requestLanding(plane("B"));
        tower.requestLanding(plane("C"));

        assertThat(calls).containsExactly("A:CLEARED", "B:HOLD 1", "C:HOLD 2");
        assertThat(tower.queue()).containsExactly("B", "C");
    }

    @Test
    void shouldClearTheNextAircraftWhenTheRunwayIsFreedAndRenumberTheRest() {
        ControlTower tower = new ControlTower();
        Aircraft a = plane("A");
        tower.requestLanding(a);
        tower.requestLanding(plane("B"));
        tower.requestLanding(plane("C"));
        tower.requestLanding(plane("D"));
        calls.clear();

        tower.landed(a);

        assertThat(calls).containsExactly("B:CLEARED", "C:HOLD 1", "D:HOLD 2");
        assertThat(tower.onRunway()).hasValue("B");
        assertThat(tower.queue()).containsExactly("C", "D");
    }

    @Test
    void shouldFreeTheRunwayWhenTheClearedAircraftDiverts() {
        ControlTower tower = new ControlTower();
        Aircraft a = plane("A");
        tower.requestLanding(a);
        tower.requestLanding(plane("B"));
        calls.clear();

        assertThat(tower.cancel(a)).isTrue();

        assertThat(calls).containsExactly("B:CLEARED");
        assertThat(tower.onRunway()).hasValue("B");
        assertThat(tower.cancel(a)).isFalse();
    }

    @Test
    void shouldRemoveAWaitingAircraftAndRenumberOnlyThoseBehindIt() {
        ControlTower tower = new ControlTower();
        tower.requestLanding(plane("A"));
        Aircraft b = plane("B");
        tower.requestLanding(b);
        tower.requestLanding(plane("C"));
        tower.requestLanding(plane("D"));
        calls.clear();

        assertThat(tower.cancel(b)).isTrue();

        assertThat(calls).containsExactly("C:HOLD 1", "D:HOLD 2");
        assertThat(tower.queue()).containsExactly("C", "D");
    }

    @Test
    void shouldLetAnEmergencyOvertakeTheQueue() {
        ControlTower tower = new ControlTower();
        tower.requestLanding(plane("A"));
        tower.requestLanding(plane("B"));
        tower.requestLanding(plane("C"));
        Aircraft d = plane("D");
        tower.requestLanding(d);
        calls.clear();

        tower.declareEmergency(d);

        assertThat(calls).containsExactly("D:HOLD 1", "B:HOLD 2", "C:HOLD 3");
        assertThat(tower.queue()).containsExactly("D", "B", "C");
        calls.clear();
        tower.declareEmergency(d);
        assertThat(calls).isEmpty();
    }

    @Test
    void shouldRejectRequestsThatMakeNoSense() {
        ControlTower tower = new ControlTower();
        Aircraft a = plane("A");
        Aircraft b = plane("B");
        tower.requestLanding(a);
        tower.requestLanding(b);

        assertThatThrownBy(() -> tower.requestLanding(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> tower.requestLanding(plane("A"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> tower.requestLanding(b)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> tower.landed(b)).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> tower.landed(plane("Z"))).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> tower.declareEmergency(a)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> tower.declareEmergency(plane("Z"))).isInstanceOf(IllegalArgumentException.class);
        assertThat(tower.onRunway()).hasValue("A");
        assertThat(tower.queue()).containsExactly("B");
    }

    @Test
    void shouldReturnToAnEmptyAirspaceWhenEveryoneHasLanded() {
        ControlTower tower = new ControlTower();
        Aircraft a = plane("A");
        Aircraft b = plane("B");
        tower.requestLanding(a);
        tower.requestLanding(b);

        tower.landed(a);
        tower.landed(b);

        assertThat(tower.onRunway()).isEmpty();
        assertThat(tower.queue()).isEmpty();
        tower.requestLanding(plane("C"));
        assertThat(calls).endsWith("C:CLEARED");
    }

    @Test
    void shouldLetRealJetsTalkOnlyToTheTower() {
        Tower tower = new ControlTower();
        PassengerJet first = new PassengerJet("QF1", tower);
        PassengerJet second = new PassengerJet("QF2", tower);
        PassengerJet third = new PassengerJet("QF3", tower);
        assertThat(first.status()).isEqualTo("AIRBORNE");

        first.requestLanding();
        second.requestLanding();
        third.requestLanding();
        assertThat(first.status()).isEqualTo("CLEARED");
        assertThat(second.status()).isEqualTo("HOLDING #1");
        assertThat(third.status()).isEqualTo("HOLDING #2");

        first.touchdown();
        assertThat(first.status()).isEqualTo("LANDED");
        assertThat(second.status()).isEqualTo("CLEARED");
        assertThat(third.status()).isEqualTo("HOLDING #1");

        second.divert();
        assertThat(second.status()).isEqualTo("DIVERTED");
        assertThat(third.status()).isEqualTo("CLEARED");
        assertThat(tower.onRunway()).hasValue("QF3");
    }

    @Test
    void shouldNeverLetAircraftHoldReferencesToEachOther() {
        for (Field field : PassengerJet.class.getDeclaredFields()) {
            assertThat(Aircraft.class.isAssignableFrom(field.getType()))
                    .as("field " + field.getName() + " must not point at another aircraft")
                    .isFalse();
            assertThat(PassengerJet.class.isAssignableFrom(field.getType())).isFalse();
        }
        assertThat(PassengerJet.class.getDeclaredFields()).isNotEmpty();
        assertThat(new PassengerJet("X", new ControlTower()).id()).isEqualTo("X");
    }
}
