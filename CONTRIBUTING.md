# Authoring guide: no hints in the problem files

The point of this repo is that the candidate does the thinking. A problem ships the **what**, never the **how**.

## README.md
Keep: title, difficulty/duration/scaffolding table, **Problem**, **Requirements**, **Running**.

- Requirements state observable behaviour only: values, thresholds, formats, error cases, ordering that a caller can see.
- The scaffolding row says what is provided and that the rest is yours to write. No design advice.
- Do not add: a "Pattern:" / "Principle:" explanation, "Rust adaptation" tips, a "Constraints (what the tests catch)" section, or wording such as "must be final/abstract", "not an if/switch", "use RefCell/OnceCell".

## Stub source files
- At most a one-line role comment ("Card payment gateway."). No steps, ordering, pitfalls or pattern mechanics.
- Keep signatures and the `todo!()` / `UnsupportedOperationException("Not implemented yet")` bodies.
- Provided `contract` files may state facts (return-code tables), not implementation advice.

## Tests
- Structural (reflection) checks are fine, but their failure messages must name the rule that was broken, not the fix.

## Where help lives
Optional help goes in `HINTS.md` (three levels, smallest nudge first, collapsed), opened on purpose. Never in code or README.

## Verify before committing
```bash
cd rust && cargo check --workspace --tests
# Java: mvn test per problem, or javac over src/main when offline
```
