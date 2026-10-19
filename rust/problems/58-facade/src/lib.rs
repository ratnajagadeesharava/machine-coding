//! Photo upload entry point.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

pub struct PhotoUploadFacade;

impl PhotoUploadFacade {
    pub fn new(
        scanner: Box<dyn VirusScanner>,
        resizer: Box<dyn ImageResizer>,
        storage: Box<dyn PhotoStorage>,
        index: Box<dyn SearchIndex>,
    ) -> PhotoUploadFacade {
        todo!()
    }

    /// Uploads a photo. See README for the spec.
    pub fn upload(&self, file_name: &str, content: &str) -> UploadResult {
        todo!()
    }
}
