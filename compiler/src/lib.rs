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
