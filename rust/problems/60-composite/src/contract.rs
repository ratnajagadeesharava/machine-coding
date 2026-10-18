//! Provided: the component trait and the error type.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsError {
    /// Blank name, or a name containing '/'.
    InvalidName,
    /// A child with that name already exists in the folder.
    DuplicateName,
}

/// The component interface: the same operations work on a single file or on a whole folder tree.
pub trait FsNode {
    /// Name of this node (no '/').
    fn name(&self) -> &str;
    /// Total size in bytes (for a folder: everything inside it, recursively).
    fn size(&self) -> u64;
    /// Number of files (for a folder: all files inside it, recursively; a plain file counts 1).
    fn file_count(&self) -> usize;
    /// Paths of all files, relative to the PARENT of this node: a file gives `[name]`; a folder gives, for each
    /// child path `p`, `name + "/" + p`, in child insertion order (an empty folder gives an empty list).
    fn paths(&self) -> Vec<String>;
    /// The direct child with that name. Plain files have none (the default); folders override this.
    fn child(&self, _name: &str) -> Option<&dyn FsNode> {
        None
    }
}
