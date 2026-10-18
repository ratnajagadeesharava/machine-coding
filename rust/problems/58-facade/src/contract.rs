//! Provided: the four subsystems and the result type.

pub trait VirusScanner {
    fn is_clean(&self, content: &str) -> bool;
}

pub trait ImageResizer {
    /// Returns the image scaled down so that it is at most `max_width` pixels wide.
    fn resize(&self, content: &str, max_width: u32) -> String;
}

pub trait PhotoStorage {
    /// Stores the file and returns its URL, or an error message when the disk or network fails.
    fn store(&self, file_name: &str, content: &str) -> Result<String, String>;
    fn delete(&self, url: &str);
}

pub trait SearchIndex {
    /// Fails with a message when the search cluster is down.
    fn index(&self, url: &str, file_name: &str) -> Result<(), String>;
}

/// `url` is "" unless the status is "OK".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadResult {
    pub status: String,
    pub url: String,
}
