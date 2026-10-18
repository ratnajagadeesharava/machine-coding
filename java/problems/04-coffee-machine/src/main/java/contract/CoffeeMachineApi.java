package contract;

import java.util.List;
import java.util.Map;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface CoffeeMachineApi {

    void refill(String ingredient, int units);

    /** Defines or replaces a recipe. Returns false if the recipe is invalid. */
    boolean addRecipe(String drink, Map<String, Integer> ingredients);

    /** "OK", "UNKNOWN_DRINK" or "INSUFFICIENT:&lt;ingredient&gt;". */
    String dispense(String drink);

    int stock(String ingredient);

    /** Ingredients with stock strictly below the threshold, sorted alphabetically. */
    List<String> lowStock(int threshold);
}
