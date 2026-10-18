package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.CoffeeMachineApi;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private CoffeeMachineApi machine() {
        return Candidate.of(Solution::create);
    }

    @Test
    void shouldReportAlphabeticallyFirstMissingIngredient() {
        CoffeeMachineApi m = machine();
        m.addRecipe("x", Map.of("zinc", 1, "apple", 1, "mango", 1));
        m.refill("mango", 5);

        assertThat(m.dispense("x")).isEqualTo("INSUFFICIENT:apple");
    }

    @Test
    void shouldTreatNeverRefilledIngredientAsEmpty() {
        CoffeeMachineApi m = machine();
        m.refill("water", 10);
        m.addRecipe("x", Map.of("water", 1, "ghost", 1));

        assertThat(m.dispense("x")).isEqualTo("INSUFFICIENT:ghost");
        assertThat(m.stock("water")).isEqualTo(10);
    }

    @Test
    void lowStockShouldIncludeFullyConsumedIngredients() {
        CoffeeMachineApi m = machine();
        m.refill("water", 10);
        m.addRecipe("x", Map.of("water", 10));
        m.dispense("x");

        assertThat(m.lowStock(1)).containsExactly("water");
    }

    @Test
    void replacingARecipeShouldAffectNextDispenseOnly() {
        CoffeeMachineApi m = machine();
        m.refill("a", 10);
        m.refill("b", 10);
        m.addRecipe("d", Map.of("a", 1));
        m.dispense("d");
        m.addRecipe("d", Map.of("b", 2));
        m.dispense("d");

        assertThat(m.stock("a")).isEqualTo(9);
        assertThat(m.stock("b")).isEqualTo(8);
    }
}
