//! The Verilization compiler generates serialization code for various languages.

pub mod util;
pub mod model;
pub mod lang;
pub mod parser;
mod type_check;
mod model_loader;
mod verror;
mod memory_output_handler;


pub use memory_output_handler::MemoryOutputHandler;
pub use model_loader::load_all_models;

pub use verror::VError;
pub use type_check::TypeCheckError;

#[cfg(not(target_arch = "wasm32"))]
mod file_output_handler;

#[cfg(not(target_arch = "wasm32"))]
pub use file_output_handler::FileOutputHandler;

#[cfg(not(target_arch = "wasm32"))]
pub use model_loader::load_files;

