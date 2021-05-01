use crate::{lang, model, TypeCheckError};
use std::io;

#[derive(Debug)]
pub enum VError {
	ParseError(nom::error::Error<String>),
	ParseIncompleteError,
	TypeCheckError(TypeCheckError),
	IOError(io::Error),
	ModelError(model::ModelError),
	GeneratorError(lang::GeneratorError),
    NoInputFiles,
}

impl From<nom::error::Error<&str>> for VError {
	fn from(err: nom::error::Error<&str>) -> Self {
		VError::ParseError(nom::error::Error {
			input: String::from(err.input),
			code: err.code,
		})
	}
}

impl <E> From<nom::Err<E>> for VError where Self : From<E> {
	fn from(err: nom::Err<E>) -> Self {
		match err {
			nom::Err::Incomplete(_) => VError::ParseIncompleteError,
			nom::Err::Error(err) => VError::from(err),
			nom::Err::Failure(err) => VError::from(err),
		}
	}
}

impl From<io::Error> for VError {
	fn from(err: io::Error) -> Self {
		VError::IOError(err)
	}
}

impl From<TypeCheckError> for VError {
	fn from(error: TypeCheckError) -> Self {
		VError::TypeCheckError(error)
	}
}

impl From<model::ModelError> for VError {
	fn from(error: model::ModelError) -> Self {
		VError::ModelError(error)
	}
}

impl From<lang::GeneratorError> for VError {
	fn from(error: lang::GeneratorError) -> Self {
		VError::GeneratorError(error)
	}
}
