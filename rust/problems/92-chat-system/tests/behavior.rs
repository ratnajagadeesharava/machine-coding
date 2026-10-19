#![allow(unused_imports, clippy::needless_range_loop)]
use p92_chat_system::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::HashSet;
use std::sync::Arc;

fn chat() -> Option<Arc<dyn ChatService>> {
    let c = create()?;
    for u in ["ann", "bob", "cy", "dee"] {
        c.register(u);
    }
    Some(c)
}

/// "sender:text" for each history entry.
fn texts(history: Vec<String>) -> Vec<String> {
    history
        .into_iter()
        .map(|h| {
            let mut p = h.splitn(3, '|');
            let (_, sender, text) = (p.next().unwrap(), p.next().unwrap(), p.next().unwrap());
            format!("{sender}:{text}")
        })
        .collect()
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_wait_for_offline_recipients_and_deliver_on_connect() {
    let c = sut!(chat());

    let id = c.send_direct("ann", "bob", "hi").unwrap();

    assert_eq!(c.status_of(&id), Ok("SENT".to_string()));
    c.connect("bob").unwrap();
    assert_eq!(c.status_of(&id), Ok("DELIVERED".to_string()));
}

#[test]
fn should_deliver_immediately_to_online_recipients_and_then_mark_read() {
    let c = sut!(chat());
    c.connect("bob").unwrap();

    let id = c.send_direct("ann", "bob", "hi").unwrap();
    assert_eq!(c.status_of(&id), Ok("DELIVERED".to_string()));
    assert_eq!(c.unread_count("bob", "dm:ann:bob"), Ok(1));
    c.read("bob", "dm:ann:bob").unwrap();

    assert_eq!(c.status_of(&id), Ok("READ".to_string()));
    assert_eq!(c.unread_count("bob", "dm:ann:bob"), Ok(0));
}

#[test]
fn should_stop_delivering_after_disconnect() {
    let c = sut!(chat());
    c.connect("bob").unwrap();
    c.disconnect("bob").unwrap();

    let id = c.send_direct("ann", "bob", "hi").unwrap();

    assert_eq!(c.status_of(&id), Ok("SENT".to_string()));
}

#[test]
fn should_keep_direct_history_in_send_order_for_both_sides() {
    let c = sut!(chat());
    c.send_direct("ann", "bob", "one").unwrap();
    c.send_direct("bob", "ann", "two").unwrap();
    c.send_direct("ann", "bob", "three").unwrap();
    c.send_direct("cy", "ann", "other chat").unwrap();

    assert_eq!(
        texts(c.history("ann", "dm:ann:bob").unwrap()),
        strs(&["ann:one", "bob:two", "ann:three"])
    );
    assert_eq!(
        texts(c.history("bob", "dm:ann:bob").unwrap()),
        strs(&["ann:one", "bob:two", "ann:three"])
    );
    assert_eq!(
        texts(c.history("ann", "dm:ann:cy").unwrap()),
        strs(&["cy:other chat"])
    );
    assert!(c.history("dee", "dm:ann:bob").unwrap().is_empty());
}

#[test]
fn should_aggregate_group_status_over_all_recipients() {
    let c = sut!(chat());
    c.create_group("g", "ann", &["bob", "cy"]).unwrap();
    c.connect("bob").unwrap();

    let id = c.send_group("ann", "g", "hello all").unwrap();
    assert_eq!(c.status_of(&id), Ok("SENT".to_string())); // cy has not got it
    c.connect("cy").unwrap();
    assert_eq!(c.status_of(&id), Ok("DELIVERED".to_string()));
    c.read("bob", "group:g").unwrap();
    assert_eq!(c.status_of(&id), Ok("DELIVERED".to_string())); // cy has not read it
    c.read("cy", "group:g").unwrap();
    assert_eq!(c.status_of(&id), Ok("READ".to_string()));
}

#[test]
fn should_not_count_the_sender_as_a_recipient() {
    let c = sut!(chat());
    c.create_group("g", "ann", &["bob"]).unwrap();
    c.connect("bob").unwrap();

    let id = c.send_group("ann", "g", "x").unwrap();

    assert_eq!(c.unread_count("ann", "group:g"), Ok(0));
    assert_eq!(c.status_of(&id), Ok("DELIVERED".to_string()));
}

#[test]
fn should_only_let_members_send_and_only_the_admin_manage_members() {
    let c = sut!(chat());
    c.create_group("g", "ann", &["bob"]).unwrap();

    assert_eq!(c.send_group("cy", "g", "x"), Err(ChatError::NotAllowed));
    assert_eq!(c.add_member("bob", "g", "cy"), Err(ChatError::NotAllowed));
    assert_eq!(
        c.remove_member("bob", "g", "ann"),
        Err(ChatError::NotAllowed)
    );
    assert_eq!(
        c.remove_member("ann", "g", "ann"),
        Err(ChatError::NotAllowed)
    );
    assert_eq!(c.send_group("ann", "nope", "x"), Err(ChatError::Unknown));
    c.add_member("ann", "g", "cy").unwrap();
    c.send_group("cy", "g", "now I can").unwrap();
}

#[test]
fn should_show_new_members_only_messages_sent_after_they_joined() {
    let c = sut!(chat());
    c.create_group("g", "ann", &["bob"]).unwrap();
    c.send_group("ann", "g", "before").unwrap();

    c.add_member("ann", "g", "cy").unwrap();
    c.send_group("bob", "g", "after").unwrap();

    assert_eq!(
        texts(c.history("cy", "group:g").unwrap()),
        strs(&["bob:after"])
    );
    assert_eq!(
        texts(c.history("bob", "group:g").unwrap()),
        strs(&["ann:before", "bob:after"])
    );
}

#[test]
fn should_stop_receiving_after_leaving_but_keep_what_was_already_there() {
    let c = sut!(chat());
    c.create_group("g", "ann", &["bob", "cy"]).unwrap();
    c.send_group("ann", "g", "one").unwrap();

    c.remove_member("cy", "g", "cy").unwrap(); // leaves
    c.send_group("ann", "g", "two").unwrap();

    assert_eq!(
        texts(c.history("cy", "group:g").unwrap()),
        strs(&["ann:one"])
    );
    assert_eq!(
        texts(c.history("bob", "group:g").unwrap()),
        strs(&["ann:one", "ann:two"])
    );
    assert_eq!(c.send_group("cy", "g", "x"), Err(ChatError::NotAllowed));
    assert_eq!(c.unread_count("cy", "group:g"), Ok(1));
}

#[test]
fn should_ignore_messages_of_a_member_who_left_for_the_status() {
    let c = sut!(chat());
    c.create_group("g", "ann", &["bob", "cy"]).unwrap();
    c.connect("bob").unwrap();
    let id = c.send_group("ann", "g", "x").unwrap(); // cy never connects
    assert_eq!(c.status_of(&id), Ok("SENT".to_string()));

    c.remove_member("ann", "g", "cy").unwrap();
    c.read("bob", "group:g").unwrap();

    assert_eq!(c.status_of(&id), Ok("READ".to_string()));
}

#[test]
fn should_reject_unknown_users() {
    let c = sut!(chat());

    assert_eq!(c.send_direct("ann", "ghost", "x"), Err(ChatError::Unknown));
    assert_eq!(c.send_direct("ghost", "ann", "x"), Err(ChatError::Unknown));
    assert_eq!(c.connect("ghost"), Err(ChatError::Unknown));
    assert_eq!(c.status_of("m-nope"), Err(ChatError::Unknown));
}

#[test]
fn should_give_every_member_the_same_order_under_concurrent_senders() {
    let c = sut!(chat());
    c.create_group("g", "ann", &["bob", "cy", "dee"]).unwrap();
    let users = ["ann", "bob", "cy", "dee"];

    run_concurrently(8, |t| {
        for i in 0..100 {
            c.send_group(users[t % 4], "g", &format!("t{t}-{i}"))
                .unwrap();
        }
    });

    let reference = c.history("ann", "group:g").unwrap();
    assert_eq!(reference.len(), 800);
    assert_eq!(reference.iter().collect::<HashSet<_>>().len(), 800);
    for u in users {
        assert_eq!(c.history(u, "group:g").unwrap(), reference);
    }
    for t in 0..8 {
        let mut last: i64 = -1;
        for h in &reference {
            let text = h.splitn(3, '|').nth(2).unwrap();
            if let Some(n) = text.strip_prefix(&format!("t{t}-")) {
                let n: i64 = n.parse().unwrap();
                assert!(n > last);
                last = n;
            }
        }
    }
}

#[test]
fn should_deliver_every_message_exactly_once_while_users_connect_and_disconnect() {
    let c = sut!(chat());

    run_concurrently(5, |t| {
        if t == 4 {
            for _ in 0..300 {
                c.connect("bob").unwrap();
                c.disconnect("bob").unwrap();
            }
            c.connect("bob").unwrap();
            return;
        }
        for i in 0..200 {
            c.send_direct("ann", "bob", &format!("m{t}-{i}")).unwrap();
        }
    });

    let ids: HashSet<String> = c
        .history("bob", "dm:ann:bob")
        .unwrap()
        .iter()
        .map(|h| h.split('|').next().unwrap().to_string())
        .collect();
    assert_eq!(ids.len(), 800);
    for id in &ids {
        assert_eq!(c.status_of(id), Ok("DELIVERED".to_string()));
    }
    assert_eq!(c.unread_count("bob", "dm:ann:bob"), Ok(800));
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let c = sut!(chat());
        c.create_group("g", "ann", &["bob"]).unwrap();

        run_concurrently(4, |t| {
            for i in 0..20 {
                c.send_group(
                    if t % 2 == 0 { "ann" } else { "bob" },
                    "g",
                    &format!("x{t}{i}"),
                )
                .unwrap();
            }
        });

        let a = c.history("ann", "group:g").unwrap();
        assert_eq!(a.len(), 80, "run {run}");
        assert_eq!(a, c.history("bob", "group:g").unwrap(), "run {run}");
    }
}
