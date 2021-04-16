use verilization_compiler::*;

use std::env;
use std::ffi::OsString;
use lang::{GeneratorError, Language};


fn command_version() -> Result<(), GeneratorError> {
	println!("TODO: Version");
	Ok(())
}

fn command_help() -> Result<(), GeneratorError> {
	println!("TODO: Help");
	Ok(())
}

fn command_generate<Lang: Language>(input_files: Vec<OsString>, options: Lang::Options) -> Result<(), GeneratorError> {
	let model = load_files(input_files)?;

	Lang::generate(&model, options, &mut file_output_handler::FileOutputHandler {})
}


fn parse_generate_command<Args, Lang : Language>(mut args: Args) -> Result<(), GeneratorError> where Args : Iterator<Item = OsString> {
	let mut input_files = Vec::new();
	let mut lang_options = Lang::empty_options();


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
						Lang::add_option(&mut lang_options, option, value)?
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

	let lang_options = Lang::finalize_options(lang_options)?;

	command_generate::<Lang>(input_files, lang_options)
}

fn parse_args<Args>(mut args: Args) -> Result<(), GeneratorError> where Args : Iterator<Item = OsString> {
	while let Some(arg) = args.next() {
		match arg.to_str().unwrap() {
			"version" | "--version" | "-v" => return command_version(),
			"help" | "--help" | "-h" => return command_help(),
			"generate" => {
				let lang = args.next().ok_or("Language not specified")?;
				
				return match lang.to_str().unwrap() {
					"typescript" => parse_generate_command::<_, lang::typescript::TypeScriptLanguage>(args),
					"java" => parse_generate_command::<_, lang::java::JavaLanguage>(args),
					"scala" => parse_generate_command::<_, lang::scala::ScalaLanguage>(args),
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
		Ok(_) => std::process::exit(0),
		Err(err) => {
			match err {
				GeneratorError::ParseError(ParserError::ParseError(input, _)) => println!("Parse error: {}", input),
				GeneratorError::ParseError(ParserError::DuplicateVersion(_, name, ver)) => println!("Duplicate version {} for type {}", ver, name),
				GeneratorError::ParseError(ParserError::DuplicateField(_, _, name)) => println!("Duplicate field {}", name),
				GeneratorError::ParseError(ParserError::DuplicateConstant(name)) => println!("Duplicate definition of constant {}", name),
				GeneratorError::ParseError(ParserError::DuplicateType(name)) => println!("Duplicate definition of type {}", name),
				GeneratorError::TypeCheckError(TypeCheckError::TypeNotDefined(name)) => println!("Type not defined: {}", name),
				GeneratorError::TypeCheckError(TypeCheckError::TypeAddedInNewerVersion(name, version)) => println!("Type was not defined in version {}: {}", version, name),
				GeneratorError::IOError(err) => println!("{}", err),
				GeneratorError::CustomError(err) => println!("{}", err),
			};
			std::process::exit(1)
		},
	}
}
