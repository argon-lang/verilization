pub mod typescript;
pub mod java;
pub mod scala;

use crate::model;
use crate::parser::PErrorType;
use crate::type_check::TypeCheckError;
use std::ffi::OsString;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub enum GeneratorError {
	ParseError(PErrorType<String>),
	TypeCheckError(TypeCheckError),
	IOError(io::Error),
	CustomError(String),
}

impl From<PErrorType<&str>> for GeneratorError {
	fn from(err: PErrorType<&str>) -> Self {
		GeneratorError::ParseError(match err {
			PErrorType::ParseError(str, error) => PErrorType::ParseError(str.to_string(), error),
			PErrorType::DuplicateVersion(str, type_name, version) => PErrorType::DuplicateVersion(str.to_string(), type_name, version),
			PErrorType::DuplicateField(str, version, field_name) => PErrorType::DuplicateField(str.to_string(), version, field_name),
			PErrorType::DuplicateConstant(name) => PErrorType::DuplicateConstant(name),
			PErrorType::DuplicateType(name) => PErrorType::DuplicateType(name),
		})
	}
}

impl From<io::Error> for GeneratorError {
	fn from(err: io::Error) -> Self {
		GeneratorError::IOError(err)
	}
}

impl From<String> for GeneratorError {
	fn from(str: String) -> Self {
		GeneratorError::CustomError(str)
	}
}

impl From<&str> for GeneratorError {
	fn from(str: &str) -> Self {
		GeneratorError::CustomError(str.to_string())
	}
}

impl From<TypeCheckError> for GeneratorError {
	fn from(error: TypeCheckError) -> Self {
		GeneratorError::TypeCheckError(error)
	}
}


pub trait OutputHandler<'a> {
	type FileHandle : io::Write;
	fn create_file<P: AsRef<Path>>(&'a mut self, path: P) -> Result<Self::FileHandle, GeneratorError>;
}


pub trait Language {

	type OptionsBuilder;
	type Options;
	fn empty_options() -> Self::OptionsBuilder;
	fn add_option(builder: &mut Self::OptionsBuilder, name: &str, value: OsString) -> Result<(), GeneratorError>;
	fn finalize_options(builder: Self::OptionsBuilder) -> Result<Self::Options, GeneratorError>;
	
	fn generate<Output : for<'a> OutputHandler<'a>>(model: &model::Verilization, options: Self::Options, output: &mut Output) -> Result<(), GeneratorError>;
}


