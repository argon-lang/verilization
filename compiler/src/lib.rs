//! The Verilization compiler generates serialization code for various languages.

pub mod util;
pub mod model;
pub mod lang;
pub mod c_api;
mod parser;
mod type_check;
mod model_loader;

#[cfg(not(target_arch = "wasm32"))]
mod file_output_handler;

#[cfg(not(target_arch = "wasm32"))]
pub use file_output_handler::FileOutputHandler;

mod memory_output_handler;
pub use memory_output_handler::MemoryOutputHandler;

#[cfg(not(target_arch = "wasm32"))]
pub use model_loader::load_files;

pub use parser::PErrorType as ParserError;
pub use type_check::TypeCheckError;
