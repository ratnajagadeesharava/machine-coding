# 60 · Composite (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `FsNode` trait and `FsError` provided in `src/contract.rs`; the rest is yours to write. |

## Problem

Model a tiny in-memory file system. Folders contain files and other folders (nested to any depth); callers ask "how big
is this?", "how many files?" and "list everything" without caring whether they hold a file or a folder. The provided
`FsNode` trait has `name()`, `size()`, `file_count()`, `paths()` and `child(name)`; you write `File` and `Directory`.
Folders must also work with any other `FsNode` implementation a caller supplies.

## Terms

- **Path**: names joined by `/`, e.g. `docs/deep/c.txt`.
- **Relative to the parent**: `paths()` includes the node's own name as the first segment (see below).
- **Blank**: empty or only whitespace.

## Requirements

1. `File::new(name, size_bytes)`: blank name or a name containing `/` → `Err(FsError::InvalidName)` (size 0 is allowed). `name()` is the name, `size()` its size, `file_count()` is 1, `paths()` is `[name]`.
2. `Directory::new(name)` (same name rule). `size()` and `file_count()` are recursive sums over everything below; an empty folder has size 0, 0 files and an empty `paths()`.
3. `paths()` of a folder: every file below it as `folder/…/file`, starting with the folder's own name, in insertion order (depth-first through children in the order they were added), e.g. `root/a.txt`, `root/docs/b.txt`. Empty folders contribute nothing.
4. `add(child)` (`Box<dyn FsNode>`): a name already used by a direct child of this folder → `Err(FsError::DuplicateName)` and the folder is unchanged. Children may be any `FsNode`, not only the two types you write. (Cycles cannot arise: `add` takes ownership of the child.)
5. `remove(name)` removes and returns the direct child with that name (`Some`), or `None` if there is none. `child_names()` lists the direct children's names in insertion order.
6. `find(path)` returns `Option<&dyn FsNode>`: it follows a `/`-separated path relative to the folder (not including the folder's own name), e.g. `"docs/deep/c.txt"`; `None` if a step is missing, goes through a file, or the path is blank. A folder's `child(name)` returns its direct child with that name (plain files have none).

## Examples

Tree: `root` { `a.txt` (100), `docs` { `b.txt` (250), `deep` { `c.txt` (50) } }, `empty` { } }.

| Call | Result |
|---|---|
| `root.size()` / `root.file_count()` | `400` / `3` |
| `root.paths()` | `root/a.txt`, `root/docs/b.txt`, `root/docs/deep/c.txt` |
| `root.find("docs")` → `size()` / `file_count()` | `300` / `2` |
| `root.find("docs/deep/c.txt")` → `name()` | `c.txt` |
| `root.find("docs/missing.txt")`, `root.find("a.txt/inside")`, `root.find(" ")` | `None` each |
| `root.add(file "a.txt")` | `Err(DuplicateName)`; tree unchanged |
| `root.remove("docs")` twice | `Some(..)` then `None`; afterwards `size()` is 100 and `child_names()` is `a.txt`, `empty` |
| add a custom `FsNode` of size 4, 1 file, named `link` | `root.size()` 404, `file_count()` 4, last path `root/link`, `find("link")` is `Some` |

## Running

```bash
cargo test -p p60_composite     # from rust/ — tests FAIL until you implement the stubs
cargo play run 60
```
