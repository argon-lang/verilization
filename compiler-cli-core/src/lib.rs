use verilization_compiler::*;

use std::env;
use std::ffi::OsString;
use lang::{Language, LanguageOptions, LanguageOptionsBuilder, LanguageRegistry, LanguageHandler};


fn command_version() -> Result<i32, VError> {
	let version = env!("CARGO_PKG_VERSION");
	println!("verilization compiler version {} (native)", version);
	Ok(0)
}

fn command_help() -> Result<i32, VError> {
	let help_message = include_str!("help.txt");
	println!("{}", help_message);
	Ok(0)
}

fn command_generate<Lang: Language>(input_files: Vec<OsString>, options: Lang::Options) -> Result<i32, VError> {
	let model = load_files(input_files)?;

	Lang::generate(&model, options, &mut FileOutputHandler {})?;
	Ok(0)
}

struct ParseGenerateCommand<Args: Iterator<Item = OsString>> {
	args: Args,
}

impl <Args: Iterator<Item = OsString>> LanguageHandler for ParseGenerateCommand<Args> {
	type Result = Result<i32, VError>;

	fn run<Lang: Language>(&mut self) -> Self::Result {
		let mut input_files = Vec::new();
		let mut lang_options = <<Lang::Options as LanguageOptions>::Builder as LanguageOptionsBuilder>::empty();
	
	
		while let Some(arg) = self.args.next() {
			match arg.to_str().unwrap() {
				"-i" => {
					if let Some(filename) = self.args.next() {
						input_files.push(filename)
					}
					else {
						println!("Missing value for input file");
						return Ok(1);
					}
				},
	
				arg => {
					if let Some(option) = arg.strip_prefix("-o:") {
						if let Some(value) = self.args.next() {
							lang_options.add(option, value)?;
						}
						else {
							println!("Missing value for option {}", option);
							return Ok(1);
						}
					}
					else {
						println!("Unknown argument: {}", arg);
						return Ok(1);
					}
				}
			}
		}
	
		let lang_options = Lang::Options::build(lang_options)?;
	
		command_generate::<Lang>(input_files, lang_options)
	}
}

fn parse_args<Args, Registry: LanguageRegistry>(mut args: Args, registry: &Registry) -> Result<i32, VError> where Args : Iterator<Item = OsString> {
	while let Some(arg) = args.next() {
		match arg.to_str().unwrap() {
			"version" | "--version" | "-v" => return command_version(),
			"help" | "--help" | "-h" => return command_help(),
			"generate" => {
				let lang = match args.next() {
					Some(lang) => lang,
					None => {
						println!("Language not specified");
						return Ok(1);
					},
				};

				let lang = lang.to_str().unwrap();

				match registry.handle_language(lang, &mut ParseGenerateCommand { args }) {
					Some(result) => return result,
					None => {
						println!("Unknown language: {}", lang);
						return Ok(1);
					}
				}
			},
				
			arg => {
				println!("Unexpected argument: {}", arg);
				return Ok(1);
			},
		}
	}

	println!("No command specified");
	Ok(1)
}


pub fn main_impl<Registry: LanguageRegistry>(registry: &Registry) {
	let mut args = env::args_os();
	args.next();

	match parse_args(args, registry) {
		Ok(exit_code) => std::process::exit(exit_code),
		Err(err) => {
			println!("{:?}", err);
			std::process::exit(1)
		},
	}
}
