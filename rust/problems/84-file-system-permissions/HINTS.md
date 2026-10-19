# Hints — 84 In-Memory File System with Permissions (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

What does the path `/a/b/f` ask of each of the directories it passes through, before you even look at `f` itself?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Model the tree as nodes (file or directory) with owner, group and mode, resolve a path step by step from the root, and put the permission decision in one function that answers 'may this user do this operation on this node?'. Keep the three operations on a parent (create, delete, move) consistent.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Node { kind, owner, group, mode, content or children (sorted map) }`; `resolve(path)` walks the segments and checks `x` on every directory it enters; `allowed(user, node, r|w|x)` picks one class (owner, else group, else other) and tests that class only, with a root shortcut. Operations resolve the parent, check the parent permission, then mutate. Make each operation one critical section so check and mutation are not split, and for move check that the target is not inside the source before touching anything.

**Rust:** Use an enum `Node { File{..}, Dir{ children: BTreeMap<String, Node> } }` behind one `RwLock`; walk with iterators over `path.split('/')` instead of recursion so 500-level paths are not a stack problem. Octal literals are `0o755`.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
