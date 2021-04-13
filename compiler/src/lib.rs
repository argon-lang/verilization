#![feature(generic_associated_types)]
pub mod model;
mod parser;
pub mod lang;
mod type_check;
pub mod file_output_handler;
mod model_loader;

pub use model_loader::load_files;
pub use parser::PErrorType as ParserError;
pub use type_check::TypeCheckError;


use lang::GeneratorError;

use std::ffi::{CString, CStr, c_void, OsString};
use std::os::raw::c_char;
use std::ptr;
use std::io;

// C API

#[repr(C)]
pub struct LanguageOption {
    name: *const c_char,
    value: *const c_char,
}

#[repr(C)]
pub struct FileWriteHandlers {
    create: unsafe extern "C" fn(ctx: *mut c_void, name: *const c_char) -> *mut c_void,
    write: unsafe extern "C" fn(handle: *mut c_void, size: usize, value: *const u8) -> bool,
    flush: unsafe extern "C" fn(handle: *mut c_void) -> bool,
    close: unsafe extern "C" fn(handle: *mut c_void),
}

struct COutputHandler<'a> {
    handlers: &'a FileWriteHandlers,
    ctx: *mut c_void,
}

struct CFileHandle<'a> {
    handlers: &'a FileWriteHandlers,
    handle: *mut c_void,
}

impl <'a> lang::OutputHandler for COutputHandler<'a> {
    type FileHandle<'b> = CFileHandle<'b>;
    fn create_file<'b, P : AsRef<std::path::Path>>(&'b mut self, name: P) -> Result<CFileHandle<'b>, GeneratorError> {
        let name = CString::new(name.as_ref().to_str().unwrap()).unwrap();
        let handle = unsafe { (self.handlers.create)(self.ctx, name.as_ptr()) };
        if handle.is_null() {
            Err(GeneratorError::from("Could not open file."))
        }
        else {
            Ok(CFileHandle {
                handlers: self.handlers,
                handle: handle,
            })
        }
    }
}

impl <'a> io::Write for CFileHandle<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let ok = unsafe { (self.handlers.write)(self.handle, buf.len(), buf.as_ptr()) };
        if ok {
            Ok(buf.len())
        }
        else {
            Err(io::Error::new(io::ErrorKind::Other, "Error writing"))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let ok = unsafe { (self.handlers.flush)(self.handle) };
        if ok {
            Ok(())
        }
        else {
            Err(io::Error::new(io::ErrorKind::Other, "Error writing"))
        }
    }
}

impl <'a> Drop for CFileHandle<'a> {
    fn drop(&mut self) {
        unsafe { (self.handlers.close)(self.handle) }
    }
}


// Parses source files
#[no_mangle]
pub unsafe extern "C" fn verilization_parse(nfiles: usize, files: *const *const c_char, verilization: *mut *mut model::Verilization) -> *mut c_char {
    let files = std::slice::from_raw_parts(files, nfiles);

    match verilization_parse_impl(files) {
        Ok(model) => {
            *verilization = Box::into_raw(Box::new(model));
            ptr::null_mut()
        },
        Err(err) => {
            *verilization = ptr::null_mut();
            CString::new(format!("{:?}", err)).unwrap().into_raw()
        },
    }
}

unsafe fn verilization_parse_impl(files: &[*const c_char]) -> Result<model::Verilization, GeneratorError> {
    let models = files.iter().map(|content| {
        let content = CStr::from_ptr(*content).to_str().unwrap();
        parser::parse_model(content)
            .map(|(_, model)| model)
            .map_err(GeneratorError::from)
    });

    model_loader::load_all_models(models)
}


#[no_mangle]
pub unsafe extern "C" fn verilization_error_destroy(error: *mut c_char) {
    CString::from_raw(error);
}

#[no_mangle]
pub unsafe extern "C" fn verilization_destroy(verilization: *mut model::Verilization) {
    Box::from_raw(verilization);
}

#[no_mangle]
pub unsafe extern "C" fn verilization_generate(verilization: *const model::Verilization, language: *const c_char, noptions: usize, options: *const LanguageOption, handlers: *const FileWriteHandlers, ctx: *mut c_void) -> *mut c_char {
    let verilization = verilization.as_ref().unwrap();
    let language = CStr::from_ptr(language).to_str().unwrap();
    let options = std::slice::from_raw_parts(options, noptions);

    let mut output = COutputHandler {
        handlers: handlers.as_ref().unwrap(),
        ctx: ctx,
    };

    let result = match language {
        "typescript" => verilization_generate_lang::<lang::typescript::TypeScriptLanguage, _>(verilization, options, &mut output),
        "java" => verilization_generate_lang::<lang::java::JavaLanguage, _>(verilization, options, &mut output),
        "scala" => verilization_generate_lang::<lang::scala::ScalaLanguage, _>(verilization, options, &mut output),
        _ => Err(GeneratorError::from(format!("Unknown language: {}", language))),
    };

    match result {
        Ok(()) => ptr::null_mut(),
        Err(err) => CString::new(format!("{:?}", err)).unwrap().into_raw(),
    }
}

unsafe fn verilization_generate_lang<Lang: lang::Language, Output: lang::OutputHandler>(verilization: &model::Verilization, options: &[LanguageOption], output: &mut Output) -> Result<(), GeneratorError> {
    let mut lang_options = Lang::empty_options();
    for option in options {
        let name = CStr::from_ptr(option.name).to_str().unwrap();
        let value = CStr::from_ptr(option.value).to_str().unwrap();
        Lang::add_option(&mut lang_options, name, OsString::from(value))?;
    }
    let lang_options = Lang::finalize_options(lang_options)?;

    Lang::generate(verilization, lang_options, output)
}


