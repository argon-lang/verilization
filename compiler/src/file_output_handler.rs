use std::fs;
use std::fs::File;
use std::path::Path;
use crate::lang::{OutputHandler, GeneratorError};


pub struct FileOutputHandler {}

impl <'output> OutputHandler<'output> for FileOutputHandler {
    type FileHandle = File;
    fn create_file<P: AsRef<Path>>(&'output mut self, path: P) -> Result<Self::FileHandle, GeneratorError> {
        if let Some(dir) = path.as_ref().parent() {
            fs::create_dir_all(dir)?;
        }

        Ok(File::create(path)?)
    }
}
