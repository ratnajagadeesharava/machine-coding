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
