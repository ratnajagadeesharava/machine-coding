# Interviewer notes — 110 Runtime Data Source Access Layer

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A refused API put over an existing key keeps the old value and the old size.
- Whitespace-only key on FILE is blank (error), not an empty-string key.
- API case-insensitivity does not trim; FILE trimming does not fold case; DATABASE does neither.
- A 40-character value stored on DATABASE survives a trip through API (the limit is on writes only).
- A failed switch (unknown name, wrong case, null) leaves the current source, and where later writes land, unchanged.
- Thousands of switches lose no record in any source.
- Writers racing a switcher: every successfully written key is in exactly one source afterwards (records conserved).
- Readers racing a switcher on a key present in all sources never see a missing value or an exception.

## Likely interviewer questions
- Where do the source-specific rules live, and what does the calling code know about them?
- What exactly is switched: a reference, a flag, a name? What does an in-flight call see?
- Why are the records of each source kept separately instead of one map keyed by (source, key)? Would either be wrong?
- What is the validation order (blank key vs. refused value vs. unknown source) and does it matter?
- What does `size()` mean if a key is normalized (trim, lower case)?

## Invariants
- At every instant there is exactly one current source.
- Each call reads the current source once and then works on that source only.
- A refused operation changes nothing.
- Records of a source change only through calls made while it was current.
- The value returned is the value stored, byte for byte.

## Common mistakes
- One shared map with the source folded into the key by the caller (switch code in business paths).
- `if (source.equals("API")) ... else if ...` scattered through get/put/remove/size instead of one place per source.
- Reading the current source several times inside one call (check on source A, write on source B).
- Normalizing the key on `put` but not on `get`/`remove` (FILE trim, API case).
- Lower-casing the stored value for API, or trimming values on FILE.
- Applying the 32-character limit to reads/switches or counting bytes instead of characters.
- Clearing a source's records when switching to it, or when switching away.
- Taking a single lock around everything so readers block each other (works, but fails the performance discussion).
- Accepting `"api"` or `" API"` as source names.

## Over-engineering traps
- A plugin system, reflection-based discovery, dependency injection container before a second source type is requested.
- An event bus or observer for switch notifications nobody asked for.
- A class hierarchy of validators when three small rules would do.
- Persisting to a real file or database.

## Design and scaling discussion
- Business code depends on one narrow interface; sources are interchangeable behind it and own their rules. Adding a source means adding one class and registering its name in one place.
- A volatile/atomic reference to the current source gives lock-free reads; each source's own concurrent map gives per-key atomicity. A global lock is simpler but serializes the hot path.
- Atomic switch semantics: calls started before the switch complete on the old source; calls started after see the new one. A call that races the switch lands on either, never on both.
- Copy-on-switch (V4) and read-through (V3) both break the "sources are independent" guarantee and need a stated consistency story.
- In production the switch would be a configuration change with an audit record and a health check on the target.

## Trade-off questions to ask when the candidate is done
- Enum of sources vs. registry by name: what do you gain and lose when a fourth source arrives from another team?
- Where would you reject the 33-character value, in the source or in a validating wrapper? Who knows the limit?
- Should the switch wait for in-flight writes to the old source to finish? What does that cost?
- How would you test the "never lands on two sources" property without sleeps or flakiness?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
