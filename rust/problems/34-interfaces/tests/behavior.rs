use p34_interfaces::contract::*;
use p34_interfaces::{HumiditySensor, SensorHub, TemperatureSensor};

/// A sensor written in the test: the hub must work with ANY implementation of the trait.
struct Custom {
    id: String,
    unit: String,
    value: f64,
}

impl Sensor for Custom {
    fn id(&self) -> &str {
        &self.id
    }
    fn unit(&self) -> &str {
        &self.unit
    }
    fn read(&self) -> Result<f64, SensorFault> {
        Ok(self.value)
    }
}

fn custom(id: &str, unit: &str, value: f64) -> Box<dyn Sensor> {
    Box::new(Custom { id: id.into(), unit: unit.into(), value })
}

fn temp(id: &str, v: f64) -> Box<dyn Sensor> {
    Box::new(TemperatureSensor::new(id, Box::new(move || v)))
}

fn humidity(id: &str, v: f64) -> Box<dyn Sensor> {
    Box::new(HumiditySensor::new(id, Box::new(move || v)))
}

#[test]
fn should_read_a_temperature_sensor_through_the_trait() {
    let s: Box<dyn Sensor> = temp("t1", 21.5);

    assert_eq!(s.id(), "t1");
    assert_eq!(s.unit(), "C");
    assert_eq!(s.read(), Ok(21.5));
}

#[test]
fn should_report_percent_for_humidity() {
    let s = humidity("h1", 40.0);

    assert_eq!(s.unit(), "%");
    assert_eq!(s.read(), Ok(40.0));
}

#[test]
fn should_treat_impossible_readings_as_faults() {
    assert!(temp("t", -300.0).read().is_err());
    assert!(humidity("h", 120.0).read().is_err());
    assert!(humidity("h", -0.5).read().is_err());
    assert_eq!(humidity("e", 100.0).read(), Ok(100.0));
}

#[test]
fn should_read_all_in_registration_order_and_skip_faulty_sensors() {
    let mut hub = SensorHub::new();
    hub.register(temp("b", 10.0)).unwrap();
    hub.register(humidity("broken", -5.0)).unwrap();
    hub.register(humidity("a", 55.0)).unwrap();

    assert_eq!(hub.read_all(), vec![("b".to_string(), 10.0), ("a".to_string(), 55.0)]);
    assert_eq!(hub.last_faults(), vec!["broken".to_string()]);
}

#[test]
fn should_work_with_a_sensor_it_has_never_seen() {
    let mut hub = SensorHub::new();
    hub.register(custom("lux1", "lx", 300.0)).unwrap();
    hub.register(temp("t1", 20.0)).unwrap();

    let all = hub.read_all();
    assert!(all.contains(&("lux1".to_string(), 300.0)));
    assert!(all.contains(&("t1".to_string(), 20.0)));
    assert_eq!(hub.ids(), vec!["lux1".to_string(), "t1".to_string()]);
}

#[test]
fn should_reject_duplicate_ids() {
    let mut hub = SensorHub::new();
    hub.register(temp("same", 1.0)).unwrap();

    assert_eq!(hub.register(custom("same", "C", 2.0)), Err(HubError::DuplicateId));
    assert_eq!(hub.ids().len(), 1);
}

#[test]
fn should_average_healthy_sensors_of_one_unit() {
    let mut hub = SensorHub::new();
    hub.register(temp("t1", 20.0)).unwrap();
    hub.register(temp("t2", 30.0)).unwrap();
    hub.register(temp("t3", -500.0)).unwrap();
    hub.register(custom("t4", "C", 10.0)).unwrap();
    hub.register(humidity("h1", 80.0)).unwrap();

    assert_eq!(hub.average("C"), Some(20.0));
    assert_eq!(hub.average("%"), Some(80.0));
    assert_eq!(hub.average("lx"), None);
}

#[test]
fn should_unregister_by_id() {
    let mut hub = SensorHub::new();
    hub.register(temp("t1", 20.0)).unwrap();
    hub.register(temp("t2", 21.0)).unwrap();

    assert!(hub.unregister("t1"));
    assert!(!hub.unregister("t1"));
    assert_eq!(hub.ids(), vec!["t2".to_string()]);
}
