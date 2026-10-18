# 60 · Composite (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `FsNode` trait and `FsError` are provided in `src/contract.rs`; `File` (leaf) and `Directory` (composite) are stubs. |

> **Pattern:** *Composite* — compose objects into tree structures and let clients treat **individual objects and
> compositions uniformly**. `size()` on a file and `size()` on a folder with 10,000 files is the same call.
>
> **Rust adaptation:** a folder owns `Vec<Box<dyn FsNode>>`. Because adding a folder to another **moves** it, a
> cycle cannot be built, so (unlike the Java version) there is no cycle check. The trait's `child` method (default
> `None`, overridden by `Directory`) lets `find` walk the tree without downcasting.

## Problem

Model a tiny file system. Folders contain files and other folders; callers ask "how big is this?", "how many files?"
and "list everything" without caring whether they hold a file or a folder.

## Requirements

1. `File::new(name, size_bytes)`: blank name or a name containing `/` → `Err(FsError::InvalidName)`. `size()` is its size, `file_count()` is 1, `paths()` is `[name]`.
2. `Directory::new(name)` (same name rule). `size()` and `file_count()` are recursive sums; an empty folder has size 0 and no files.
3. `paths()` of a folder: every file below it as `folder/…/file`, in insertion order, e.g. `root/a.txt`, `root/docs/b.txt`.
4. `add(child)`: a name already used in this folder → `Err(FsError::DuplicateName)` and the folder is unchanged.
5. `remove(name)` returns the removed node (or `None`). `child_names()` lists the direct children in insertion order.
6. `find("docs/deep/c.txt")` follows a `/`-separated path relative to the folder; `None` if a step is missing, goes through a file, or the path is blank.

## Constraints (what the tests catch)

- The tests add a `Shortcut` node defined in the test file to a folder. A folder that only understands `File`/`Directory` fails.

## Running

```bash
cargo test -p p60_composite     # from rust/ — tests FAIL until you implement the stubs
cargo play run 60
```
