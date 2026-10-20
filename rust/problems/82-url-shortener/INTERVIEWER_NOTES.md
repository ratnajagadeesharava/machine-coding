# Interviewer notes — 82 URL Shortener

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- An alias that later equals a code the counter would generate (e.g. alias `abc` and the counter reaches `abc`): the generator must skip it.
- A generated code used as an alias for another URL is rejected, for the same URL it is harmless but must still not crash.
- Shortening the URL again after its code was deleted gives a new, working code (not the old one).
- Alias of length exactly 3 and exactly 20 is valid; 2, 21, a space and non-ASCII characters are not.
- URL with query/fragment is stored verbatim; the scheme check must not reject `HTTPS://` unless you decide so (state the assumption).
- Unknown code never creates a click or a counter entry.
- One million URLs still fit in 6 characters (62^4 is already 14.7 million).

## Likely interviewer questions
- What is your code alphabet and how many URLs fit in 6 characters? What happens at overflow?
- Why a counter and not a random string or a hash of the URL? What does each do under collisions?
- How do you make 'same URL -> same code' hold when two threads shorten it simultaneously?
- How does an alias interact with the generator? Can a vanity alias ever be a valid generated code?
- What does delete do to the reverse index? Who owns the click counter after delete?

## Invariants
- url_to_code and code_to_url agree: every live code appears in both, pointing at each other.
- A code never maps to two different URLs over the lifetime of the service (including after delete).
- The click counter of a live code equals the number of successful resolves of it.
- The generator counter is monotonic; a value handed out (or skipped because of an alias) is never handed out again.

## Common mistakes
- Check-then-insert on a plain map or `get` followed by `put`: two threads create two codes for the same URL.
- Counting a click before checking that the code exists, or counting on `clicks` calls.
- Reusing the counter value after a delete (e.g. `size()+1` as the next id).
- Generating a code that equals an existing alias without checking.
- Hash-of-URL codes with no collision handling, or a hash truncated to 6 characters that silently overwrites.
- Allowing `-` or `_` in generated codes (the test requires alphanumeric).

## Over-engineering traps
- A pluggable `CodeGenerator` strategy hierarchy with five implementations before the counter version passes.
- A database layer, repositories and DTOs for what is two maps and a counter.
- Bloom filters or caches before the O(1) maps are correct.

## Design and scaling discussion
- Single counter vs pre-allocated ranges per node vs hash + retry: coordination cost against predictability of codes.
- Read-heavy workload: resolve is 100x more frequent than shorten, so cache `code -> url` and shard the click counters; clicks can be eventually consistent, mappings cannot.
- Storage: a key-value store keyed by code with a secondary index on the URL hash; the reverse index is the hard part to keep unique across nodes.
- Delete semantics: tombstones prevent reissue but cost space; the code space (62^6 = 56 billion) makes that affordable.
- 301 vs 302 redirects and their effect on click counting are a product trade-off worth naming.

## Trade-off questions to ask when the candidate is done
- What breaks if two nodes each run a local counter? Which fix would you pick and why?
- Sequential codes vs random codes: what do you gain and lose (guessability, collision handling, locality)?
- Would you make `resolve` write to the click counter synchronously? What if clicks could be 1 second stale?
- How would you test the alias/generator collision without a million URLs?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
