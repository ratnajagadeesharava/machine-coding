use p35_inheritance::contract::PayrollError;
use p35_inheritance::{Engineer, Intern, Manager, Payable, Staff};

fn staff(name: &str, base: i64) -> Staff {
    Staff::new(name, base).unwrap()
}

/// A type that only says where its Staff is: it must get every other method for free.
struct Plain(Staff);

impl Payable for Plain {
    fn staff(&self) -> &Staff {
        &self.0
    }
}

#[test]
fn should_pay_the_base_salary_for_a_plain_employee() {
    let e = Plain(staff("Zed", 1000));

    assert_eq!(e.staff().name(), "Zed");
    assert_eq!(e.monthly_pay(), 1000);
    assert_eq!(e.role(), "Employee");
    assert_eq!(e.describe(), "Employee: Zed earns 1000");
}

#[test]
fn should_raise_engineer_pay_by_twenty_percent_per_level() {
    assert_eq!(Engineer::new(staff("A", 4000), 1).unwrap().monthly_pay(), 4000);
    assert_eq!(Engineer::new(staff("B", 4000), 2).unwrap().monthly_pay(), 4800);
    assert_eq!(Engineer::new(staff("C", 4000), 3).unwrap().monthly_pay(), 5600);
}

#[test]
fn should_pay_interns_half() {
    assert_eq!(Intern::new(staff("Ivy", 1001)).monthly_pay(), 500);
}

#[test]
fn should_add_five_percent_of_the_reports_pay_to_a_manager() {
    let reports: Vec<Box<dyn Payable>> = vec![
        Box::new(Engineer::new(staff("E", 4000), 2).unwrap()), // 4800
        Box::new(Intern::new(staff("I", 1000))),               // 500
    ];
    let m = Manager::new(staff("M", 5000), reports);

    assert_eq!(m.monthly_pay(), 5000 + 5300 / 20);
}

#[test]
fn should_let_managers_manage_managers() {
    let team = Manager::new(
        staff("Lead", 5000),
        vec![Box::new(Engineer::new(staff("E", 4000), 2).unwrap()), Box::new(Intern::new(staff("I", 1000)))],
    );
    let team_pay = team.monthly_pay();
    let dept = Manager::new(staff("Head", 8000), vec![Box::new(team)]);

    assert_eq!(dept.monthly_pay(), 8000 + team_pay / 20);
}

#[test]
fn should_describe_every_kind_through_the_one_inherited_method() {
    let all: Vec<Box<dyn Payable>> = vec![
        Box::new(Plain(staff("P", 1000))),
        Box::new(Engineer::new(staff("E", 1000), 3).unwrap()),
        Box::new(Intern::new(staff("I", 1000))),
        Box::new(Manager::new(staff("M", 1000), vec![])),
    ];

    let described: Vec<String> = all.iter().map(|p| p.describe()).collect();

    assert_eq!(
        described,
        vec!["Employee: P earns 1000", "Engineer: E earns 1400", "Intern: I earns 500", "Manager: M earns 1000"]
    );
}

#[test]
fn should_reject_invalid_input() {
    assert!(matches!(Staff::new("  ", 1), Err(PayrollError::InvalidName)));
    assert!(matches!(Staff::new("A", -1), Err(PayrollError::NegativeSalary)));
    assert!(matches!(Engineer::new(staff("A", 1), 0), Err(PayrollError::InvalidLevel)));
    assert!(matches!(Engineer::new(staff("A", 1), 4), Err(PayrollError::InvalidLevel)));
}
