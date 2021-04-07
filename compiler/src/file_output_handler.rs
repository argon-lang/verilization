use std::fs;
use std::fs::File;
use std::path::Path;
use crate::lang::{OutputHandler, GeneratorError};


pub struct FileOutputHandler {}

impl <'a> OutputHandler<'a> for FileOutputHandler {
    type FileHandle = File;
    fn create_file<P: AsRef<Path>>(&'a mut self, path: P) -> Result<Self::FileHandle, GeneratorError> {
        if let Some(dir) = path.as_ref().parent() {
            fs::create_dir_all(dir)?;
        }

        Ok(File::create(path)?)
    }
}
