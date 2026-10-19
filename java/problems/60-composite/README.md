# 60 · Composite (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `FsNode` component interface is provided; the rest is yours to write. |

## Problem

Model a tiny file system. Folders contain files and other folders; callers ask "how big is this?", "how many files?"
and "list everything" without caring whether they hold a file or a folder.

## Requirements

1. `FileNode(name, sizeBytes)`: blank name, a name containing `/`, or a negative size → `IllegalArgumentException`. `size()` is its size, `fileCount()` is 1, `paths()` is `[name]`.
2. `DirectoryNode(name)` (same name rules). `size()` and `fileCount()` are recursive sums; an empty folder has size 0 and no files.
3. `paths()` of a folder: every file below it as `folder/…/file`, in insertion order, e.g. `root/a.txt`, `root/docs/b.txt`.
4. `add(child)`: `null`, a name already used in this folder, or a **cycle** (adding the folder to itself, or adding any folder that contains this one) → `IllegalArgumentException`, and nothing changes.
5. `remove(name)` returns whether it existed. `children()` is a read-only view in insertion order.
6. `find("docs/deep/c.txt")` follows a `/`-separated path relative to the folder; empty if a step is missing or goes through a file; a blank path → `IllegalArgumentException`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
