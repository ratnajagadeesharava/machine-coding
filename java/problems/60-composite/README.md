# 60 · Composite (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `FsNode` component interface is provided; the rest is yours to write. |

## Problem

Model a tiny in-memory file system. Folders contain files and other folders (nested to any depth); callers ask "how big
is this?", "how many files?" and "list everything" without caring whether they hold a file or a folder. The provided
`FsNode` interface has `name()`, `size()`, `fileCount()` and `paths()`; you write `FileNode` and `DirectoryNode`.
Folders must also work with any other `FsNode` implementation a caller supplies.

## Terms

- **Path**: names joined by `/`, e.g. `docs/deep/c.txt`.
- **Relative to the parent**: `paths()` includes the node's own name as the first segment (see below).
- **Cycle**: a folder that would end up inside itself, directly or through descendants.

## Requirements

1. `FileNode(name, sizeBytes)`: blank name, a name containing `/`, or a negative size → `IllegalArgumentException` (size 0 is allowed). `name()` is the name, `size()` its size, `fileCount()` is 1, `paths()` is `[name]`.
2. `DirectoryNode(name)` (same name rules). `size()` and `fileCount()` are recursive sums over everything below; an empty folder has size 0, 0 files and an empty `paths()`.
3. `paths()` of a folder: every file below it as `folder/…/file`, starting with the folder's own name, in insertion order (depth-first through children in the order they were added), e.g. `root/a.txt`, `root/docs/b.txt`. Empty folders contribute nothing.
4. `add(child)`: `null`, a name already used by a direct child of this folder, or a **cycle** (adding the folder to itself, or adding any folder that contains this one) → `IllegalArgumentException`, and nothing changes. Children may be any `FsNode`, not only the two classes you write.
5. `remove(name)` removes the direct child with that name and returns whether it existed. `children()` returns a read-only view of the direct children in insertion order (modifying it → `UnsupportedOperationException`).
6. `find(path)` returns an `Optional<FsNode>`: it follows a `/`-separated path relative to the folder (not including the folder's own name), e.g. `"docs/deep/c.txt"`; empty if a step is missing or goes through a file; a blank path → `IllegalArgumentException`.

## Examples

Tree: `root` { `a.txt` (100), `docs` { `b.txt` (250), `deep` { `c.txt` (50) } }, `empty` { } }.

| Call | Result |
|---|---|
| `root.size()` / `root.fileCount()` | `400` / `3` |
| `root.paths()` | `root/a.txt`, `root/docs/b.txt`, `root/docs/deep/c.txt` |
| `root.find("docs")` → `size()` / `fileCount()` | `300` / `2` |
| `root.find("docs/deep/c.txt")` → `name()` | `c.txt` |
| `root.find("docs/missing.txt")`, `root.find("a.txt/inside")` | empty, empty |
| `root.find(" ")` | `IllegalArgumentException` |
| `root.add(new FileNode("a.txt", 1))` | `IllegalArgumentException` (duplicate) |
| `root.add(root)`, or `deep.add(root)` / `deep.add(docs)` | `IllegalArgumentException` (cycle); tree unchanged |
| `root.remove("docs")` twice | `true` then `false`; afterwards `size()` is 100 and `children()` names are `a.txt`, `empty` |
| add a custom `FsNode` of size 4, 1 file, named `link` | `root.size()` 404, `fileCount()` 4, `paths()` ends with `root/link` |

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
