#![feature(generic_associated_types)]

mod util;
pub mod model;
mod parser;
pub mod lang;
mod type_check;

#[cfg(not(target_arch = "wasm32"))]
pub mod file_output_handler;

pub mod memory_output_handler;

mod model_loader;

mod c_api;

#[cfg(not(target_arch = "wasm32"))]
pub use model_loader::load_files;

pub use parser::PErrorType as ParserError;
pub use type_check::TypeCheckError;
