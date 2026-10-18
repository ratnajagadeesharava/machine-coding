package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.CoffeeMachineApi;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private CoffeeMachineApi machine() {
        return Candidate.of(Solution::create);
    }

    @Test
    void shouldDispenseDrinkAndConsumeIngredients() {
        CoffeeMachineApi m = machine();
        m.refill("water", 100);
        m.refill("coffee", 10);
        m.addRecipe("espresso", Map.of("water", 30, "coffee", 2));

        assertThat(m.dispense("espresso")).isEqualTo("OK");

        assertThat(m.stock("water")).isEqualTo(70);
        assertThat(m.stock("coffee")).isEqualTo(8);
    }

    @Test
    void shouldRejectUnknownDrink() {
        CoffeeMachineApi m = machine();
        assertThat(m.dispense("tea")).isEqualTo("UNKNOWN_DRINK");
    }

    @Test
    void shouldReportMissingIngredient() {
        CoffeeMachineApi m = machine();
        m.refill("water", 100);
        m.refill("coffee", 10);
        m.refill("milk", 2);
        m.addRecipe("latte", Map.of("water", 30, "coffee", 2, "milk", 5));

        assertThat(m.dispense("latte")).isEqualTo("INSUFFICIENT:milk");
    }

    @Test
    void shouldNotConsumeAnythingWhenDispenseFails() {
        CoffeeMachineApi m = machine();
        m.refill("water", 100);
        m.refill("coffee", 10);
        m.refill("milk", 2);
        m.addRecipe("latte", Map.of("water", 30, "coffee", 2, "milk", 5));

        m.dispense("latte");

        assertThat(m.stock("water")).isEqualTo(100);
        assertThat(m.stock("coffee")).isEqualTo(10);
        assertThat(m.stock("milk")).isEqualTo(2);
    }

    @Test
    void shouldAccumulateRefills() {
        CoffeeMachineApi m = machine();
        m.refill("milk", 5);
        m.refill("milk", 7);

        assertThat(m.stock("milk")).isEqualTo(12);
        assertThat(m.stock("unobtainium")).isZero();
    }

    @Test
    void shouldRejectInvalidRecipes() {
        CoffeeMachineApi m = machine();

        assertThat(m.addRecipe("nothing", Map.of())).isFalse();
        assertThat(m.addRecipe("bad", Map.of("water", 0))).isFalse();
        assertThat(m.addRecipe("worse", Map.of("water", -3))).isFalse();
        assertThat(m.dispense("nothing")).isEqualTo("UNKNOWN_DRINK");
        assertThat(m.dispense("bad")).isEqualTo("UNKNOWN_DRINK");
    }

    @Test
    void shouldReplaceExistingRecipe() {
        CoffeeMachineApi m = machine();
        m.refill("water", 100);
        m.refill("coffee", 100);
        m.addRecipe("coffee-drink", Map.of("water", 10));
        m.addRecipe("coffee-drink", Map.of("coffee", 4));

        m.dispense("coffee-drink");

        assertThat(m.stock("water")).isEqualTo(100);
        assertThat(m.stock("coffee")).isEqualTo(96);
    }

    @Test
    void shouldShareIngredientsBetweenDrinks() {
        CoffeeMachineApi m = machine();
        m.refill("water", 50);
        m.addRecipe("a", Map.of("water", 30));
        m.addRecipe("b", Map.of("water", 30));

        assertThat(m.dispense("a")).isEqualTo("OK");
        assertThat(m.dispense("b")).isEqualTo("INSUFFICIENT:water");
    }

    @Test
    void shouldAllowConsumingStockExactlyToZero() {
        CoffeeMachineApi m = machine();
        m.refill("water", 30);
        m.addRecipe("a", Map.of("water", 30));

        assertThat(m.dispense("a")).isEqualTo("OK");
        assertThat(m.stock("water")).isZero();
        assertThat(m.dispense("a")).isEqualTo("INSUFFICIENT:water");
    }

    @Test
    void shouldListLowStockIngredientsSorted() {
        CoffeeMachineApi m = machine();
        m.refill("water", 100);
        m.refill("milk", 3);
        m.refill("coffee", 4);
        m.refill("sugar", 10);

        assertThat(m.lowStock(5)).containsExactly("coffee", "milk");
        assertThat(m.lowStock(4)).containsExactly("milk");
        assertThat(m.lowStock(1)).isEmpty();
    }

    @Test
    void shouldServeExactlyAsManyDrinksAsStockAllowsWhenOutletsRaceForOneIngredient() {
        CoffeeMachineApi m = machine();
        m.refill("milk", 50);
        m.addRecipe("latte", Map.of("milk", 5));

        List<String> results = Concurrently.collect(40, i -> m.dispense("latte"));

        assertThat(results.stream().filter("OK"::equals).count()).isEqualTo(10);
        assertThat(results.stream().filter("INSUFFICIENT:milk"::equals).count()).isEqualTo(30);
        assertThat(m.stock("milk")).isZero();
    }

    @Test
    void shouldConserveStockUnderConcurrentMixedDrinks() {
        CoffeeMachineApi m = machine();
        m.refill("water", 1000);
        m.refill("coffee", 120);
        m.refill("milk", 90);
        m.addRecipe("americano", Map.of("water", 3, "coffee", 1));
        m.addRecipe("latte", Map.of("water", 2, "milk", 1, "coffee", 1));

        List<String> results = Concurrently.collect(16, t -> {
            StringBuilder ok = new StringBuilder();
            for (int i = 0; i < 40; i++) {
                String drink = (i + t) % 2 == 0 ? "americano" : "latte";
                if (m.dispense(drink).equals("OK")) {
                    ok.append(drink.charAt(0));
                }
            }
            return ok.toString();
        });

        long americanos = results.stream().flatMapToInt(String::chars).filter(c -> c == 'a').count();
        long lattes = results.stream().flatMapToInt(String::chars).filter(c -> c == 'l').count();
        assertThat(m.stock("water")).isEqualTo((int) (1000 - 3 * americanos - 2 * lattes));
        assertThat(m.stock("coffee")).isEqualTo((int) (120 - americanos - lattes));
        assertThat(m.stock("milk")).isEqualTo((int) (90 - lattes));
        assertThat(m.stock("coffee")).isGreaterThanOrEqualTo(0);
    }
}
