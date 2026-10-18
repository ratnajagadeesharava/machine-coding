use p47_ocp::contract::*;
use p47_ocp::{CarPolicy, FeeCalculator, MotorcyclePolicy};

const CALCULATOR_SOURCE: &str = include_str!("../src/calculator.rs");

/// New requirement, written in the test: electric vehicles park free for an hour, then 50 per started 30 min.
struct ElectricPolicy;

impl FeePolicy for ElectricPolicy {
    fn supports(&self, t: &str) -> bool {
        t.eq_ignore_ascii_case("electric")
    }
    fn fee(&self, minutes: u32) -> u64 {
        if minutes <= 60 {
            0
        } else {
            50 * ((minutes as u64 - 60 + 29) / 30)
        }
    }
}

struct Flat(&'static str, u64);

impl FeePolicy for Flat {
    fn supports(&self, t: &str) -> bool {
        self.0.eq_ignore_ascii_case(t)
    }
    fn fee(&self, _minutes: u32) -> u64 {
        self.1
    }
}

fn standard() -> FeeCalculator {
    FeeCalculator::new(vec![Box::new(CarPolicy::new()), Box::new(MotorcyclePolicy::new())])
}

#[test]
fn should_price_cars_by_the_hour_then_by_half_hour() {
    let calc = standard();

    assert_eq!(calc.fee("car", 0), Ok(0));
    assert_eq!(calc.fee("car", 30), Ok(200));
    assert_eq!(calc.fee("car", 60), Ok(200));
    assert_eq!(calc.fee("car", 61), Ok(300));
    assert_eq!(calc.fee("CAR", 90), Ok(300));
    assert_eq!(calc.fee("Car", 91), Ok(400));
    assert_eq!(calc.fee("car", 1000), Ok(1500));
}

#[test]
fn should_charge_motorcycles_half() {
    let calc = standard();

    assert_eq!(calc.fee("motorcycle", 30), Ok(100));
    assert_eq!(calc.fee("Motorcycle", 91), Ok(200));
    assert_eq!(calc.fee("motorcycle", 1000), Ok(750));
}

#[test]
fn should_support_a_new_vehicle_type_without_changing_the_calculator() {
    let calc = FeeCalculator::new(vec![
        Box::new(CarPolicy::new()),
        Box::new(MotorcyclePolicy::new()),
        Box::new(ElectricPolicy),
    ]);

    assert_eq!(calc.fee("electric", 45), Ok(0));
    assert_eq!(calc.fee("electric", 61), Ok(50));
    assert_eq!(calc.fee("electric", 120), Ok(100));
    assert_eq!(calc.fee("car", 61), Ok(300));
}

#[test]
fn should_accept_new_policies_at_runtime() {
    let mut calc = standard();
    assert_eq!(calc.fee("boat", 10), Err(FeeError::UnknownVehicle));

    calc.register(Box::new(Flat("boat", 999)));

    assert_eq!(calc.fee("boat", 10), Ok(999));
    assert_eq!(calc.fee("car", 30), Ok(200));
}

#[test]
fn should_let_the_latest_registered_policy_override_an_earlier_one() {
    let mut calc = standard();

    calc.register(Box::new(Flat("car", 0)));

    assert_eq!(calc.fee("car", 500), Ok(0));
    assert_eq!(calc.fee("motorcycle", 30), Ok(100));
}

#[test]
fn should_work_with_no_built_in_vehicles_at_all() {
    let calc = FeeCalculator::new(vec![Box::new(Flat("boat", 42))]);

    assert_eq!(calc.fee("boat", 5), Ok(42));
    assert_eq!(calc.fee("car", 5), Err(FeeError::UnknownVehicle));
}

#[test]
fn should_reject_unknown_vehicles() {
    let calc = standard();

    assert_eq!(calc.fee("spaceship", 10), Err(FeeError::UnknownVehicle));
    assert_eq!(calc.fee("", 10), Err(FeeError::UnknownVehicle));
    assert_eq!(calc.fee("car", 30), Ok(200));
}

#[test]
fn should_keep_vehicle_knowledge_out_of_the_calculator_source() {
    assert_eq!(standard().fee("car", 30), Ok(200));

    let source = CALCULATOR_SOURCE.to_lowercase();
    assert!(!source.contains("\"car\""), "calculator.rs must not mention vehicle types");
    assert!(!source.contains("\"motorcycle\""), "calculator.rs must not mention vehicle types");
}
