use crate::model;
use model::Named;
use crate::lang::{GeneratorError, Language, OutputHandler};
use std::ffi::OsString;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;
use num_bigint::{BigUint, BigInt, Sign};

pub struct TSOptionsBuilder {
	output_dir: Option<OsString>,
	package_mapping: HashMap<model::PackageName, OsString>,
}

pub struct TSOptions {
	pub output_dir: OsString,
	pub package_mapping: HashMap<model::PackageName, OsString>,
}

// Built-in types do not require conversion
fn requires_conversion(field_type: &model::Type) -> bool {
	match field_type {
		model::Type::List(inner) => requires_conversion(inner),
		model::Type::Option(inner) => requires_conversion(inner),
		model::Type::Defined(_, _) => true,
		_ => false,
	}
}


fn open_ts_file<'output, Output: OutputHandler>(options: &TSOptions, output: &'output mut Output, name: &model::QualifiedName) -> Result<Output::FileHandle<'output>, GeneratorError> {
	let pkg_dir = options.package_mapping.get(&name.package).ok_or(format!("Unmapped package: {}", name.package))?;
	let mut path = PathBuf::from(&options.output_dir);
	path.push(pkg_dir);
	path.push(name.name.clone() + ".ts");
	Ok(output.create_file(path)?)
}


