# 84 · In-Memory File System with Permissions (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

You are building the storage core of a small multi-user, in-memory file system, like a toy Unix. Users belong to groups; every file and directory has an owner, a group and a permission mode such as `rwxr-xr-x`. Users create directories and files, read and write file content, list directories, move and delete entries, and change permissions. If a user is not allowed to do something, the call fails and the file system is left exactly as it was. The superuser `root` is allowed everything. Many sessions use the file system concurrently, so for example two users creating the same path can never both succeed.

## Problem

Build the storage core of a small multi-user, in-memory file system, in the spirit of a Unix machine. Users belong
to groups; every file and directory has an owner, a group and a mode such as `rwxr-xr-x`. Users can create directories and
files, read and write them, list a directory, rename or move entries, delete them, and change permissions. A request that the
user is not allowed to make must fail and must change nothing. There is a superuser, `root`, who may do anything. Many
sessions use the file system at once, so two users creating the same file must not both succeed.

## Requirements

1. Paths are absolute and `/`-separated (`/home/alice/a.txt`). A relative path, `.`/`..` segments, or an empty segment (`/a//b`) are invalid. `/` is the root directory.
2. `addUser(user, groups)` registers a user; the first group is the primary group. `root` always exists and may do anything. Using an unknown user in any call is an error.
3. The root directory `/` exists from the start, owned by root, and is not special-cased by the tests beyond starting with mode 0755.
4. `mkdir` creates a directory with owner = caller, group = caller's primary group, mode 0755. `createFile` does the same for a file with mode 0644 and the given content. Both need `w` and `x` on the parent directory.
5. `read` needs `r` on the file; `write` replaces the content of an existing file and needs `w` on it; `list` needs `r` on a directory and returns the entry names sorted.
6. Permission class: the owner uses the owner bits; otherwise a user whose group list contains the entry's group uses the group bits; everyone else uses the other bits. Only that one class is consulted (no fall-through to a more permissive one). Root bypasses all checks.
7. To reach any entry the user needs `x` on every directory on the path (all ancestors, starting at `/`).
8. `delete` removes a file or an EMPTY directory and needs `w`+`x` on the parent. `/` cannot be deleted.
9. `chmod` changes the mode (an octal value 0..0777) and is allowed to the owner or root. `chown(user, path, owner, group)` is allowed to root only.
10. `move` renames or moves a file or directory; the target must not exist, the user needs `w`+`x` on both parent directories, and a directory cannot be moved into its own subtree.
11. `mode`, `owner`, `group` are unchecked inspection helpers that fail only if the path does not exist.

## Constraints

- The file system is shared by many threads (Java: `Must be thread-safe`; Rust: `&self`, `Send + Sync`). Every operation is atomic: it either completes or fails without a visible partial effect.
- Exactly one of 16 threads creating the same path succeeds; 8 threads can add 2,000 entries to one directory without losing any.
- Concurrent mkdir/create/move/delete cycles in sibling directories must not deadlock or corrupt the tree (2,400 cycles in the tests).
- Paths can be 500 levels deep (hidden test): avoid recursion that overflows the stack on long paths.
- No clock or randomness; `list` returns names in ascending (lexicographic) order.

## Examples

```
users: alice[dev,ops]  bob[dev]  carol[hr]
chmod(root, "/", 0777)
mkdir(alice, "/shared")                 -> owner alice, group dev, mode 0755
createFile(alice, "/shared/f", "x")     -> mode 0644
createFile(carol, "/shared/g", "x")     -> denied (others have r-x, no w)
chmod(alice, "/shared", 0775)
createFile(bob, "/shared/g", "x")       -> ok (group dev now has rwx)
chmod(alice, "/shared/f", 0640); read(carol, "/shared/f") -> denied; read(bob, ...) -> ok
```

### Worked traces

**Modes as bits**: each octal digit is owner / group / other; `r` = 4, `w` = 2, `x` = 1. So `0o755` = `rwxr-xr-x`, `0o644` = `rw-r--r--`, `0o640` = `rw-r-----`, `0o077` = `---rwxrwx`.

**Who is checked** (`alice` in groups [dev, ops], `bob` in [dev], `carol` in [hr]; root has made `/` mode 0o777)
```
create_file(alice, "/f", "x"); chmod(alice, "/f", 0o640)    f: owner alice, group dev, rw-r-----
read(alice, "/f") ok    write(alice, "/f", "y") ok          (owner bits rw-)
read(bob, "/f") ok      write(bob, "/f", "z") -> Err(PermissionDenied)   (group bits r--)
read(carol, "/f") -> Err(PermissionDenied)                  (other bits ---)
chmod(alice, "/f", 0o077): read(alice, "/f") -> Err(PermissionDenied)    (owner bits ---; no fall-through to group/other)
chmod(alice, "/f", 0o407): alice reads ok; bob (group, bits ---) -> Err(PermissionDenied) even though other is rwx
read(root, "/f") ok                                         (root bypasses all checks)
```

