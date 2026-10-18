//! Template Method — the trait's provided `run` is the fixed algorithm; implementors supply the steps.
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use std::cell::RefCell;
use std::rc::Rc;

/// The template. `run` is the algorithm; implementors only fill in the steps.
///
/// ```text
/// open()?;                                  // if this fails, run returns the error and close() is NOT called
/// loop over read_record():                  // an Err here: close(), then return the error
///   read += 1
///   if !validate(record) { skipped += 1; continue }
///   transform(record) then save(..): both Ok -> saved += 1, any Err -> failed += 1 and carry on
/// close()                                   // always runs once open() succeeded
/// Ok(ImportReport { read, saved, skipped, failed })
/// ```
pub trait DataImporter {
    // ---- steps every importer must provide ----

    /// Prepare the source (open a file, connect...).
    fn open(&mut self) -> Result<(), ImportError>;

    /// The next raw record, `Ok(None)` when the source is exhausted.
    fn read_record(&mut self) -> Result<Option<String>, ImportError>;

    /// Store one transformed record.
    fn save(&mut self, record: &str) -> Result<(), ImportError>;

    // ---- steps with a sensible default (hooks) ----

    /// Default: a record is valid when it is not blank.
    fn validate(&mut self, record: &str) -> bool {
        todo!()
    }

    /// Default: the record unchanged.
    fn transform(&mut self, record: &str) -> Result<String, ImportError> {
        todo!()
    }

    /// Default: does nothing.
    fn close(&mut self) {
        todo!()
    }

    // ---- the template itself: write it once, here ----

    fn run(&mut self) -> Result<ImportReport, ImportError> {
        todo!()
    }
}

/// Imports CSV lines "name,email,city". The first line is a header and is skipped. A record is valid when it has
/// exactly 3 comma-separated fields (empty trailing fields count) and none of them is blank after trimming. Valid
/// records are transformed by trimming every field and joining them with "|" ("ann , a@x.io, Rome" ->
/// "ann|a@x.io|Rome"), and saved by appending to the shared sink.
pub struct CsvImporter;

impl CsvImporter {
    pub fn new(lines: Vec<String>, sink: Rc<RefCell<Vec<String>>>) -> CsvImporter {
        todo!()
    }
}

impl DataImporter for CsvImporter {
    fn open(&mut self) -> Result<(), ImportError> {
        todo!()
    }
    fn read_record(&mut self) -> Result<Option<String>, ImportError> {
        todo!()
    }
    fn save(&mut self, record: &str) -> Result<(), ImportError> {
        todo!()
    }
    fn validate(&mut self, record: &str) -> bool {
        todo!()
    }
    fn transform(&mut self, record: &str) -> Result<String, ImportError> {
        todo!()
    }
}
