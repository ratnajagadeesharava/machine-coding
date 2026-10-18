//! Facade — one simple `upload` call hides a four-step workflow across four subsystems.
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

    /// Runs the whole workflow and reports one status:
    /// 1. file name non-blank and ending with ".jpg" or ".png" (any case), content non-empty; otherwise
    ///    `("REJECTED_INVALID", "")` and NO subsystem is called;
    /// 2. `scanner.is_clean(content)` false -> `("REJECTED_VIRUS", "")`; nothing else is called;
    /// 3. `resizer.resize(content, 1024)`;
    /// 4. `storage.store(file_name, resized)`; on `Err` -> `("FAILED_STORAGE", "")`, nothing is indexed;
    /// 5. `index.index(url, file_name)`; on `Err` -> `storage.delete(url)` and `("FAILED_INDEXING", "")`;
    /// 6. success -> `("OK", url)`.
    pub fn upload(&self, file_name: &str, content: &str) -> UploadResult {
        todo!()
    }
}
