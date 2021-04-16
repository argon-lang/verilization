use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::ffi::OsString;
use lang::GeneratorError;
use verilization_compiler::{lang, file_output_handler};
use sha2::Digest;
use std::convert::TryInto;

fn build_file_map(path: &Path, rel_path: &Path, map: &mut HashMap<OsString, [u8; 32]>) -> Result<(), GeneratorError> {
    
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            
            build_file_map(&entry.path(), &rel_path.join(entry.file_name()), map)?;
        }
    }
    else {
        let mut file = File::open(path)?;
        let mut hash = sha2::Sha256::default();
        let mut buf = [0u8; 4096];

        let mut len = file.read(&mut buf)?;

        while len > 0 {
            hash.update(&buf[0..len]);
            len = file.read(&mut buf)?;
        }

        let hash = hash.finalize();

        let data = hash.as_slice().try_into().map_err(|_| format!("Invalid hash: {} bytes", hash.as_slice().len()))?;
        map.insert(OsString::from(rel_path), data);
    }


    Ok(())
}

pub fn run_generator(f: impl FnOnce(&Path) -> Result<(), GeneratorError>) -> Result<HashMap<OsString, [u8; 32]>, GeneratorError> {
    let temp = tempdir::TempDir::new("verilization")?;
    let path = temp.path().canonicalize()?;
    f(&path)?;

    let mut map: HashMap<OsString, [u8; 32]> = HashMap::new();
    build_file_map(&path, Path::new(""), &mut map)?;

    Ok(map)
}


pub fn print_file_map(map: &HashMap<OsString, [u8; 32]>) {
    for (file, hash) in map {
        println!("{}: {:x?}", file.to_str().unwrap(), hash);
    }
}