pub trait TSGenerator<'model> {
	type GeneratorFile : Write;
	fn file(&mut self) -> &mut Self::GeneratorFile;
	fn model(&mut self) -> &'model model::Verilization;
	fn generator_element_name(&self) -> Option<&'model model::QualifiedName>;
	fn options(&self) -> &TSOptions;
	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model>;
	fn scope(&self) -> &model::Scope<'model>;
	fn current_dir(&self) -> Result<PathBuf, GeneratorError>;

	fn write_import_name(&mut self, name: &model::QualifiedName) -> Result<(), GeneratorError> {
		write!(self.file(), "sym_")?;

		for part in &name.package.package {
			write!(self.file(), "{}_", part)?;
		}

		write!(self.file(), "{}", &name.name)?;

		Ok(())
	}

	fn write_imports(&mut self) -> Result<(), GeneratorError> {
		let current_path = self.current_dir()?;


		let mut referenced_types: Vec<_> = self.referenced_types().collect();
		referenced_types.sort();

		for t in referenced_types {
			if self.generator_element_name() == Some(t) {
				continue;
			}

			let t = match self.scope().lookup(t.clone()) {
				model::ScopeLookup::TypeParameter(_) => continue,
				model::ScopeLookup::NamedType(t) => t,
			};

			let import_pkg_dir = self.options().package_mapping.get(&t.package).ok_or(format!("Unmapped package: {}", t.package))?;
			let mut abs_import_path = PathBuf::from(&self.options().output_dir);
			abs_import_path.push(import_pkg_dir);

			let mut import_path: PathBuf = pathdiff::diff_paths(abs_import_path, &current_path).ok_or("Could not find relative path.")?;
			import_path.push(t.name.clone() + ".js");


			write!(self.file(), "import * as ")?;
			self.write_import_name(&t)?;
			writeln!(self.file(), " from \"./{}\";", import_path.to_str().unwrap())?;
		}

		Ok(())
	}

	fn write_type_args(&mut self, version: &BigUint, args: &Vec<model::Type>) -> Result<(), GeneratorError> {
		if !args.is_empty() {
			write!(self.file(), "<")?;
			for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
				self.write_type(version, arg)?;
			});
			write!(self.file(), ">")?;
		}
	
		Ok(())
	}


	fn write_type(&mut self, version: &BigUint, t: &model::Type) -> Result<(), GeneratorError> {
		Ok(match t {
			// Map built-in types to the equivalent JS type.
			model::Type::Nat |
			model::Type::Int |
			model::Type::U64 |
			model::Type::I64 => write!(self.file(), "bigint")?,
	
			model::Type::U8 |
			model::Type::I8 |
			model::Type::U16 |
			model::Type::I16 |
			model::Type::U32 |
			model::Type::I32 => write!(self.file(), "number")?,
			
			model::Type::String => write!(self.file(), "string")?,
	
	
			model::Type::List(inner) => {
				// Use typed arrays for finite numeric types
				match **inner {
					model::Type::U8 => write!(self.file(), "Uint8Array")?,
					model::Type::I8 => write!(self.file(), "Int8Array")?,
					model::Type::U16 => write!(self.file(), "Uint16Array")?,
					model::Type::I16 => write!(self.file(), "Int16Array")?,
					model::Type::U32 => write!(self.file(), "Uint32Array")?,
					model::Type::I32 => write!(self.file(), "Int32Array")?,
					model::Type::U64 => write!(self.file(), "BigUint64Array")?,
					model::Type::I64 => write!(self.file(), "BigInt64Array")?,
					_ => {
						write!(self.file(), "ReadOnlyArray<")?;
						self.write_type(version, inner)?;
						write!(self.file(), ">")?;
					}
				}
			},
	
			// Options map to { value: T } | null because option(option(T)) is distinct from option(T)
			model::Type::Option(inner) => {
				write!(self.file(), "{{ readonly value: ")?;
				self.write_type(version, inner)?;
				write!(self.file(), "}} | null")?;
			},
	
			model::Type::Defined(t, args) => {
				match self.scope().lookup(t.clone()) {
					model::ScopeLookup::NamedType(t) => {
						match self.model().get_type(&t).ok_or("Could not find type")? {
							model::NamedTypeDefinition::StructType(type_def) | model::NamedTypeDefinition::EnumType(type_def) => {
								let ver_type = type_def.versioned(version).ok_or("Could not find version of type")?;

								// Only use a qualifier if not a value of the current type.
								if self.generator_element_name() != Some(&t) {
									self.write_import_name(&t)?;
									write!(self.file(), ".")?;
								}
					
								write!(self.file(), "V{}", ver_type.version)?;

							},
						}
					},
					model::ScopeLookup::TypeParameter(name) => {
						write!(self.file(), "{}", name)?;
					},
				}
				
				self.write_type_args(version, args)?;
			},
		})
	}

	fn write_version_convert(&mut self, prev_ver: &BigUint, version: &BigUint, t: &model::Type, value_name: &str) -> Result<(), GeneratorError> {
		match t {
			model::Type::Defined(name, args) => match self.scope().lookup(name.clone()) {
				model::ScopeLookup::NamedType(name) => {
					match self.model().get_type(&name).ok_or("Could not find type")? {
						model::NamedTypeDefinition::StructType(type_def) | model::NamedTypeDefinition::EnumType(type_def) => {
							let ver_type = type_def.versioned(version).ok_or("Could not find version of type")?;
							

							let mut conversion_method_name = format!("from_v{}", prev_ver);

							if ver_type.version < *version { // Final type with no newer versions
								// Conversion only required for type parameters
								if args.is_empty() {
									write!(self.file(), "{}", value_name)?;
									return Ok(())
								}
								else {
									conversion_method_name = "convert".to_string();
								}
							}

							if Some(&name) != self.generator_element_name() {
								self.write_import_name(&name)?;
								write!(self.file(), ".")?;
							}
				
							write!(self.file(), "V{}.{}", version, conversion_method_name)?;

							if !args.is_empty() {
								write!(self.file(), "<")?;
								for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
									self.write_type(prev_ver, arg)?;
									write!(self.file(), ", ")?;
									self.write_type(version, arg)?;
								});
								write!(self.file(), ">")?;
							}

							write!(self.file(), "(")?;
							for arg in args {
								write!(self.file(), "value => ")?;
								self.write_version_convert(prev_ver, version, arg, "value")?;
								write!(self.file(), ", ")?;
							}
							write!(self.file(), "{})", value_name)?;

						},
					}		
				},
	
				model::ScopeLookup::TypeParameter(name) => {
					write!(self.file(), "{}_conv({})", name, value_name)?;
				},
			},
	
			model::Type::List(inner) if requires_conversion(inner) => {
				write!(self.file(), "{}.map(value => ", value_name)?;
				self.write_version_convert(prev_ver, version, inner, "value")?;
				write!(self.file(), ")")?;
			},
	
			model::Type::Option(inner) if requires_conversion(inner) => {
				write!(self.file(), "(function(value: ")?;
				self.write_type(prev_ver, t)?;
				write!(self.file(), ") {{ if(value !== null) return ")?;
				self.write_version_convert(prev_ver, version, inner, "value.value")?;
				write!(self.file(), "; else return null; }})({})", value_name)?;
			},
	
	
			_ => write!(self.file(), "{}", value_name)?,
		};
	
		Ok(())
	}

	fn write_codec(&mut self, version: &BigUint, t: &model::Type) -> Result<(), GeneratorError> {
		match t {
			model::Type::Nat => write!(self.file(), "StandardCodecs.nat")?,
			model::Type::Int => write!(self.file(), "StandardCodecs.int")?,
			model::Type::U8 => write!(self.file(), "StandardCodecs.u8")?,
			model::Type::I8 => write!(self.file(), "StandardCodecs.i8")?,
			model::Type::U16 => write!(self.file(), "StandardCodecs.u16")?,
			model::Type::I16 => write!(self.file(), "StandardCodecs.i16")?,
			model::Type::U32 => write!(self.file(), "StandardCodecs.u32")?,
			model::Type::I32 => write!(self.file(), "StandardCodecs.i32")?,
			model::Type::U64 => write!(self.file(), "StandardCodecs.u64")?,
			model::Type::I64 => write!(self.file(), "StandardCodecs.i64")?,
			model::Type::String => write!(self.file(), "StandardCodecs.string")?,
			model::Type::List(inner) => {
				match **inner {
					model::Type::U8 => write!(self.file(), "StandardCodecs.u8list")?,
					model::Type::I8 => write!(self.file(), "StandardCodecs.i8list")?,
					model::Type::U16 => write!(self.file(), "StandardCodecs.u16list")?,
					model::Type::I16 => write!(self.file(), "StandardCodecs.i16list")?,
					model::Type::U32 => write!(self.file(), "StandardCodecs.u32list")?,
					model::Type::I32 => write!(self.file(), "StandardCodecs.i32list")?,
					model::Type::U64 => write!(self.file(), "StandardCodecs.u64list")?,
					model::Type::I64 => write!(self.file(), "StandardCodecs.i64list")?,
					_ => {
						write!(self.file(), "StandardCodecs.list(")?;
						self.write_codec(version, inner)?;
						write!(self.file(), ")")?;
					},
				}
			},
			model::Type::Option(inner) => {
				write!(self.file(), "StandardCodecs.option(")?;
				self.write_codec(version, inner)?;
				write!(self.file(), ")")?;
			},
			model::Type::Defined(name, args) => match self.scope().lookup(name.clone()) {
				model::ScopeLookup::NamedType(name) => {
					self.write_import_name(&name)?;

					let type_ver = match self.model().get_type(&name).ok_or("Could not find type")? {
						model::NamedTypeDefinition::StructType(type_def) | model::NamedTypeDefinition::EnumType(type_def) => {
							let ver_type = type_def.versioned(version).ok_or("Could not find version of type")?;
							ver_type.version
						},
					};

					write!(self.file(), ".V{}.codec", type_ver)?;
					self.write_type_args(version, args)?;
					if !args.is_empty() {
						write!(self.file(), "(")?;
						for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
							self.write_codec(version, arg)?;
						});
						write!(self.file(), ")")?;
					}
				},
				model::ScopeLookup::TypeParameter(name) => {
					write!(self.file(), "{}_codec", name)?;
				}
			},
		}
	
		Ok(())
	}
}

