mod util;
pub mod model;
mod parser;
pub mod lang;
mod type_check;

#[cfg(not(target_arch = "wasm32"))]
mod file_output_handler;

#[cfg(not(target_arch = "wasm32"))]
pub use file_output_handler::FileOutputHandler;

pub mod memory_output_handler;

mod model_loader;

mod c_api;

#[cfg(not(target_arch = "wasm32"))]
pub use model_loader::load_files;

pub use parser::PErrorType as ParserError;
pub use type_check::TypeCheckError;
