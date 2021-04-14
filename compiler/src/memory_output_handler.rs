use std::collections::HashMap;
use std::path::Path;
use crate::lang::{OutputHandler, GeneratorError};


pub struct MemoryOutputHandler {
    pub files: HashMap<String, Vec<u8>>,
}

impl OutputHandler for MemoryOutputHandler {
    type FileHandle<'state> = &'state mut Vec<u8>;
    fn create_file<'state, P: AsRef<Path>>(&'state mut self, path: P) -> Result<Self::FileHandle<'state>, GeneratorError> {
        let filename = path.as_ref().to_str().ok_or("Invalid filename")?.to_string();
        Ok(self.files.entry(filename).or_insert_with(Vec::new))
    }
}
