use p63_flyweight::contract::MapError;
use p63_flyweight::{MapLayer, StyleFactory};
use std::collections::HashSet;
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn should_share_one_style_object_per_shape_and_colour() {
    let factory = StyleFactory::new();

    let a = factory.get("pin", "red").unwrap();
    let b = factory.get("pin", "red").unwrap();
    let c = factory.get("pin", "blue").unwrap();
    let d = factory.get("flag", "red").unwrap();
    let e = factory.get("pin", "Red").unwrap();

    assert!(Arc::ptr_eq(&a, &b));
    assert!(!Arc::ptr_eq(&a, &c) && !Arc::ptr_eq(&a, &d) && !Arc::ptr_eq(&a, &e));
    assert_eq!(factory.instances_created(), 4);
    assert_eq!(a.shape(), "pin");
    assert_eq!(a.color(), "red");
    assert_eq!(a.bitmap_size(), 4096);
}

#[test]
fn should_create_only_three_styles_for_ten_thousand_markers() {
    let factory = Arc::new(StyleFactory::new());
    let mut layer = MapLayer::new(factory.clone());
    let kinds = [("pin", "red"), ("pin", "blue"), ("flag", "green")];

    for i in 0..10_000usize {
        let (shape, color) = kinds[i % 3];
        layer.add_marker((i % 90) as f64, (i % 180) as f64, &format!("m{i}"), shape, color).unwrap();
    }

    let distinct: HashSet<*const _> = layer.markers().iter().map(|m| Arc::as_ptr(m.style())).collect();
    assert_eq!(layer.marker_count(), 10_000);
    assert_eq!(factory.instances_created(), 3);
    assert_eq!(distinct.len(), 3);
}

#[test]
fn should_keep_per_marker_data_on_the_marker_and_heavy_data_on_the_style() {
    let factory = Arc::new(StyleFactory::new());
    let mut layer = MapLayer::new(factory.clone());
    layer.add_marker(12.5, -3.25, "Cafe", "pin", "red").unwrap();
    layer.add_marker(1.0, 2.0, "Bank", "pin", "red").unwrap();

    let first = &layer.markers()[0];
    let second = &layer.markers()[1];

    assert_eq!(first.lat(), 12.5);
    assert_eq!(first.lng(), -3.25);
    assert_eq!(first.label(), "Cafe");
    assert_eq!(second.label(), "Bank");
    assert!(Arc::ptr_eq(first.style(), second.style()));
    assert!(Arc::ptr_eq(first.style(), &factory.get("pin", "red").unwrap()));
}

#[test]
fn should_create_each_style_only_once_even_when_many_threads_ask_together() {
    let factory = Arc::new(StyleFactory::new());
    let threads = 32;
    let barrier = Arc::new(Barrier::new(threads));

    let results: Vec<Vec<usize>> = thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let factory = factory.clone();
                let barrier = barrier.clone();
                scope.spawn(move || {
                    barrier.wait();
                    let mut seen = vec![0usize; 5];
                    for _ in 0..200 {
                        for (k, slot) in seen.iter_mut().enumerate() {
                            *slot = Arc::as_ptr(&factory.get("pin", &format!("color-{k}")).unwrap()) as usize;
                        }
                    }
                    seen
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    assert_eq!(factory.instances_created(), 5);
    for k in 0..5 {
        let distinct: HashSet<usize> = results.iter().map(|seen| seen[k]).collect();
        assert_eq!(distinct.len(), 1, "styles seen for key {k}");
    }
}

#[test]
fn should_describe_each_marker_on_its_own_line() {
    let mut layer = MapLayer::new(Arc::new(StyleFactory::new()));
    layer.add_marker(12.5, -3.25, "Cafe", "pin", "red").unwrap();
    layer.add_marker(-40.0, 100.5, "Harbour", "flag", "blue").unwrap();

    assert_eq!(layer.render(), vec!["pin/red @ 12.5,-3.25 Cafe", "flag/blue @ -40,100.5 Harbour"]);
}

#[test]
fn should_reject_invalid_input_and_leave_the_layer_unchanged() {
    let factory = Arc::new(StyleFactory::new());
    let mut layer = MapLayer::new(factory.clone());

    assert!(matches!(factory.get(" ", "red"), Err(MapError::InvalidStyle)));
    assert!(matches!(factory.get("pin", ""), Err(MapError::InvalidStyle)));
    assert_eq!(layer.add_marker(91.0, 0.0, "x", "pin", "red"), Err(MapError::InvalidPosition));
    assert_eq!(layer.add_marker(0.0, -181.0, "x", "pin", "red"), Err(MapError::InvalidPosition));
    assert_eq!(layer.add_marker(0.0, 0.0, "x", "", "red"), Err(MapError::InvalidStyle));

    assert_eq!(layer.marker_count(), 0);
    assert_eq!(factory.instances_created(), 0);
}

#[test]
fn should_let_markers_be_built_directly_from_a_shared_style() {
    let factory = StyleFactory::new();
    let style = factory.get("pin", "red").unwrap();

    let a = p63_flyweight::Marker::new(1.0, 2.0, "a", style.clone()).unwrap();
    let b = p63_flyweight::Marker::new(3.0, 4.0, "b", style.clone()).unwrap();

    assert!(Arc::ptr_eq(a.style(), b.style()));
    assert_eq!(Arc::strong_count(&style), 4); // this variable + the factory's own entry + the two markers
    assert!(p63_flyweight::Marker::new(95.0, 0.0, "bad", style).is_err());
}
