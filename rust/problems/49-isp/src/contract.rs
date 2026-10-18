//! Provided: three tiny, single-purpose traits and the error type.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceError {
    /// A blank document.
    BlankDocument,
    /// Fewer than one page requested.
    InvalidPages,
}

/// Something that can print. Nothing more.
pub trait Printer {
    /// Returns what came out of the printer.
    fn print(&self, document: &str) -> Result<String, DeviceError>;
}

/// Something that can scan. Nothing more.
pub trait Scanner {
    /// Returns the next scanned page.
    fn scan(&mut self) -> String;
}

/// Something that can send faxes. Nothing more.
pub trait FaxMachine {
    /// `Ok(true)` if the fax was sent, `Ok(false)` if the number was rejected.
    fn fax(&mut self, number: &str, document: &str) -> Result<bool, DeviceError>;
}
