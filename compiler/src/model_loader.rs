use crate::VError;
use crate::model;
use crate::type_check::type_check_verilization;

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

/// Loads a set of files into a model.
/// 
/// ```no_run
/// use verilization_compiler::load_files;
/// # fn main() -> Result<(), verilization_compiler::lang::GeneratorError> {
/// let model = load_files(vec!("hello.verilization", "world.verilization"))?;
/// // ...
/// # Ok(())
/// # }
/// ```
#[cfg(not(target_arch = "wasm32"))]
pub fn load_files<P : AsRef<Path>>(files: Vec<P>) -> Result<model::Verilization, VError> {
	use crate::parser;
	
	let models = files
		.into_iter()
		.map(|file| {
			let content = std::fs::read_to_string(file).expect("Could not read input file.");
			let (_, model) = parser::parse_model(&content)?;
			let model = model()?;
			Ok(model)
		});

	load_all_models(models)
}

pub fn load_all_models<M : Iterator<Item = Result<model::Verilization, VError>>>(mut models: M) -> Result<model::Verilization, VError> {

	let mut model = models.next().ok_or(VError::NoInputFiles)??;
	while let Some(other) = models.next() {
		model.merge(other?)?;
	}

	type_check_verilization(&model)?;

	Ok(model)
}