fn current_dir_of_name<'model, Gen: TSGenerator<'model>>(gen: &Gen, name: &model::QualifiedName) -> Result<PathBuf, GeneratorError> {
	let current_pkg_dir = gen.options().package_mapping.get(&name.package).ok_or(format!("Unmapped package: {}", name.package))?;
	let mut current_path = PathBuf::from(&gen.options().output_dir);
	current_path.push(current_pkg_dir);
	Ok(current_path)
}



struct TSConstGenerator<'model, 'opt, 'output, Output: OutputHandler> {
	file: Output::FileHandle<'output>,
	model: &'model model::Verilization,
	options: &'opt TSOptions,
	constant: Named<'model, model::Constant>,
	scope: model::Scope<'model>,
}

impl <'model, 'opt, 'output, Output: OutputHandler> TSGenerator<'model> for TSConstGenerator<'model, 'opt, 'output, Output> {
	type GeneratorFile = Output::FileHandle<'output>;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}

	fn model(&mut self) -> &'model model::Verilization {
		self.model
	}

	fn generator_element_name(&self) -> Option<&'model model::QualifiedName> {
		Some(self.constant.name())
	}

	fn options(&self) -> &TSOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.constant.referenced_types()
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}

	fn current_dir(&self) -> Result<PathBuf, GeneratorError> {
		current_dir_of_name(self, self.constant.name())
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> TSConstGenerator<'model, 'opt, 'output, Output> {

	fn open(model: &'model model::Verilization, options: &'opt TSOptions, output: &'output mut Output, constant: Named<'model, model::Constant>) -> Result<Self, GeneratorError> {
		let file = open_ts_file(options, output, constant.name())?;
		Ok(TSConstGenerator {
			file: file,
			model: model,
			options: options,
			constant: constant,
			scope: constant.scope(),
		})
	}

	fn generate(&mut self) -> Result<(), GeneratorError> {
		self.write_imports()?;

		for ver in self.constant.versions() {
			write!(self.file, "export const v{}: ", ver.version)?;
			self.write_type(&ver.version, self.constant.value_type())?;
			write!(self.file, " = ")?;
			if let Some(value) = ver.value {
				self.write_constant_value(&ver.version, value)?;
			}
			else {
				let prev_ver: BigInt = BigInt::from_biguint(Sign::Plus, ver.version.clone()) - 1;
				let prev_ver = prev_ver.to_biguint().unwrap();
				self.write_version_convert(&prev_ver, &ver.version, self.constant.value_type(), &format!("v{}", prev_ver))?;
			}
			writeln!(self.file, ";")?;
		}

		Ok(())
	}

	fn write_constant_value(&mut self, _version: &BigUint, value: &model::ConstantValue) -> Result<(), GeneratorError> {
		Ok(match value {
			model::ConstantValue::Integer(n) => write!(self.file, "{}", n)?,
		})
	}
}

