#![allow(unused_imports, clippy::needless_range_loop)]
use p82_url_shortener::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::HashSet;

#[test]
fn should_handle_300k_urls_with_short_codes() {
    let s = sut!(create());
    let mut codes = HashSet::new();

    for i in 0..300_000 {
        let code = s.shorten(&format!("https://example.com/{i}")).unwrap();
        assert!(code.len() <= 6);
        codes.insert(code);
    }

    assert_eq!(codes.len(), 300_000);
}

#[test]
fn should_not_let_aliases_collide_with_generated_codes_under_load() {
    let s = sut!(create());

    run_concurrently(8, |t| {
        for i in 0..2_000 {
            s.shorten_with_alias(
                &format!("https://example.com/alias/{t}/{i}"),
                &format!("alias-{t}-{i}"),
            )
            .unwrap();
            s.shorten(&format!("https://example.com/gen/{t}/{i}"))
                .unwrap();
        }
    });

    for t in 0..8 {
        assert_eq!(
            s.resolve(&format!("alias-{t}-5")),
            Some(format!("https://example.com/alias/{t}/5"))
        );
    }
}
