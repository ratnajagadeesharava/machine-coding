use p66_strategy::contract::*;
use p66_strategy::{cheapest, ByWeight, DistanceBased, FlatRate, FreeOverThreshold, ShippingCalculator};

const LIGHT_NEAR: Parcel = Parcel { weight_grams: 500, distance_km: 10, value_cents: 2000 };
const HEAVY_FAR: Parcel = Parcel { weight_grams: 2500, distance_km: 300, value_cents: 20_000 };

/// A strategy written in the test: customer collects the parcel themselves.
struct Pickup;

impl ShippingStrategy for Pickup {
    fn name(&self) -> String {
        "pickup".to_string()
    }
    fn cost(&self, _parcel: &Parcel) -> u64 {
        0
    }
}

fn parcel(weight_grams: u32, distance_km: u32, value_cents: u64) -> Parcel {
    Parcel { weight_grams, distance_km, value_cents }
}

#[test]
fn should_charge_a_flat_rate_regardless_of_the_parcel() {
    let flat = FlatRate::new(499);

    assert_eq!(flat.cost(&LIGHT_NEAR), 499);
    assert_eq!(flat.cost(&HEAVY_FAR), 499);
    assert_eq!(flat.name(), "flat");
}

#[test]
fn should_charge_by_the_started_kilogram() {
    let by_weight = ByWeight::new(200);

    assert_eq!(by_weight.cost(&parcel(1, 0, 0)), 200);
    assert_eq!(by_weight.cost(&parcel(1000, 0, 0)), 200);
    assert_eq!(by_weight.cost(&parcel(1001, 0, 0)), 400);
    assert_eq!(by_weight.cost(&HEAVY_FAR), 600);
    assert_eq!(by_weight.name(), "weight");
}

#[test]
fn should_charge_by_distance() {
    let by_distance = DistanceBased::new(3);

    assert_eq!(by_distance.cost(&LIGHT_NEAR), 30);
    assert_eq!(by_distance.cost(&HEAVY_FAR), 900);
    assert_eq!(by_distance.cost(&parcel(10, 0, 0)), 0);
    assert_eq!(by_distance.name(), "distance");
}

#[test]
fn should_ship_free_above_the_threshold_and_delegate_below_it() {
    let strategy = FreeOverThreshold::new(10_000, Box::new(FlatRate::new(499)));

    assert_eq!(strategy.cost(&parcel(100, 5, 10_000)), 0);
    assert_eq!(strategy.cost(&HEAVY_FAR), 0);
    assert_eq!(strategy.cost(&parcel(100, 5, 9_999)), 499);
    assert_eq!(strategy.name(), "free>=10000/flat");
}

#[test]
fn should_let_strategies_wrap_other_strategies() {
    let nested = FreeOverThreshold::new(50_000, Box::new(FreeOverThreshold::new(10_000, Box::new(ByWeight::new(100)))));

    assert_eq!(nested.cost(&parcel(2500, 1, 60_000)), 0);
    assert_eq!(nested.cost(&parcel(2500, 1, 20_000)), 0);
    assert_eq!(nested.cost(&parcel(2500, 1, 5_000)), 300);
    assert_eq!(nested.name(), "free>=50000/free>=10000/weight");
}

#[test]
fn should_price_with_whichever_strategy_is_current_and_swap_at_runtime() {
    let mut calc = ShippingCalculator::new(Box::new(FlatRate::new(499)));
    assert_eq!(calc.quote(&HEAVY_FAR), Ok(499));

    calc.set_strategy(Box::new(DistanceBased::new(3)));
    assert_eq!(calc.quote(&HEAVY_FAR), Ok(900));
    assert_eq!(calc.strategy_name(), "distance");

    calc.set_strategy(Box::new(Pickup));
    assert_eq!(calc.quote(&HEAVY_FAR), Ok(0));
    assert_eq!(calc.strategy_name(), "pickup");
}

#[test]
fn should_pick_the_cheapest_strategy_and_prefer_the_earlier_one_on_a_tie() {
    let options: Vec<Box<dyn ShippingStrategy>> =
        vec![Box::new(FlatRate::new(499)), Box::new(ByWeight::new(200)), Box::new(DistanceBased::new(3)), Box::new(Pickup)];

    assert_eq!(cheapest(&options, &HEAVY_FAR), Ok(3));
    assert_eq!(cheapest(&options[..3], &LIGHT_NEAR), Ok(2));
    let tie: Vec<Box<dyn ShippingStrategy>> = vec![Box::new(FlatRate::new(100)), Box::new(ByWeight::new(100))];
    assert_eq!(cheapest(&tie, &parcel(1000, 1, 1)), Ok(0));
}

#[test]
fn should_reject_invalid_parcels_and_empty_choices() {
    let calc = ShippingCalculator::new(Box::new(FlatRate::new(1)));
    let options: Vec<Box<dyn ShippingStrategy>> = vec![];

    assert_eq!(calc.quote(&parcel(0, 1, 1)), Err(ShipError::InvalidParcel));
    assert_eq!(cheapest(&options, &LIGHT_NEAR), Err(ShipError::NoStrategies));
    assert_eq!(cheapest(&[Box::new(Pickup) as Box<dyn ShippingStrategy>], &parcel(0, 1, 1)), Err(ShipError::InvalidParcel));
    assert_eq!(calc.quote(&LIGHT_NEAR), Ok(1));
}
