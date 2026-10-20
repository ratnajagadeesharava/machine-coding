//! Data importer.
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use std::cell::RefCell;
use std::rc::Rc;

/// A data importer.
pub trait DataImporter {
    /// Prepare the source.
    fn open(&mut self) -> Result<(), ImportError>;

    /// The next raw record, `Ok(None)` when the source is exhausted.
    fn read_record(&mut self) -> Result<Option<String>, ImportError>;

    /// Store one record.
    fn save(&mut self, record: &str) -> Result<(), ImportError>;

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

    /// Runs one import and reports the counts.
    fn run(&mut self) -> Result<ImportReport, ImportError> {
        todo!()
    }
}

/// Imports CSV lines "name,email,city" into a shared sink.
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
