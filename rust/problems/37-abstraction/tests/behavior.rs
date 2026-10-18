use p37_abstraction::contract::AccountError;
use p37_abstraction::{Account, AccountState, CheckingAccount, SavingsAccount};

/// Written in the test: it supplies only the four required methods and must get the rest for free.
struct Configurable {
    state: AccountState,
    fee: i64,
    allow: bool,
}

impl Configurable {
    fn new(fee: i64, allow: bool) -> Self {
        Configurable { state: AccountState::new(), fee, allow }
    }
}

impl Account for Configurable {
    fn state(&self) -> &AccountState {
        &self.state
    }
    fn state_mut(&mut self) -> &mut AccountState {
        &mut self.state
    }
    fn monthly_fee(&self) -> i64 {
        self.fee
    }
    fn can_withdraw(&self, _amount: i64) -> bool {
        self.allow
    }
}

#[test]
fn should_implement_deposit_and_withdraw_once_in_the_shared_part() {
    let mut a = Configurable::new(0, true);

    a.deposit(500).unwrap();
    a.withdraw(200).unwrap();

    assert_eq!(a.balance(), 300);
}

#[test]
fn should_ask_the_account_type_before_every_withdrawal() {
    let mut a = Configurable::new(0, false);
    a.deposit(500).unwrap();

    assert_eq!(a.withdraw(1), Err(AccountError::Declined));
    assert_eq!(a.balance(), 500);
}

#[test]
fn should_charge_the_account_type_fee_at_month_end() {
    let mut a = Configurable::new(75, true);
    a.deposit(100).unwrap();

    a.month_end();
    a.month_end();

    assert_eq!(a.balance(), -50);
}

#[test]
fn should_reject_non_positive_amounts() {
    let mut a = Configurable::new(0, true);

    assert_eq!(a.deposit(0), Err(AccountError::InvalidAmount));
    assert_eq!(a.deposit(-5), Err(AccountError::InvalidAmount));
    assert_eq!(a.withdraw(0), Err(AccountError::InvalidAmount));
    assert_eq!(a.balance(), 0);
}

#[test]
fn should_keep_a_savings_account_above_its_minimum_balance() {
    let mut s = SavingsAccount::new(1000);
    s.deposit(1500).unwrap();

    s.withdraw(500).unwrap();
    assert_eq!(s.withdraw(1), Err(AccountError::Declined));
    s.month_end();

    assert_eq!(s.balance(), 1000);
}

#[test]
fn should_let_a_checking_account_overdraw_within_the_limit_and_charge_a_fee() {
    let mut c = CheckingAccount::new(500);
    c.deposit(100).unwrap();

    c.withdraw(600).unwrap();
    assert_eq!(c.balance(), -500);
    assert_eq!(c.withdraw(1), Err(AccountError::Declined));
    c.month_end();

    assert_eq!(c.balance(), -700);
}

#[test]
fn should_waive_the_checking_fee_for_rich_customers() {
    let mut c = CheckingAccount::new(0);
    c.deposit(100_000).unwrap();

    c.month_end();

    assert_eq!(c.balance(), 100_000);
}

#[test]
fn should_treat_different_accounts_uniformly_through_the_trait() {
    let mut all: Vec<Box<dyn Account>> = vec![
        Box::new(SavingsAccount::new(0)),
        Box::new(CheckingAccount::new(0)),
        Box::new(Configurable::new(10, true)),
    ];
    for a in all.iter_mut() {
        a.deposit(1000).unwrap();
        a.month_end();
    }

    let balances: Vec<i64> = all.iter().map(|a| a.balance()).collect();

    assert_eq!(balances, vec![1000, 800, 990]);
}
