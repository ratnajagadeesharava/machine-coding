//! Provided: the prototype trait and the error types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlueprintError {
    /// Disk index out of range, or size of 0 GB.
    InvalidDisk,
    /// Blank blueprint name.
    InvalidName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    BlankKey,
    BlankName,
    UnknownKey,
}

/// A server blueprint.
pub trait Blueprint {
    fn name(&self) -> &str;
    fn rename(&mut self, new_name: &str);
    /// "web", "db", ...
    fn kind(&self) -> &str;
    /// Blank tags and tags already present are ignored.
    fn add_tag(&mut self, tag: &str);
    /// Tags in insertion order.
    fn tags(&self) -> Vec<String>;
    /// Disk sizes in GB, in order.
    fn disk_sizes_gb(&self) -> Vec<u32>;
    /// Index out of range or 0 GB -> `Err(InvalidDisk)`.
    fn resize_disk(&mut self, index: usize, new_size_gb: u32) -> Result<(), BlueprintError>;
    /// A completely independent DEEP copy of the same concrete type, with the same name, tags and disk sizes.
    /// Changing the copy never changes the original, and vice versa.
    fn clone_box(&self) -> Box<dyn Blueprint>;
}
