package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Customer;
import contract.Order;
import contract.Wallet;
import org.junit.jupiter.api.Test;
import solution.Cashier;
import solution.ShippingCalculator;
import solution.SimpleWallet;
import solution.StandardCustomer;

class BehaviorTest {

    /** A customer whose wallet is off limits: any code that reaches through it fails the test. */
    private static final class StrictCustomer implements Customer {
        long funds;
        int payCalls;

        StrictCustomer(long funds) { this.funds = funds; }

        public String name() { return "strict"; }
        public boolean canAfford(long amount) { return funds >= amount; }
        public void pay(long amount) { payCalls++; funds -= amount; }
        public Wallet getWallet() { throw new AssertionError("Law of Demeter: do not reach through the customer to its wallet"); }
    }

    /** An order whose customer is off limits. */
    private record StrictOrder(String city, long items) implements Order {
        public Customer getCustomer() { throw new AssertionError("Law of Demeter: do not reach through the order to its customer"); }
        public String shippingCity() { return city; }
        public long itemTotal() { return items; }
    }

    @Test
    void shouldApproveAndTakePaymentWhenTheCustomerCanAfford() {
        Cashier cashier = new Cashier();
        StrictCustomer customer = new StrictCustomer(500);

        assertThat(cashier.checkout(customer, 300)).isEqualTo("APPROVED");

        assertThat(customer.payCalls).isEqualTo(1);
        assertThat(customer.funds).isEqualTo(200);
        assertThat(cashier.revenue()).isEqualTo(300);
    }

    @Test
    void shouldDeclineWithoutTakingMoneyWhenTheCustomerCannotAfford() {
        Cashier cashier = new Cashier();
        StrictCustomer customer = new StrictCustomer(100);

        assertThat(cashier.checkout(customer, 300)).isEqualTo("DECLINED");

        assertThat(customer.payCalls).isZero();
        assertThat(cashier.revenue()).isZero();
    }

    @Test
    void shouldAccumulateRevenueAcrossCustomers() {
        Cashier cashier = new Cashier();

        cashier.checkout(new StrictCustomer(1000), 400);
        cashier.checkout(new StrictCustomer(50), 400);
        cashier.checkout(new StrictCustomer(1000), 250);

        assertThat(cashier.revenue()).isEqualTo(650);
    }

    @Test
    void shouldLetTheCustomerPayFromItsOwnWalletAndHideTheDetails() {
        Wallet wallet = new SimpleWallet(1000);
        Customer ann = new StandardCustomer("Ann", wallet);

        assertThat(ann.name()).isEqualTo("Ann");
        assertThat(ann.canAfford(1000)).isTrue();
        assertThat(ann.canAfford(1001)).isFalse();
        ann.pay(400);

        assertThat(wallet.balance()).isEqualTo(600);
        assertThatThrownBy(() -> ann.pay(601)).isInstanceOf(IllegalStateException.class);
        assertThat(wallet.balance()).isEqualTo(600);
    }

    @Test
    void shouldWorkEndToEndWithTheRealCustomer() {
        Cashier cashier = new Cashier();
        Wallet wallet = new SimpleWallet(500);
        Customer bob = new StandardCustomer("Bob", wallet);

        assertThat(cashier.checkout(bob, 500)).isEqualTo("APPROVED");
        assertThat(cashier.checkout(bob, 1)).isEqualTo("DECLINED");

        assertThat(wallet.balance()).isZero();
        assertThat(cashier.revenue()).isEqualTo(500);
    }

    @Test
    void shouldPriceShippingFromTheOrderWithoutReachingThroughToTheCustomer() {
        ShippingCalculator shipping = new ShippingCalculator();

        assertThat(shipping.cost(new StrictOrder("Paris", 5000))).isEqualTo(1500);
        assertThat(shipping.cost(new StrictOrder("LYON", 5000))).isEqualTo(1800);
        assertThat(shipping.cost(new StrictOrder("Berlin", 5000))).isEqualTo(2500);
        assertThat(shipping.cost(new StrictOrder("Paris", 10_000))).isZero();
        assertThat(shipping.cost(new StrictOrder("Berlin", 9_999))).isEqualTo(2500);
    }

    @Test
    void shouldRejectInvalidInput() {
        Cashier cashier = new Cashier();

        assertThatThrownBy(() -> cashier.checkout(null, 100)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> cashier.checkout(new StrictCustomer(100), 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new SimpleWallet(-1)).isInstanceOf(IllegalArgumentException.class);
        Wallet w = new SimpleWallet(10);
        assertThatThrownBy(() -> w.debit(0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> w.debit(11)).isInstanceOf(IllegalStateException.class);
        assertThat(w.balance()).isEqualTo(10);
    }
}
