use p36_polymorphism::contract::*;
use p36_polymorphism::{Bus, Car, Motorcycle, TollPlaza, Truck};

/// Types that exist only in this test file: the plaza must handle them without any change.
struct Ambulance(String);
impl Vehicle for Ambulance {
    fn plate(&self) -> &str {
        &self.0
    }
    fn toll(&self, _peak: bool) -> u64 {
        0
    }
}

struct Limo(String);
impl Vehicle for Limo {
    fn plate(&self) -> &str {
        &self.0
    }
    fn toll(&self, peak: bool) -> u64 {
        if peak {
            5000
        } else {
            2500
        }
    }
}

#[test]
fn should_charge_a_car_more_at_peak_time() {
    let car = Car::new("C-1");

    assert_eq!(car.toll(false), 500);
    assert_eq!(car.toll(true), 750);
    assert_eq!(car.plate(), "C-1");
}

#[test]
fn should_never_surcharge_motorcycles() {
    let bike = Motorcycle::new("M-1");

    assert_eq!(bike.toll(false), 200);
    assert_eq!(bike.toll(true), 200);
}

#[test]
fn should_charge_trucks_per_axle() {
    assert_eq!(Truck::new("T", 2).unwrap().toll(false), 1000);
    assert_eq!(Truck::new("T", 5).unwrap().toll(false), 1900);
    assert_eq!(Truck::new("T", 4).unwrap().toll(true), 2400);
    assert!(matches!(Truck::new("T", 1), Err(TollError::InvalidAxles)));
}

#[test]
fn should_charge_buses_a_flat_rate_plus_twenty_percent_at_peak() {
    let bus = Bus::new("B-1");

    assert_eq!(bus.toll(false), 800);
    assert_eq!(bus.toll(true), 960);
}

#[test]
fn should_sum_the_right_toll_for_each_different_vehicle_through_one_call() {
    let mut plaza = TollPlaza::new();
    let queue: Vec<Box<dyn Vehicle>> = vec![
        Box::new(Car::new("C")),
        Box::new(Motorcycle::new("M")),
        Box::new(Truck::new("T", 3).unwrap()),
        Box::new(Bus::new("B")),
    ];

    let charged: u64 = queue.iter().map(|v| plaza.charge(v.as_ref(), false)).sum();

    assert_eq!(charged, 500 + 200 + 1300 + 800);
    assert_eq!(plaza.total(), charged);
}

#[test]
fn should_handle_vehicle_types_it_has_never_seen() {
    let mut plaza = TollPlaza::new();

    assert_eq!(plaza.charge(&Ambulance("AMB".into()), true), 0);
    assert_eq!(plaza.charge(&Limo("LIMO".into()), true), 5000);
    assert_eq!(plaza.charge(&Car::new("C"), true), 750);
    assert_eq!(plaza.total(), 5750);
}

#[test]
fn should_keep_receipts_in_charge_order() {
    let mut plaza = TollPlaza::new();
    plaza.charge(&Car::new("AB-12"), false);
    plaza.charge(&Ambulance("AMB".into()), false);
    plaza.charge(&Bus::new("BUS-9"), true);

    assert_eq!(plaza.receipts(), vec!["AB-12:500", "AMB:0", "BUS-9:960"]);
}

#[test]
fn should_start_empty() {
    let plaza = TollPlaza::new();

    assert_eq!(plaza.total(), 0);
    assert!(plaza.receipts().is_empty());
}
