//! Defines generator related code for languages.

pub mod generator;
pub mod dummy_generator;

use crate::model;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use num_bigint::BigUint;
use std::marker::PhantomData;

/// Error that could occur during generation.
#[derive(Debug)]
pub enum GeneratorError {
	IOError(io::Error),
	UnknownLanguage(String),
	InvalidOptions(String),
	UnmappedPackage(model::PackageName),
	CouldNotFind(model::QualifiedName),
	CouldNotFindVersion(model::QualifiedName, BigUint),
	CouldNotResolveTypeParameter(String),
	TypeCannotBeSequence(model::QualifiedName),
	TypeDoesNotHaveCase(model::QualifiedName, Option<BigUint>, String),
	IncorrectCaseArity(model::QualifiedName, String),
    ArityMismatch(usize, usize),
	RecordLiteralNotForStruct,
	ExternTypeDoesNotHaveRecordLiteral(model::QualifiedName),
	CouldNotFindRecordField(model::QualifiedName, Option<BigUint>, String),
	CouldNotGenerateType,
	InvalidTypeForConstant,
	InvalidTypeForCodec,
	InvalidTypeForIntValue,
	InvalidTypeForString,
	TypeMismatch,
    TypeNotFinal,
	InvalidTypeInExternLiteral,
}

impl From<io::Error> for GeneratorError {
	fn from(err: io::Error) -> Self {
		GeneratorError::IOError(err)
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

	/// Finalized options.
	type Options : LanguageOptions;

	/// Gets the name of the language.
	fn name() -> &'static str;
	
	/// Generates serialization code for the language.
	fn generate<Output: for<'output> OutputHandler<'output>>(model: &model::Verilization, options: Self::Options, output: &mut Output) -> Result<(), GeneratorError>;

}

pub trait LanguageOptions : Sized {
	/// An intermediate step for the language options.
	type Builder : LanguageOptionsBuilder;

	/// Ensures that any required options have been set and creates the options.
	fn build(builder: Self::Builder) -> Result<Self, GeneratorError>;
}

pub trait LanguageOptionsBuilder {
	/// Gets an option builder with no options set.
	fn empty() -> Self;

	/// Sets an option.
	fn add(&mut self, name: &str, value: OsString) -> Result<(), GeneratorError>;
}

pub trait LanguageHandler {
	type Result;
	fn run<Lang: Language>(&mut self) -> Self::Result;
}

pub trait LanguageRegistry : Sized {
	fn has_language(&self, lang_name: &str) -> bool;
	fn handle_language<Handler: LanguageHandler>(&self, lang_name: &str, handler: &mut Handler) -> Option<Handler::Result>;
	fn each_language<Handler: LanguageHandler>(&self, handler: &mut Handler) -> Vec<Handler::Result>;

	fn add_language<Lang: Language>(self) -> LanguageRegistryCons<Lang, Self> {
		LanguageRegistryCons {
			prev: self,
			dummy_lang: PhantomData {},
		}
	}
}

pub struct EmptyLanguageRegistry {}
pub struct LanguageRegistryCons<Lang: Language, Prev: LanguageRegistry> {
	prev: Prev,
	dummy_lang: PhantomData<Lang>,
}

pub fn language_registry_new() -> EmptyLanguageRegistry {
	EmptyLanguageRegistry {}
}

impl LanguageRegistry for EmptyLanguageRegistry {
	fn has_language(&self, _lang_name: &str) -> bool {
		false
	}

	fn handle_language<Handler: LanguageHandler>(&self, _lang_name: &str, _handler: &mut Handler) -> Option<Handler::Result> {
		None
	}

	fn each_language<Handler: LanguageHandler>(&self, _handler: &mut Handler) -> Vec<Handler::Result> {
		Vec::new()
	}
}

impl <Lang: Language, Prev: LanguageRegistry> LanguageRegistry for LanguageRegistryCons<Lang, Prev> {
	fn has_language(&self, lang_name: &str) -> bool  {
		lang_name == Lang::name() || self.prev.has_language(lang_name)
	}

	fn handle_language<Handler: LanguageHandler>(&self, lang_name: &str, handler: &mut Handler) -> Option<Handler::Result> {
		if lang_name == Lang::name() {
			Some(handler.run::<Lang>())
		}
		else {
			self.prev.handle_language(lang_name, handler)
		}
	}

	fn each_language<Handler: LanguageHandler>(&self, handler: &mut Handler) -> Vec<Handler::Result> {
		let mut results = self.prev.each_language(handler);
		results.push(handler.run::<Lang>());
		results
	}
}

