use std::fs;
use std::fs::File;
use std::path::Path;
use crate::lang::{OutputHandler, GeneratorError};


pub struct FileOutputHandler {}

impl OutputHandler for FileOutputHandler {
    type FileHandle<'state> = File;
    fn create_file<'state, P: AsRef<Path>>(&'state mut self, path: P) -> Result<Self::FileHandle<'state>, GeneratorError> {
        if let Some(dir) = path.as_ref().parent() {
            fs::create_dir_all(dir)?;
        }

        Ok(File::create(path)?)
    }
}
