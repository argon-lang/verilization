use crate::lang::GeneratorError;
use crate::model;
use crate::type_check::type_check_verilization;

#[cfg(not(target_arch = "wasm32"))]
pub fn load_files(files: Vec<std::ffi::OsString>) -> Result<model::Verilization, GeneratorError> {
	use crate::parser;
	
	let models = files
		.into_iter()
		.map(|file| {
			let content = std::fs::read_to_string(file).expect("Could not read input file.");
			parser::parse_model(&content)
				.map(|(_, model)| model)
				.map_err(GeneratorError::from)
		});

	load_all_models(models)
}

pub fn load_all_models<M : Iterator<Item = Result<model::Verilization, GeneratorError>>>(mut models: M) -> Result<model::Verilization, GeneratorError> {

	let mut model = models.next().ok_or("No input files were specified")??;
	models.try_for_each(|other|
		model.merge(other?).map_err(|err| GeneratorError::from(format!("Duplicate definition of {}", err)))
	)?;

	type_check_verilization(&model)?;

	Ok(model)
}

