//! Defines generator related code for languages.

pub mod generator;
pub mod typescript;
pub mod java;
pub mod scala;

use crate::model;
use crate::parser::PErrorType;
use crate::type_check::TypeCheckError;
use std::ffi::OsString;
use std::io;
use std::path::Path;

/// Error that could occur during generation.
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
			PErrorType::DuplicateFieldValue => PErrorType::DuplicateFieldValue,
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

/// Outputs files produced by the generator.
/// 
/// Allows for capturing the output without writing directly to the file system.
pub trait OutputHandler<'state> {
	type FileHandle : io::Write;
	fn create_file<P: AsRef<Path>>(&'state mut self, path: P) -> Result<Self::FileHandle, GeneratorError>;
}

/// Defines a language supported by Verilization.
pub trait Language {

	/// An intermediate step for the language options.
	type OptionsBuilder;

	/// Finalized options.
	type Options;

	/// Gets an option builder with no options set.
	fn empty_options() -> Self::OptionsBuilder;

	/// Sets an option.
	fn add_option(builder: &mut Self::OptionsBuilder, name: &str, value: OsString) -> Result<(), GeneratorError>;

	/// Ensures that any required options have been set and finalizes the options.
	fn finalize_options(builder: Self::OptionsBuilder) -> Result<Self::Options, GeneratorError>;
	
	/// Generates serialization code for the language.
	fn generate<Output: for<'output> OutputHandler<'output>>(model: &model::Verilization, options: Self::Options, output: &mut Output) -> Result<(), GeneratorError>;

}