#[derive(Default)]
struct TSStructType {}

#[derive(Default)]
struct TSEnumType {}

struct TSTypeGenerator<'model, 'opt, 'output, Output: OutputHandler, Extra> {
	file: Output::FileHandle<'output>,
	model: &'model model::Verilization,
	options: &'opt TSOptions,
	type_def: Named<'model, model::TypeDefinitionData>,
	scope: model::Scope<'model>,
	versions: HashSet<BigUint>,
	_extra: Extra,
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> TSGenerator<'model> for TSTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	type GeneratorFile = Output::FileHandle<'output>;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}

	fn model(&mut self) -> &'model model::Verilization {
		self.model
	}

	fn generator_element_name(&self) -> Option<&'model model::QualifiedName> {
		Some(self.type_def.name())
	}

	fn options(&self) -> &TSOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.type_def.referenced_types()
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}

	fn current_dir(&self) -> Result<PathBuf, GeneratorError> {
		current_dir_of_name(self, self.type_def.name())
	}
}

trait TSExtraGeneratorOps {
	fn write_versioned_type(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
	fn write_from_prev_version(&mut self, ver_type: &model::TypeVersionInfo, prev_ver: &BigUint) -> Result<(), GeneratorError>;
	fn write_codec_read(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
	fn write_codec_write(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
}



impl <'model, 'opt, 'output, Output: OutputHandler, Extra: Default> TSTypeGenerator<'model, 'opt, 'output, Output, Extra> where TSTypeGenerator<'model, 'opt, 'output, Output, Extra> : TSExtraGeneratorOps {

	fn open(model: &'model model::Verilization, options: &'opt TSOptions, output: &'output mut Output, type_def: Named<'model, model::TypeDefinitionData>) -> Result<Self, GeneratorError> {
		let file = open_ts_file(options, output, type_def.name())?;
		Ok(TSTypeGenerator {
			file: file,
			model: model,
			options: options,
			type_def: type_def,
			scope: type_def.scope(),
			versions: HashSet::new(),
			_extra: Extra::default(),
		})
	}
	
	
	fn generate(&mut self) -> Result<(), GeneratorError> {
		writeln!(self.file, "import {{Codec, FormatWriter, FormatReader, StandardCodecs}} from \"@verilization/runtime\";")?;
		self.write_imports()?;

		for ver_type in self.type_def.versions() {
			self.versioned_type(&ver_type)?;
		}

		Ok(())
	}
	

	fn versioned_type(&mut self, ver_type: &model::TypeVersionInfo<'model>) -> Result<(), GeneratorError> {

		self.write_versioned_type(ver_type)?;

		let version = &ver_type.version;

		let prev_ver: BigInt = BigInt::from_biguint(Sign::Plus, version.clone()) - 1;
		let prev_ver = prev_ver.to_biguint().unwrap();

		if ver_type.explicit_version && !self.versions.is_empty() {
			writeln!(self.file, "import {{v{}_to_v{}}} from \"./{}.conv.js\";", prev_ver, version, self.type_def.name().name)?;
		}
		writeln!(self.file, "export namespace V{} {{", version)?;


		// Skip conversion function for first version.
		if !self.versions.is_empty() {
			write!(self.file, "\texport function from_v{}", prev_ver)?;


			self.write_type_params_with(|param| format!("{}_1, {}_2", param, param))?;

			write!(self.file, "(")?;
			for param in self.type_def.type_params() {
				write!(self.file, "{}_conv: (prev: {}_1) => {}_2, ", param, param, param)?;
			}
			write!(self.file, "prev: V{}", prev_ver)?;
			self.write_type_params_with(|param| format!("{}_1", param))?;
			write!(self.file, "): V{}", version)?;
			self.write_type_params_with(|param| format!("{}_2", param))?;
			writeln!(self.file, " {{")?;
			if ver_type.explicit_version {
				write!(self.file, "\t\treturn v{}_to_v{}(", prev_ver, version)?;
				
				for param in self.type_def.type_params() {
					write!(self.file, "{}_conv, ", param)?;
				}				

				writeln!(self.file, "prev);")?;
			}
			else {
				self.write_from_prev_version(ver_type, &prev_ver)?;
			}
			writeln!(self.file, "\t}}")?;
		}

		if self.type_def.type_params().is_empty() {
			writeln!(self.file, "\texport const codec: Codec<V{}> = {{", version)?;
		}
		else {
			write!(self.file, "\texport function codec")?;
			self.write_type_params()?;
			write!(self.file, "(")?;
			for_sep!(param, self.type_def.type_params(), { write!(self.file, ", ")?; }, {
				write!(self.file, "{}_codec: Codec<{}>", param, param)?;
			});
			write!(self.file, "): Codec<V{}", version)?;
			self.write_type_params()?;
			writeln!(self.file, "> {{ return {{")?;
		}

		write!(self.file, "\t\tasync read(reader: FormatReader): Promise<V{}", version)?;
		self.write_type_params()?;
		writeln!(self.file, "> {{")?;
		self.write_codec_read(ver_type)?;
		writeln!(self.file, "\t\t}},")?;

		write!(self.file, "\t\tasync write(writer: FormatWriter, value: V{}", version)?;
		self.write_type_params()?;
		writeln!(self.file, "): Promise<void> {{")?;
		self.write_codec_write(ver_type)?;
		writeln!(self.file, "\t\t}},")?;
		writeln!(self.file, "\t}};")?;

		
		if self.type_def.type_params().is_empty() {
			writeln!(self.file, "}}")?;
		}
		else {
			writeln!(self.file, "}}; }}")?;
		}

		self.versions.insert(version.clone());

		Ok(())
	}

	fn write_type_params(&mut self) -> Result<(), GeneratorError> {
		self.write_type_params_with(|s| s.to_string())
	}

	fn write_type_params_with(&mut self, f: impl Fn(&str) -> String) -> Result<(), GeneratorError> {
		if !self.type_def.type_params().is_empty() {
			write!(self.file, "<")?;
			for_sep!(param, self.type_def.type_params(), { write!(self.file, ", ")?; }, {
				write!(self.file, "{}", f(param))?;
			});
			write!(self.file, ">")?;
		}
	
		Ok(())
	}


}

impl <'model, 'opt, 'output, Output: OutputHandler> TSExtraGeneratorOps for TSTypeGenerator<'model, 'opt, 'output, Output, TSStructType> {

	fn write_versioned_type(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		writeln!(self.file, "export interface V{}", ver_type.version)?;
		self.write_type_params()?;
		writeln!(self.file, " {{")?;
		for (field_name, field) in &ver_type.ver_type.fields {
			write!(self.file, "\treadonly {}: ", field_name)?;
			self.write_type(&ver_type.version, &field.field_type)?;
			writeln!(self.file, ";")?;
		}
		writeln!(self.file, "}}")?;
		Ok(())
	}

	fn write_from_prev_version(&mut self, ver_type: &model::TypeVersionInfo, prev_ver: &BigUint) -> Result<(), GeneratorError> {
		writeln!(self.file, "\t\treturn {{")?;
		for (field_name, field) in &ver_type.ver_type.fields {
			write!(self.file, "\t\t\t{}: ", field_name)?;
			self.write_version_convert(prev_ver, &ver_type.version, &field.field_type, &format!("prev.{}", field_name))?;
			writeln!(self.file, ",")?;
		}
		writeln!(self.file, "\t\t}};")?;
		Ok(())
	}

	fn write_codec_read(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		writeln!(self.file, "\t\t\treturn {{")?;
		for (field_name, field) in &ver_type.ver_type.fields {
			write!(self.file, "\t\t\t\t{}: await ", field_name)?;
			self.write_codec(&ver_type.version, &field.field_type)?;
			writeln!(self.file, ".read(reader),")?;
		}
		writeln!(self.file, "\t\t\t}};")?;
		Ok(())
	}

	fn write_codec_write(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		for (field_name, field) in &ver_type.ver_type.fields {
			write!(self.file, "\t\t\tawait ")?;
			self.write_codec(&ver_type.version, &field.field_type)?;
			writeln!(self.file, ".write(writer, value.{});", field_name)?;
		}
		Ok(())
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> TSExtraGeneratorOps for TSTypeGenerator<'model, 'opt, 'output, Output, TSEnumType> {

	fn write_versioned_type(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		write!(self.file, "export type V{}", ver_type.version)?;
		self.write_type_params()?;
		write!(self.file, " = ")?;
		let mut is_first = true;
		for (field_name, field) in &ver_type.ver_type.fields {
			if !is_first {
				writeln!(self.file)?;
				write!(self.file, "\t| ")?;
			}
			else {
				is_first = false;
			}
			write!(self.file, "{{ readonly tag: \"{}\", readonly {}: ", field_name, field_name)?;
			self.write_type(&ver_type.version, &field.field_type)?;
			write!(self.file, ", }}")?;
		}

		writeln!(self.file, ";")?;
		

		Ok(())
	}

	fn write_from_prev_version(&mut self, ver_type: &model::TypeVersionInfo, prev_ver: &BigUint) -> Result<(), GeneratorError> {
		writeln!(self.file, "\t\tswitch(prev.tag) {{")?;
		for (field_name, field) in &ver_type.ver_type.fields {
			write!(self.file, "\t\t\tcase \"{}\": return {{ tag: \"{}\", \"{}\": ", field_name, field_name, field_name)?;
			self.write_version_convert(prev_ver, &ver_type.version, &field.field_type, &format!("prev.{}", field_name))?;
			writeln!(self.file, "}};")?;
		}
		writeln!(self.file, "\t\t\tdefault: return prev;")?;
		writeln!(self.file, "\t\t}}")?;
		Ok(())
	}

	fn write_codec_read(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		writeln!(self.file, "\t\t\tconst tag = await StandardCodecs.nat.read(reader);")?;
		writeln!(self.file, "\t\t\tswitch(tag) {{")?;
		for (index, (field_name, field)) in ver_type.ver_type.fields.iter().enumerate() {
			write!(self.file, "\t\t\t\tcase {}n: return {{ tag: \"{}\", \"{}\": await ", index, field_name, field_name)?;
			self.write_codec(&ver_type.version, &field.field_type)?;
			writeln!(self.file, ".read(reader) }};")?;
		}
		writeln!(self.file, "\t\t\t\tdefault: throw new Error(\"Unknown tag\");")?;
		writeln!(self.file, "\t\t\t}}")?;
		Ok(())
	}

	fn write_codec_write(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		writeln!(self.file, "\t\t\tswitch(value.tag) {{")?;
		for (index, (field_name, field)) in ver_type.ver_type.fields.iter().enumerate() {
			writeln!(self.file, "\t\t\t\tcase \"{}\":", field_name)?;
			writeln!(self.file, "\t\t\t\t\tawait StandardCodecs.nat.write(writer, {}n);", index)?;
			write!(self.file, "\t\t\t\t\tawait ")?;
			self.write_codec(&ver_type.version, &field.field_type)?;
			writeln!(self.file, ".write(writer, value.{});", field_name)?;
			writeln!(self.file, "\t\t\t\t\tbreak;")?;
		}
		writeln!(self.file, "\t\t\t\tdefault: throw new Error(\"Unknown tag\");")?;
		writeln!(self.file, "\t\t\t}}")?;
		Ok(())
	}
}


pub struct TypeScriptLanguage {}

impl Language for TypeScriptLanguage {
	type OptionsBuilder = TSOptionsBuilder;
	type Options = TSOptions;

	fn empty_options() -> TSOptionsBuilder {
		TSOptionsBuilder {
			output_dir: None,
			package_mapping: HashMap::new(),
		}
	}

	fn add_option(builder: &mut TSOptionsBuilder, name: &str, value: OsString) -> Result<(), GeneratorError> {
		if name == "out_dir" {
			if builder.output_dir.is_some() {
				return Err(GeneratorError::from("Output directory already specified"))
			}

			builder.output_dir = Some(value);
			Ok(())
		}
		else if let Some(pkg) = name.strip_prefix("pkg:") {
			let package = model::PackageName::from_str(pkg);

			if builder.package_mapping.insert(package, value).is_some() {
				return Err(GeneratorError::from(format!("Package already mapped: {}", pkg)))
			}
			Ok(())
		}
		else {
			Err(GeneratorError::from(format!("Unknown option: {}", name)))
		}
	}

	fn finalize_options(builder: Self::OptionsBuilder) -> Result<Self::Options, GeneratorError> {
		let output_dir = builder.output_dir.ok_or("Output directory not specified")?;
		Ok(TSOptions {
			output_dir: output_dir,
			package_mapping: builder.package_mapping,
		})
	}

	fn generate<Output: OutputHandler>(model: &model::Verilization, options: Self::Options, output: &mut Output) -> Result<(), GeneratorError> {
		for constant in model.constants() {
			let mut const_gen = TSConstGenerator::open(model, &options, output, constant)?;
			const_gen.generate()?;
		}

		for t in model.types() {
			match t {
				model::NamedTypeDefinition::StructType(t) => {
					let mut type_gen: TSTypeGenerator<_, TSStructType> = TSTypeGenerator::open(model, &options, output, t)?;
					type_gen.generate()?;		
				},
				model::NamedTypeDefinition::EnumType(t) => {
					let mut type_gen: TSTypeGenerator<_, TSEnumType> = TSTypeGenerator::open(model, &options, output, t)?;
					type_gen.generate()?;		
				},
			}
		}

		Ok(())
	}

}
