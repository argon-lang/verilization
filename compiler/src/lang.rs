pub mod typescript;
pub mod java;
pub mod scala;

use crate::model;
use crate::parser::PErrorType;
use crate::type_check::TypeCheckError;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use num_bigint::BigUint;

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

impl From<nom::Err<PErrorType<&str>>> for GeneratorError {
	fn from(err: nom::Err<PErrorType<&str>>) -> Self {
		match err {
			nom::Err::Incomplete(_) => GeneratorError::from("Parse error"),
			nom::Err::Error(err) => GeneratorError::from(err),
			nom::Err::Failure(err) => GeneratorError::from(err),
		}
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


pub trait OutputHandler {
	type FileHandle<'state> : io::Write;
	fn create_file<'state, P: AsRef<Path>>(&'state mut self, path: P) -> Result<Self::FileHandle<'state>, GeneratorError>;
}


pub trait Language {

	type OptionsBuilder;
	type Options;
	fn empty_options() -> Self::OptionsBuilder;
	fn add_option(builder: &mut Self::OptionsBuilder, name: &str, value: OsString) -> Result<(), GeneratorError>;
	fn finalize_options(builder: Self::OptionsBuilder) -> Result<Self::Options, GeneratorError>;
	
	fn generate<Output: OutputHandler>(model: &model::Verilization, options: Self::Options, output: &mut Output) -> Result<(), GeneratorError>;


	fn write_codec<F: io::Write>(file: &mut F, options: &Self::Options, version: &BigUint, type_name: Option<&model::QualifiedName>, t: &model::Type) -> Result<(), GeneratorError>;

}