**Reaching a path needs `x` on every ancestor**
```
mkdir(alice,"/a"); mkdir(alice,"/a/b"); create_file(alice,"/a/b/f","x"); chmod(alice,"/a",0o744)
read(carol, "/a/b/f") -> Err(PermissionDenied)   (carol has only r-- on /a: no x, cannot pass through)
list(carol, "/a")     -> Ok(["b"])               (listing /a needs r on /a, not x on /a)
list(carol, "/a/b")   -> Err(PermissionDenied)   (cannot reach /a/b)
```

**Errors**
```
mkdir(alice, "a/b"), "/a/../b", "/a//b"          -> Err(InvalidPath)
mkdir(alice, "/missing/child")                   -> Err(NotFound)
create_file(alice, "/f", ..) when /f exists      -> Err(AlreadyExists)
mkdir(alice, "/f/sub") with /f a file            -> Err(NotADirectory);  list(alice, "/f") -> Err(NotADirectory)
read(alice, "/")                                 -> Err(IsADirectory)
read("nobody", "/f")                             -> Err(UnknownUser)
delete(alice, "/d") with /d non-empty            -> Err(NotEmpty);  delete(alice, "/") -> Err(InvalidPath)
chmod(alice, "/f", 0o1000)                       -> Err(InvalidMode);  chown(alice, "/f", "bob", "dev") -> Err(PermissionDenied)
```

**Rename**
```
rename(alice, "/src/inner", "/dst/moved") -> Ok; /dst/moved/f is readable, list(/src) is empty
rename(alice, "/dst", "/dst/moved/loop")  -> Err(InvalidPath)     (into its own subtree)
rename(alice, "/x", "/dst")               -> Err(AlreadyExists)   (target exists)
rename(carol, "/ro/f", "/y")              -> Err(PermissionDenied) (carol cannot remove entries from /ro)
```

### Edge cases

- Failed operations change nothing: no half-created entry, no half-moved subtree.
- Only one permission class is consulted: owner if the user is the owner, otherwise group if the entry's group is in the user's group list, otherwise other. An owner with fewer bits than the group is still denied.
- `x` is needed on **every ancestor directory** of the target (starting at `/`); `list` additionally needs `r` on the directory being listed, but not `x` on it.
- Creating or deleting an entry needs `w` and `x` on the parent directory; changing a file's own content needs only `w` on the file.
- `delete` removes files and empty directories only. `/` can never be deleted.
- `chmod` accepts 0 to 0777 and is allowed to the owner or root. `chown` is for root only, even if the caller owns the entry.
- A move's target must not already exist; the user needs `w`+`x` on both the source parent and the target parent; a directory cannot move into itself or its own subtree.
- `list` returns plain entry names (not paths), sorted ascending; `/` lists its children.
- `mode`, `owner` and `group` skip all permission checks and fail only if the path does not exist.
- Every new entry is owned by the caller, with the caller's primary group (the first group given to `addUser`). `/` starts root-owned with mode 0755, so ordinary users can create nothing in `/` until root `chmod`s it.
- Racing callers: of 16 threads creating the same path exactly one succeeds; 8 threads adding 2,000 files to one directory lose none.

## Glossary

