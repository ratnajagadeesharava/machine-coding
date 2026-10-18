use p44_yagni::contract::ShortenError;
use p44_yagni::UrlShortener;

const SOURCE: &str = include_str!("../src/lib.rs");

#[test]
fn should_hand_out_sequential_base36_codes() {
    let mut s = UrlShortener::new();

    assert_eq!(s.shorten("https://example.com/a").unwrap(), "1");
    assert_eq!(s.shorten("https://example.com/b").unwrap(), "2");
    for i in 3..=10 {
        s.shorten(&format!("http://x.io/{i}")).unwrap();
    }
    assert_eq!(s.shorten("http://x.io/last").unwrap(), "b");
}

#[test]
fn should_return_the_same_code_for_the_same_url() {
    let mut s = UrlShortener::new();

    let first = s.shorten("https://example.com/page").unwrap();
    s.shorten("https://example.com/other").unwrap();

    assert_eq!(s.shorten("https://example.com/page").unwrap(), first);
    assert_eq!(s.shorten("  https://example.com/page  ").unwrap(), first);
}

#[test]
fn should_resolve_codes_back_to_the_original_url() {
    let mut s = UrlShortener::new();
    let code = s.shorten("  https://example.com/long/path?q=1  ").unwrap();

    assert_eq!(s.resolve(&code).as_deref(), Some("https://example.com/long/path?q=1"));
    assert_eq!(s.resolve("zzz"), None);
    assert_eq!(s.resolve(""), None);
}

#[test]
fn should_reject_things_that_are_not_http_urls() {
    let mut s = UrlShortener::new();

    assert_eq!(s.shorten("ftp://example.com"), Err(ShortenError::InvalidUrl));
    assert_eq!(s.shorten("example.com"), Err(ShortenError::InvalidUrl));
    assert_eq!(s.shorten("https://"), Err(ShortenError::InvalidUrl));
    assert_eq!(s.shorten(""), Err(ShortenError::InvalidUrl));
    assert_eq!(s.shorten("https://ok.example").unwrap(), "1");
}

#[test]
fn should_expose_only_what_was_asked_for() {
    assert_eq!(UrlShortener::new().shorten("https://example.com").unwrap(), "1");

    assert_eq!(SOURCE.matches("pub fn ").count(), 3, "public functions: new, shorten, resolve (YAGNI: nothing else)");
    assert_eq!(SOURCE.matches("pub struct ").count(), 1);
    assert_eq!(SOURCE.matches("pub trait ").count(), 0);
    assert_eq!(SOURCE.matches("pub enum ").count(), 0);
}

#[test]
fn should_not_build_machinery_nobody_asked_for() {
    assert_eq!(UrlShortener::new().shorten("https://example.com").unwrap(), "1");

    let code: Vec<&str> = SOURCE.lines().map(str::trim).filter(|l| !l.starts_with("//")).collect();
    let joined = code.join("\n");
    for speculative in ["trait ", "dyn ", "Mutex", "RwLock", "Arc<", "thread", "Instant", "Duration", "rand", "async "] {
        assert!(!joined.contains(speculative), "nobody asked for: {speculative}");
    }
}
