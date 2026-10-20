# Interviewer notes — 84 In-Memory File System with Permissions

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Owner bits 000 but group/other bits permissive: the owner is still denied (first matching class, not the most permissive).
- Mode 0407: owner r--, group ---, other rwx: a group member who is not the owner is denied.
- A user with `r` on a directory but no `x` can list it but cannot reach its children; a user without `x` on an ancestor cannot even list the child directory.
- Moving a directory into its own subtree, and moving onto an existing target.
- Deleting `/`, deleting a non-empty directory, deleting a file in a directory without `w`.
- Creating over an existing entry from many threads: exactly one winner.
- A 500-level deep path: iterative walk, no stack overflow, no quadratic string handling.
- A user in several groups matches the entry's group via any of them, not only the primary one.

## Likely interviewer questions
- How do you decide which of owner/group/other applies to a user, and why is it not 'the most permissive that matches'?
- Where in your design does the check for 'x on every ancestor' live? Is it duplicated per operation?
- How do you make create/delete/move atomic when they involve one or two directories?
- Why can a move into its own subtree corrupt the tree and how do you detect it?
- What does `root` bypass, and what does it not bypass (for example: types, existence)?

## Invariants
- The structure is a tree: every node has exactly one parent, there are no cycles, and `/` is the only node without a parent.
- A failed operation (denied or invalid) changes no node.
- Names within one directory are unique and listed in sorted order.
- Permission decisions use exactly one class per (user, node), with root bypassing.

## Common mistakes
- OR-ing owner, group and other bits (or taking the first class that grants access) instead of picking the user's class first.
- Checking `x` only on the parent and not on every directory along the path.
- Creating the entry first and checking permissions afterwards (or after a partial mutation).
- Letting the same path be created twice because lookup and insert are separate steps.
- Allowing move into the entry's own subtree or onto an existing target.
- Deleting a non-empty directory recursively because it 'seemed convenient'.
- Recursive path resolution that overflows on 500-level deep trees.

## Over-engineering traps
- Abstract `INode`/`Visitor`/`Command` hierarchies and an undo log for a file system that only has nine operations.
- Implementing the setuid/setgid/sticky bits, symlinks and ACLs now.
- Hand-rolled path tries with fine-grained locks before a single-lock version passes the tests.

## Design and scaling discussion
- Locking: one global lock is correct and simple; per-directory locks raise parallelism but make rename across directories a lock-ordering problem.
- Unix really does it per-inode with a lookup cache (dentry cache); a path -> node cache would be stale after rename/chmod, which is why invalidation matters.
- Storage: metadata in memory with a write-ahead log, file content in blocks; that is where listing sorted by name becomes a B-tree.
- Users and groups would live in an identity service; the file system should cache group membership and accept staleness for a bounded time.
- Why execute-on-directory is a traversal right and not a read right: it enables 'you can open a file you know the path of but not list the directory'.

## Trade-off questions to ask when the candidate is done
- One lock for the whole tree or a lock per directory? What do you do about rename across directories?
- Store the permission check in each operation, or in a central policy function? What do you gain in testability?
- Would you check permissions before or after resolving the target? What does that leak (existence of paths)?
- How would you extend the model to ACLs without rewriting every operation?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