- **Mode**: the permission bits of an entry, written as an octal number (`0755`).
- **Owner / group / other**: the three classes of user a mode talks about; each has its own `r`, `w`, `x` bits.
- **Primary group**: the first group in a user's group list; new entries get it as their group.
- **`r` / `w` / `x`**: read (view a file's content, or list a directory), write (change a file's content, or add/remove entries in a directory), execute (on a directory: pass through it to reach what is inside).
- **Ancestor**: any directory on the path above an entry (`/a` and `/a/b` are ancestors of `/a/b/f`).
- **Root**: the superuser account `root`; it bypasses every permission check.
- **Empty directory**: a directory with no entries.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: bad path, missing or existing entry, wrong entry type, non-empty directory, unknown user and invalid mode throw `IllegalArgumentException`; a missing permission throws `SecurityException`.
- Rust: `FsError` with `InvalidPath`, `NotFound`, `AlreadyExists`, `NotADirectory`, `IsADirectory`, `NotEmpty`, `UnknownUser`, `PermissionDenied`, `InvalidMode`. `move` is called `rename`; modes are `u32` (`0o755`); `add_user` returns nothing.
- Error mapping the Rust tests rely on: missing parent -> `NotFound`; creating over an existing entry -> `AlreadyExists`; creating under or listing a file -> `NotADirectory`; `read` of `/` -> `IsADirectory`; delete of a non-empty directory -> `NotEmpty`; delete of `/` and moving a directory into itself -> `InvalidPath`; chmod above 0o777 -> `InvalidMode`.
- New entries get owner = caller and group = caller's primary group; the root directory `/` starts as root-owned with mode 0755, so the tests first `chmod(root, "/", 0777)` before ordinary users create entries.
- Only the matching class's bits count: mode 0077 denies the owner (and allows group and other); mode 0407 lets the owner read but denies a group member, though `other` is rwx.
- `list` of a directory needs `r` on that directory plus `x` on its ancestors, not `x` on the directory itself: with mode 0744 on `/a` an outsider can list `/a` but cannot reach `/a/b` or anything below it.
- `chmod` accepts 0..=0777; `01000` is invalid. `chown` by a non-root user is a permission error even for the owner.
- `move` fails with a permission error if the user cannot write to the source's parent (a user that cannot remove entries from `/ro` cannot move `/ro/f`).
- `mode(path)`, `owner(path)`, `group(path)` do not check permissions.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum FsError {
    /// relative path, ".", ".." or an empty segment; also deleting "/" or moving a directory into itself
    InvalidPath,
    /// the entry (or one of its parents) does not exist
    NotFound,
    AlreadyExists,
    /// a path component that must be a directory is a file
    NotADirectory,
    /// read/write on a directory
    IsADirectory,
    /// delete of a directory that still has entries
    NotEmpty,
    UnknownUser,
    PermissionDenied,
    /// chmod value above 0o777
    InvalidMode,
}

pub type Fs<T> = Result<T, FsError>;

pub trait FileSystem: Send + Sync {
    /// Registers a user. The first group is the primary group. The user "root" always exists and may do anything.
    fn add_user(&self, user: &str, groups: &[&str]);

    /// New directory: owner = user, group = user's primary group, mode 0o755. Needs w+x on the parent.
    fn mkdir(&self, user: &str, path: &str) -> Fs<()>;

    /// New file: owner = user, group = primary group, mode 0o644. Needs w+x on the parent.
    fn create_file(&self, user: &str, path: &str, content: &str) -> Fs<()>;

    /// Needs r on the file.
    fn read(&self, user: &str, path: &str) -> Fs<String>;

    /// Replaces the content of an existing file. Needs w on the file.
    fn write(&self, user: &str, path: &str, content: &str) -> Fs<()>;

    /// Names of the entries, sorted. Needs r on the directory.
    fn list(&self, user: &str, path: &str) -> Fs<Vec<String>>;

    /// Removes a file or an EMPTY directory. Needs w+x on the parent.
    fn delete(&self, user: &str, path: &str) -> Fs<()>;

    /// `mode` is the classic octal permission value (0o755 is rwxr-xr-x). Only the owner or root.
    fn chmod(&self, user: &str, path: &str, mode: u32) -> Fs<()>;

    /// Only root.
    fn chown(&self, user: &str, path: &str, new_owner: &str, new_group: &str) -> Fs<()>;

    /// Renames/moves a file or directory; the target must not exist. Needs w+x on both parents.
    fn rename(&self, user: &str, from: &str, to: &str) -> Fs<()>;

    /// Unchecked inspection helpers (no permission needed). `Err(NotFound)` if the path does not exist.
    fn mode(&self, path: &str) -> Fs<u32>;
    fn owner(&self, path: &str) -> Fs<String>;
    fn group(&self, path: &str) -> Fs<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn FileSystem>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Create/list/read in a nested tree; listing is sorted; `/` lists its children.
- Default owner, group and modes of new entries (0755 for directories, 0644 for files).
- Invalid paths (relative, `..`, double slash), missing parents, duplicate creates, creating under a file, listing a file, reading `/`, unknown users.
- Owner/group/other class selection: 0640 with a group member and an outsider; 0077 denying the owner; root overriding mode 0000 and chowning.
- `x` required on every directory of the path (mode 0744 on `/a` blocks an outsider from `/a/b/f` but allows `list /a`).
- `w`+`x` on the parent for create/mkdir/delete; delete only for empty directories; `/` cannot be deleted.
- chmod owner-only, chown root-only, chmod range check; move of a directory, move into its own subtree, existing target, and a permission failure on the source parent.
- Concurrency: one winner among 16 creators of one path; 2,000 files added by 8 threads in one directory; 8 threads x 300 create/move/delete cycles end with an empty `/`.
- More edge cases (500-level deep paths, first-matching-class permission semantics) are in `interviewer/`.

## Running

```bash
cargo test -p p84_file_system_permissions
STRICT=1 cargo test -p p84_file_system_permissions   # fail instead of skip while unimplemented
cargo test -p p84_file_system_permissions --features hidden   # interviewer scenarios
cargo play run 84              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.
