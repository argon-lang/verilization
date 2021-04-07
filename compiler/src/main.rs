mod model;
mod parser;
mod lang;
mod type_check;
mod file_output_handler;

use std::env;
use std::fs;
use std::ffi::OsString;
use lang::{GeneratorError, Language};


fn load_files(files: Vec<OsString>) -> Result<model::Verilization, GeneratorError> {
	let mut models = files
		.into_iter()
		.map(|file| {
			let content = fs::read_to_string(file).expect("Could not read input file.");
			parser::parse_model(&content)
				.map(|(_, model)| model)
				.map_err(|err| {
					match err {
						nom::Err::Incomplete(_) => GeneratorError::from("Parse error"),
						nom::Err::Error(err) => GeneratorError::from(err),
						nom::Err::Failure(err) => GeneratorError::from(err),
					}
				})
		});

	let mut model = models.next().ok_or("No input files were specified")??;
	models.try_for_each(|other|
		model.merge(other?).map_err(|err| GeneratorError::from(format!("Duplicate definition of {}", err)))
	)?;

	Ok(model)
}



fn command_version() -> Result<(), GeneratorError> {
	println!("TODO: Version");
	Ok(())
}

fn command_help() -> Result<(), GeneratorError> {
	println!("TODO: Help");
	Ok(())
}

fn command_generate<Lang: Language>(lang: &Lang, input_files: Vec<OsString>, options: <Lang as Language>::Options) -> Result<(), GeneratorError> {
	let model = load_files(input_files)?;

	type_check::type_check_verilization(&model)?;

	lang.generate(model, options, &mut file_output_handler::FileOutputHandler {})
}


fn parse_generate_command<Args, Lang : Language>(mut args: Args, lang: &Lang) -> Result<(), GeneratorError> where Args : Iterator<Item = OsString> {
	let mut input_files = Vec::new();
	let mut lang_options = lang.empty_options();


	while let Some(arg) = args.next() {
		match arg.to_str().unwrap() {
			"-i" => {
				if let Some(filename) = args.next() {
					input_files.push(filename)
				}
				else {
					return Err(GeneratorError::from("Missing value for input file"))
				}
			},

			arg => {
				if let Some(option) = arg.strip_prefix("-o:") {
					if let Some(value) = args.next() {
						lang.add_option(&mut lang_options, option, value)?
					}
					else {
						return Err(GeneratorError::from(format!("Missing value for option {}", option)))
					}
				}
				else {
					return Err(GeneratorError::from(format!("Unknown argument: {}", arg)))
				}
			}
		}
	}

	let lang_options = lang.finalize_options(lang_options)?;

	command_generate(lang, input_files, lang_options)
}

fn parse_args<Args>(mut args: Args) -> Result<(), GeneratorError> where Args : Iterator<Item = OsString> {
	while let Some(arg) = args.next() {
		match arg.to_str().unwrap() {
			"version" | "--version" | "-v" => return command_version(),
			"help" | "--help" | "-h" => return command_help(),
			"generate" => {
				let lang = args.next().ok_or("Language not specified: {}")?;
				
				return match lang.to_str().unwrap() {
					"typescript" => parse_generate_command(args, &lang::typescript::TYPESCRIPT_LANGUAGE),
					"java" => parse_generate_command(args, &lang::java::JAVA_LANGUAGE),
					"scala" => parse_generate_command(args, &lang::scala::SCALA_LANGUAGE),
					lang => Err(GeneratorError::from(format!("Unknown language: {}", lang))),
				}
			},
				
			arg => return Err(GeneratorError::from(format!("Unexpected argument: {}", arg)))
		}
	}

	Err(GeneratorError::from("No command specified"))
}


fn main() {
	let mut args = env::args_os();
	args.next();

	match parse_args(args) {
		Ok(_) => (),
		Err(GeneratorError::ParseError(parser::PErrorType::ParseError(input, _))) => println!("Parse error: {}", input),
		Err(GeneratorError::ParseError(parser::PErrorType::DuplicateVersion(_, name, ver))) => println!("Duplicate version {} for type {}", ver, name),
		Err(GeneratorError::ParseError(parser::PErrorType::DuplicateField(_, _, name))) => println!("Duplicate field {}", name),
		Err(GeneratorError::ParseError(parser::PErrorType::DuplicateConstant(name))) => println!("Duplicate definition of constant {}", name),
		Err(GeneratorError::ParseError(parser::PErrorType::DuplicateType(name))) => println!("Duplicate definition of type {}", name),
		Err(GeneratorError::TypeCheckError(type_check::TypeCheckError::TypeNotDefined(name))) => println!("Type not defined: {}", name),
		Err(GeneratorError::TypeCheckError(type_check::TypeCheckError::TypeAddedInNewerVersion(name, version))) => println!("Type was not defined in version {}: {}", version, name),
		Err(GeneratorError::IOError(err)) => println!("{}", err),
		Err(GeneratorError::CustomError(err)) => println!("{}", err),
	}
}
