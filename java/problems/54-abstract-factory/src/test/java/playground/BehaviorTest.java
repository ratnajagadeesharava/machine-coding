package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.AddressValidator;
import contract.PriceFormatter;
import contract.RegionKit;
import contract.TaxCalculator;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.CheckoutSummary;
import solution.EuKit;
import solution.IndiaKit;
import solution.Kits;
import solution.UsKit;

class BehaviorTest {

    /** A whole family written in the test: the client must work with it without knowing anything concrete. */
    private static final class TestKit implements RegionKit {
        public String region() { return "TEST"; }
        public TaxCalculator taxCalculator() { return net -> 1; }
        public PriceFormatter priceFormatter() { return cents -> "#" + cents; }
        public AddressValidator addressValidator() { return code -> false; }
    }

    @Test
    void shouldGiveTheUsFamily() {
        RegionKit kit = new UsKit();

        assertThat(kit.region()).isEqualTo("US");
        assertThat(kit.taxCalculator().tax(10_000)).isEqualTo(800);
        assertThat(kit.taxCalculator().tax(1257)).isEqualTo(101);
        assertThat(kit.taxCalculator().tax(1255)).isEqualTo(100);
        assertThat(kit.priceFormatter().format(1250)).isEqualTo("$12.50");
        assertThat(kit.priceFormatter().format(5)).isEqualTo("$0.05");
        assertThat(kit.priceFormatter().format(100_000)).isEqualTo("$1000.00");
        assertThat(kit.addressValidator().isValidPostalCode("12345")).isTrue();
        assertThat(kit.addressValidator().isValidPostalCode("12345-6789")).isTrue();
        assertThat(kit.addressValidator().isValidPostalCode("1234")).isFalse();
        assertThat(kit.addressValidator().isValidPostalCode("ABCDE")).isFalse();
    }

    @Test
    void shouldGiveTheEuFamily() {
        RegionKit kit = new EuKit();

        assertThat(kit.region()).isEqualTo("EU");
        assertThat(kit.taxCalculator().tax(10_000)).isEqualTo(1900);
        assertThat(kit.priceFormatter().format(1250)).isEqualTo("12,50 EUR");
        assertThat(kit.priceFormatter().format(5)).isEqualTo("0,05 EUR");
        assertThat(kit.addressValidator().isValidPostalCode("10115")).isTrue();
        assertThat(kit.addressValidator().isValidPostalCode("12345-6789")).isFalse();
    }

    @Test
    void shouldGiveTheIndianFamily() {
        RegionKit kit = new IndiaKit();

        assertThat(kit.region()).isEqualTo("IN");
        assertThat(kit.taxCalculator().tax(10_000)).isEqualTo(1800);
        assertThat(kit.priceFormatter().format(1250)).isEqualTo("Rs 12.50");
        assertThat(kit.addressValidator().isValidPostalCode("560001")).isTrue();
        assertThat(kit.addressValidator().isValidPostalCode("12345")).isFalse();
    }

    @Test
    void shouldChooseTheFactoryByRegionCode() {
        assertThat(Kits.forRegion("us")).isInstanceOf(UsKit.class);
        assertThat(Kits.forRegion("EU")).isInstanceOf(EuKit.class);
        assertThat(Kits.forRegion("In")).isInstanceOf(IndiaKit.class);
        assertThatThrownBy(() -> Kits.forRegion("JP")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Kits.forRegion(null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(Kits.forRegion("us").region()).isEqualTo("US");
    }

    @Test
    void shouldSummariseWithEverythingFromOneFamily() {
        assertThat(CheckoutSummary.summarize(new UsKit(), 10_000, "12345"))
                .isEqualTo("US: net $100.00, tax $8.00, total $108.00, postal valid");
        assertThat(CheckoutSummary.summarize(new EuKit(), 10_000, "10115"))
                .isEqualTo("EU: net 100,00 EUR, tax 19,00 EUR, total 119,00 EUR, postal valid");
        assertThat(CheckoutSummary.summarize(new IndiaKit(), 10_000, "5600"))
                .isEqualTo("IN: net Rs 100.00, tax Rs 18.00, total Rs 118.00, postal invalid");
    }

    @Test
    void shouldSummariseWithAFamilyItHasNeverSeen() {
        assertThat(CheckoutSummary.summarize(new TestKit(), 100, "whatever"))
                .isEqualTo("TEST: net #100, tax #1, total #101, postal invalid");
    }

    @Test
    void shouldGiveUsableProductsOnEveryCallAndRejectNegativeAmounts() {
        RegionKit kit = new UsKit();

        assertThat(kit.taxCalculator().tax(100)).isEqualTo(kit.taxCalculator().tax(100));
        assertThat(kit.priceFormatter().format(1)).isEqualTo("$0.01");
        assertThatThrownBy(() -> kit.taxCalculator().tax(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> kit.priceFormatter().format(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThat(kit.addressValidator().isValidPostalCode(null)).isFalse();
    }

    @Test
    void shouldNotMixProductsAcrossFamilies() {
        List<RegionKit> kits = List.of(new UsKit(), new EuKit(), new IndiaKit());

        List<String> formatted = kits.stream().map(k -> k.priceFormatter().format(1250)).toList();
        List<Long> taxes = kits.stream().map(k -> k.taxCalculator().tax(10_000)).toList();

        assertThat(formatted).containsExactly("$12.50", "12,50 EUR", "Rs 12.50");
        assertThat(taxes).containsExactly(800L, 1900L, 1800L);
    }
}
