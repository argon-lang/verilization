use std::fs;
use std::ffi::OsString;
use crate::lang::GeneratorError;
use crate::model;
use crate::parser;
use crate::type_check::type_check_verilization;


pub fn load_files(files: Vec<OsString>) -> Result<model::Verilization, GeneratorError> {
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

	type_check_verilization(&model)?;

	Ok(model)
}

