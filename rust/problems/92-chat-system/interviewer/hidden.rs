#![allow(unused_imports, clippy::needless_range_loop)]
use p92_chat_system::{contract::*, create};
use playground_support::{run_concurrently, sut};

#[test]
fn should_handle_a_large_group_with_many_messages() {
    let c = sut!(create());
    let names: Vec<String> = (0..200).map(|i| format!("u{i}")).collect();
    for n in &names {
        c.register(n);
        c.connect(n).unwrap();
    }
    c.register("admin");
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    c.create_group("big", "admin", &refs).unwrap();

    run_concurrently(10, |t| {
        for i in 0..100 {
            c.send_group(&format!("u{t}"), "big", &format!("m{t}-{i}")).unwrap();
        }
    });

    assert_eq!(c.history("u199", "group:big").unwrap().len(), 1_000);
    assert_eq!(c.unread_count("u199", "group:big"), Ok(1_000));
}

#[test]
fn should_keep_unread_counts_consistent_while_reading() {
    let c = sut!(create());
    c.register("ann");
    c.register("bob");

    run_concurrently(3, |t| {
        for _ in 0..500 {
            if t == 2 {
                c.read("bob", "dm:ann:bob").unwrap();
            } else {
                c.send_direct("ann", "bob", "x").unwrap();
            }
        }
    });
    c.read("bob", "dm:ann:bob").unwrap();

    assert_eq!(c.unread_count("bob", "dm:ann:bob"), Ok(0));
    assert_eq!(c.history("bob", "dm:ann:bob").unwrap().len(), 1_000);
}
