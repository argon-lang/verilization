use std::collections::HashMap;
use std::path::Path;
use crate::lang::{OutputHandler, GeneratorError};


pub struct MemoryOutputHandler {
    pub files: HashMap<String, Vec<u8>>,
}

impl <'output> OutputHandler<'output> for MemoryOutputHandler {
    type FileHandle = &'output mut Vec<u8>;
    fn create_file<P: AsRef<Path>>(&'output mut self, path: P) -> Result<Self::FileHandle, GeneratorError> {
        let filename = path.as_ref().to_str().ok_or("Invalid filename")?.to_string();
        Ok(self.files.entry(filename).or_insert_with(Vec::new))
    }
}
