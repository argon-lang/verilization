//! Defines the C API for use in bindings.
//! Most notably for WebAssembly.

use verilization_compiler::{lang, model, parser, load_all_models, VError, MemoryOutputHandler};
use lang::{GeneratorError, Language, LanguageOptions, LanguageOptionsBuilder, LanguageRegistry, LanguageHandler};

use std::ffi::{c_void, OsString};
use std::collections::HashMap;



/// Represents a string with a length followed by the UTF-8 data.
#[repr(C)]
pub struct APIString {
    length: usize,
    data: [u8; 0],
}

impl APIString {
    unsafe fn allocate(s: &str) -> *mut APIString {
        let ptr = verilization_mem_alloc(std::mem::size_of::<APIString>() + s.len());
        let api_str = ptr as *mut APIString;
        (*api_str).length = s.len();
        std::ptr::copy_nonoverlapping(s.as_ptr(), (*api_str).data.as_mut_ptr(), s.len());
        api_str
    }

    fn to_str<'a>(&'a self) -> Option<&'a str> {
        let data = &self.data as *const u8;
        unsafe { std::str::from_utf8(std::slice::from_raw_parts(data, self.length)).ok() }
    }
}

/// Represents a Result<T, String>.
/// If is_error is true, then the error field of data is inhabited.
/// Otherwise, the value field is inhabited.
#[repr(C)]
pub struct APIResult<T> {
    is_error: usize,
    data: APIResultPtr<T>,
}

/// Represents either a value or an error message.
#[repr(C)]
pub union APIResultPtr<T> {
    error: *mut APIString,
    value: *mut T,
}

/// An option defined by the name of the option and the value.
#[repr(C)]
pub struct LanguageOption {
    name: *mut APIString,
    value: *mut APIString,
}

/// An output file. Contains the file name and the content.
#[repr(C)]
pub struct OutputFileEntry {
    name: *mut APIString,
    length: usize,
    content: *mut u8,
}

/// A map of output files.
#[repr(C)]
pub struct OutputFileMap {
    length: usize,
    entries: [OutputFileEntry; 0],
}

impl OutputFileMap {
    unsafe fn allocate(map: &HashMap<String, Vec<u8>>) -> *mut OutputFileMap {
        let ptr = verilization_mem_alloc(std::mem::size_of::<OutputFileMap>() + map.len() * std::mem::size_of::<OutputFileEntry>()) as *mut OutputFileMap;
        (*ptr).length = map.len();

        let entries = std::slice::from_raw_parts_mut((*ptr).entries.as_mut_ptr(), map.len());
        for (index, (name, data)) in map.iter().enumerate() {
            let entry: &mut OutputFileEntry = &mut entries[index];
            entry.name = APIString::allocate(name);
            entry.length = data.len();

            let buffer = verilization_mem_alloc(data.len());
            entry.content = buffer;
            std::ptr::copy_nonoverlapping(data.as_ptr(), buffer, data.len());
        }

        ptr
    }
}


/// Allocates a block of memory.
///
/// Used to allocate a block of memory for values passed to verilization.
/// This is useful when hosting verilization as a WASM module.
#[no_mangle]
pub unsafe extern "C" fn verilization_mem_alloc(size: usize) -> *mut u8 {
    std::alloc::alloc(std::alloc::Layout::from_size_align(size, std::mem::size_of::<*mut c_void>()).unwrap())
}

/// Free a block of memory.
///
/// Used to free a block of memory allocated by `verilization_mem_alloc`.
/// Some values returned by functions should be freed using this function as well.
/// The size must be the same size used to allocate the memory.
#[no_mangle]
pub unsafe extern "C" fn verilization_mem_free(size: usize, ptr: *mut u8) {
    std::alloc::dealloc(ptr, std::alloc::Layout::from_size_align(size, std::mem::size_of::<*mut c_void>()).unwrap())
}

