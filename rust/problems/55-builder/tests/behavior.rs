use p55_builder::contract::BuildError;
use p55_builder::HttpRequest;

#[test]
fn should_apply_defaults() {
    let r = HttpRequest::builder().url("https://api.test/items").build().unwrap();

    assert_eq!(r.method(), "GET");
    assert_eq!(r.url(), "https://api.test/items");
    assert_eq!(r.timeout_millis(), 5000);
    assert!(r.headers().is_empty());
    assert_eq!(r.body(), None);
}

#[test]
fn should_build_a_full_request_step_by_step() {
    let r = HttpRequest::builder()
        .method("post")
        .url("http://api.test/items")
        .header("Content-Type", "application/json")
        .header("X-Trace", "abc")
        .body("{\"a\":1}")
        .timeout_millis(250)
        .build()
        .unwrap();

    assert_eq!(r.method(), "POST");
    assert_eq!(
        r.headers(),
        &[("content-type".to_string(), "application/json".to_string()), ("x-trace".to_string(), "abc".to_string())]
    );
    assert_eq!(r.header("CONTENT-TYPE"), Some("application/json"));
    assert_eq!(r.header("missing"), None);
    assert_eq!(r.body(), Some("{\"a\":1}"));
    assert_eq!(r.timeout_millis(), 250);
}

#[test]
fn should_append_encoded_query_parameters_in_order() {
    let r = HttpRequest::builder()
        .url("https://api.test/items")
        .query("page", "2")
        .query("q", "a b&c")
        .query("name", "caf\u{e9}")
        .build()
        .unwrap();

    assert_eq!(r.url(), "https://api.test/items?page=2&q=a%20b%26c&name=caf%C3%A9");
}

#[test]
fn should_continue_an_existing_query_string() {
    let r = HttpRequest::builder().url("https://api.test/items?sort=asc").query("page", "2").build().unwrap();

    assert_eq!(r.url(), "https://api.test/items?sort=asc&page=2");
}

#[test]
fn should_replace_a_repeated_header_in_place_ignoring_case() {
    let r = HttpRequest::builder().url("https://api.test").header("A", "1").header("B", "2").header("a", "3").build().unwrap();

    assert_eq!(r.headers(), &[("a".to_string(), "3".to_string()), ("b".to_string(), "2".to_string())]);
}

#[test]
fn should_report_bad_arguments_when_building() {
    let ok = || HttpRequest::builder().url("https://ok.test");

    assert_eq!(ok().method("FETCH").build(), Err(BuildError::InvalidMethod));
    assert_eq!(HttpRequest::builder().url("ftp://x").build(), Err(BuildError::InvalidUrl));
    assert_eq!(HttpRequest::builder().url("https://").build(), Err(BuildError::InvalidUrl));
    assert_eq!(ok().header(" ", "v").build(), Err(BuildError::InvalidName));
    assert_eq!(ok().query("", "v").build(), Err(BuildError::InvalidName));
    assert_eq!(ok().timeout_millis(0).build(), Err(BuildError::InvalidTimeout));
    assert_eq!(ok().timeout_millis(60_001).build(), Err(BuildError::InvalidTimeout));
    assert_eq!(ok().timeout_millis(60_000).build().unwrap().timeout_millis(), 60_000);
}

#[test]
fn should_remember_the_first_problem() {
    let r = HttpRequest::builder().method("FETCH").url("ftp://x").timeout_millis(0).build();

    assert_eq!(r, Err(BuildError::InvalidMethod));
}

#[test]
fn should_refuse_to_build_an_incomplete_or_inconsistent_request() {
    assert_eq!(HttpRequest::builder().build(), Err(BuildError::MissingUrl));
    assert_eq!(HttpRequest::builder().url("https://x.test").body("data").build(), Err(BuildError::BodyNotAllowed));
    assert_eq!(
        HttpRequest::builder().method("DELETE").url("https://x.test").body("data").build(),
        Err(BuildError::BodyNotAllowed)
    );
    let put = HttpRequest::builder().method("PUT").url("https://x.test").body("data").build().unwrap();
    assert_eq!(put.body(), Some("data"));
}

#[test]
fn should_let_a_builder_be_cloned_and_reused_without_affecting_earlier_requests() {
    let base = HttpRequest::builder().url("https://api.test/a").header("x-one", "1");
    let first = base.clone().build().unwrap();

    let second = base.header("x-two", "2").query("page", "9").url("https://api.test/b").build().unwrap();

    assert_eq!(first.url(), "https://api.test/a");
    assert_eq!(first.headers().len(), 1);
    assert_eq!(second.url(), "https://api.test/b?page=9");
    assert_eq!(second.headers().len(), 2);
    assert_ne!(first, second);
}
