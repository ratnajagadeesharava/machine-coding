package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Money;

class BehaviorTest {

    @Test
    void shouldExposeCurrencyAndMinorUnits() {
        Money m = Money.of("USD", 1250);

        assertThat(m.currency()).isEqualTo("USD");
        assertThat(m.minorUnits()).isEqualTo(1250);
        assertThat(Money.zero("EUR").minorUnits()).isZero();
    }

    @Test
    void shouldTreatEqualValuesAsEqualObjectsButNotTheSameInstance() {
        Money a = Money.of("USD", 500);
        Money b = Money.of("USD", 500);

        assertThat(a).isNotSameAs(b).isEqualTo(b).hasSameHashCodeAs(b);
        assertThat(a).isNotEqualTo(Money.of("USD", 501)).isNotEqualTo(Money.of("EUR", 500));
        Set<Money> set = new HashSet<>(List.of(a));
        assertThat(set).contains(b);
    }

    @Test
    void shouldReturnNewObjectsAndLeaveOperandsUntouched() {
        Money a = Money.of("USD", 1000);
        Money b = Money.of("USD", 250);

        assertThat(a.plus(b)).isEqualTo(Money.of("USD", 1250));
        assertThat(a.minus(b)).isEqualTo(Money.of("USD", 750));
        assertThat(b.minus(a)).isEqualTo(Money.of("USD", -750)).matches(Money::isNegative);
        assertThat(a.times(3)).isEqualTo(Money.of("USD", 3000));
        assertThat(a).isEqualTo(Money.of("USD", 1000));
        assertThat(b).isEqualTo(Money.of("USD", 250));
    }

    @Test
    void shouldPrintTwoDecimalPlaces() {
        assertThat(Money.of("USD", 1250).toString()).isEqualTo("USD 12.50");
        assertThat(Money.of("EUR", 5).toString()).isEqualTo("EUR 0.05");
        assertThat(Money.of("GBP", 0).toString()).isEqualTo("GBP 0.00");
        assertThat(Money.of("USD", -5).toString()).isEqualTo("USD -0.05");
        assertThat(Money.of("USD", 100_000).toString()).isEqualTo("USD 1000.00");
    }

    @Test
    void shouldRejectMixingCurrencies() {
        Money usd = Money.of("USD", 100);
        Money eur = Money.of("EUR", 100);

        assertThatThrownBy(() -> usd.plus(eur)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> usd.minus(eur)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> usd.compareTo(eur)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectInvalidCurrencyCodes() {
        assertThatThrownBy(() -> Money.of("usd", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Money.of("US", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Money.of(null, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Money.zero("DOLLARS")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldOrderAmountsOfTheSameCurrency() {
        List<Money> list = new ArrayList<>(List.of(Money.of("USD", 300), Money.of("USD", -10), Money.of("USD", 120)));

        Collections.sort(list);

        assertThat(list).containsExactly(Money.of("USD", -10), Money.of("USD", 120), Money.of("USD", 300));
        assertThat(Money.of("USD", 5).compareTo(Money.of("USD", 5))).isZero();
    }

    @Test
    void shouldFailLoudlyOnOverflow() {
        Money big = Money.of("USD", Long.MAX_VALUE);

        assertThatThrownBy(() -> big.plus(Money.of("USD", 1))).isInstanceOf(ArithmeticException.class);
        assertThatThrownBy(() -> big.times(2)).isInstanceOf(ArithmeticException.class);
    }
}