/// Parses verilization source files.
///
/// This function accepts an C-style array of strings. These strings contain the *content* of the files to be parsed.
/// A success result should be released using `verilization_destroy`.
/// An error result should be released using verilization_mem_free.
#[no_mangle]
pub unsafe extern "C" fn verilization_parse(nfiles: usize, files: *const *const APIString, result: *mut APIResult<model::Verilization>) {
    let files = std::slice::from_raw_parts(files, nfiles);

    *result = match verilization_parse_impl(files) {
        Ok(model) => APIResult {
            is_error: 0,
            data: APIResultPtr {
                value: Box::into_raw(Box::new(model)),
            },
        },
        Err(err) => APIResult {
            is_error: 1,
            data: APIResultPtr {
                error: APIString::allocate(&format!("{:?}", err)),
            },
        },
    }
}

unsafe fn verilization_parse_impl(files: &[*const APIString]) -> Result<model::Verilization, VError> {
    let models = files.iter().map(|content| {
        let content = content.as_ref().expect("Pointer was null").to_str().expect("Invalid String");
        let (_, model) = parser::parse_model(content)?;
        let model = model()?;
        Ok(model)
    });

    load_all_models(models)
}

/// Destroys a verilization model.
#[no_mangle]
pub unsafe extern "C" fn verilization_destroy(verilization: *mut model::Verilization) {
    Box::from_raw(verilization);
}

/// Generates source to handle a file format defined by a verilizaiton model.
///
/// Generates a file map containing the files generated.
/// The languge is a string indicating the language of the generated code.
/// The options are a C-style array of the language options.
/// These options are the same as the -o: flags (without the -o: prefix) to the command line interface.
/// The result and all dependent pointers must be freed using verilization_mem_free.
pub unsafe fn verilization_generate_impl<Registry: LanguageRegistry>(verilization: *const model::Verilization, language: *const APIString, noptions: usize, options: *const LanguageOption, result: *mut APIResult<OutputFileMap>, registry: &Registry) {
    *result = match verilization_generate_impl_result(verilization, language, noptions, options, registry) {
        Ok(map) => APIResult {
            is_error: 0,
            data: APIResultPtr {
                value: map,
            },
        },
        Err(err) => APIResult {
            is_error: 1,
            data: APIResultPtr {
                error: APIString::allocate(&format!("{:?}", err)),
            },
        },
    }
}


unsafe fn verilization_generate_impl_result<Registry: LanguageRegistry>(verilization: *const model::Verilization, language: *const APIString, noptions: usize, options: *const LanguageOption, registry: &Registry) -> Result<*mut OutputFileMap, GeneratorError> {
    let verilization = verilization.as_ref().expect("Verilization pointer is null");
    let language = language.as_ref().expect("Language string is null").to_str().expect("Language is invalid text");
    let options = std::slice::from_raw_parts(options, noptions)
        .iter()
        .map(|option| {
            let name = option.name.as_ref().expect("Option name is null").to_str().expect("Invalid option name text");
            let value = option.value.as_ref().expect("Option value is null").to_str().expect("Invalid option value text");
            (name, value)
        })
        .collect::<Vec<_>>();

    let mut output = MemoryOutputHandler {
        files: HashMap::new(),
    };

    match registry.handle_language(language, &mut VerilizationGenerateLang { verilization: verilization, options: options, output: &mut output, }) {
        Some(result) => result?,
        None =>Err(GeneratorError::UnknownLanguage(String::from(language)))?,
    }

    Ok(OutputFileMap::allocate(&output.files))
}

struct VerilizationGenerateLang<'a, Output> {
    verilization: &'a model::Verilization,
    options: Vec<(&'a str, &'a str)>,
    output: &'a mut Output,
}

impl <'a, Output: for<'output> lang::OutputHandler<'output>> LanguageHandler for VerilizationGenerateLang<'a, Output> {
    type Result = Result<(), GeneratorError>;

    fn run<Lang: Language>(&mut self) -> Self::Result {
        let mut lang_options = <Lang::Options as LanguageOptions>::Builder::empty();
        for (name, value) in &self.options {
            lang_options.add(name, OsString::from(value))?;
        }
        let lang_options = Lang::Options::build(lang_options)?;
    
        Lang::generate(self.verilization, lang_options, self.output)
    }
}


