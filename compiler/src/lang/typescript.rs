use crate::model;
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


fn open_ts_file<'state, Output: OutputHandler>(options: &TSOptions, output: &'state mut Output, name: &model::QualifiedName) -> Result<Output::FileHandle<'state>, GeneratorError> {
	let pkg_dir = options.package_mapping.get(&name.package).ok_or(format!("Unmapped package: {}", name.package))?;
	let mut path = PathBuf::from(&options.output_dir);
	path.push(pkg_dir);
	path.push(name.name.clone() + ".ts");
	Ok(output.create_file(path)?)
}

pub fn write_import_name<F : Write>(f: &mut F, name: &model::QualifiedName) -> Result<(), GeneratorError> {
	write!(f, "sym_")?;

	for part in &name.package.package {
		write!(f, "{}_", part)?;
	}

	write!(f, "{}", &name.name)?;

	Ok(())
}

fn write_imports<F : Write>(f: &mut F, options: &TSOptions, name: &model::QualifiedName, referenced_types: HashSet<&model::QualifiedName>) -> Result<(), GeneratorError> {
	let current_pkg_dir = options.package_mapping.get(&name.package).ok_or(format!("Unmapped package: {}", name.package))?;
	let mut current_path = PathBuf::from(&options.output_dir);
	current_path.push(current_pkg_dir);


	let mut referenced_types: Vec<_> = referenced_types.into_iter().collect();
	referenced_types.sort();

	for t in referenced_types {
		if name == t {
			continue;
		}

		let import_pkg_dir = options.package_mapping.get(&t.package).ok_or(format!("Unmapped package: {}", t.package))?;
		let mut abs_import_path = PathBuf::from(&options.output_dir);
		abs_import_path.push(import_pkg_dir);

		let mut import_path: PathBuf = pathdiff::diff_paths(abs_import_path, &current_path).ok_or("Could not find relative path.")?;
		import_path.push(t.name.clone() + ".js");


		write!(f, "import * as ")?;
		write_import_name(f, t)?;
		writeln!(f, " from \"./{}\";", import_path.to_str().unwrap())?;
	}

	Ok(())
}

fn write_type<F : Write>(f: &mut F, version: &BigUint, type_name: Option<&model::QualifiedName>, t: &model::Type) -> Result<(), GeneratorError> {
	Ok(match t {
		// Map built-in types to the equivalent JS type.
		model::Type::Nat |
		model::Type::Int |
		model::Type::U64 |
		model::Type::I64 => write!(f, "bigint")?,

		model::Type::U8 |
		model::Type::I8 |
		model::Type::U16 |
		model::Type::I16 |
		model::Type::U32 |
		model::Type::I32 => write!(f, "number")?,
		
		model::Type::String => write!(f, "string")?,


		model::Type::List(inner) => {
			// Use typed arrays for finite numeric types
			match **inner {
				model::Type::U8 => write!(f, "Uint8Array")?,
				model::Type::I8 => write!(f, "Int8Array")?,
				model::Type::U16 => write!(f, "Uint16Array")?,
				model::Type::I16 => write!(f, "Int16Array")?,
				model::Type::U32 => write!(f, "Uint32Array")?,
				model::Type::I32 => write!(f, "Int32Array")?,
				model::Type::U64 => write!(f, "BigUint64Array")?,
				model::Type::I64 => write!(f, "BigInt64Array")?,
				_ => {
					write!(f, "ReadOnlyArray<")?;
					write_type(f, version, type_name, inner)?;
					write!(f, ">")?;
				}
			}
		},

		// Options map to { value: T } | null because option option T is distinct from option T
		model::Type::Option(inner) => {
			write!(f, "{{ readonly value: ")?;
			write_type(f, version, type_name, inner)?;
			write!(f, "}} | null")?;
		},

		// This is a value of the current type, so we don't need a qualifier.
		model::Type::Defined(t) if type_name.filter(|t2| t == *t2).is_some() => write!(f, "V{}", version)?,

		model::Type::Defined(t) => {
			write_import_name(f, t)?;
			write!(f, ".V{}", version)?;
		},
	})
}

fn write_constant_value<F : Write>(f: &mut F, value: &model::ConstantValue) -> Result<(), GeneratorError> {
	Ok(match value {
		model::ConstantValue::Integer(n) => write!(f, "{}", n)?,
	})
}

// Built-in types do not require
fn requires_conversion(field_type: &model::Type) -> bool {
	match field_type {
		model::Type::List(inner) => requires_conversion(inner),
		model::Type::Option(inner) => requires_conversion(inner),
		model::Type::Defined(_) => true,
		_ => false,
	}
}


fn write_version_convert_list_inner<F : Write>(f: &mut F) -> Result<(), GeneratorError> {
	write!(f, "value")?;
	Ok(())
}

fn write_version_convert_option_inner<F : Write>(f: &mut F) -> Result<(), GeneratorError> {
	write!(f, "value.value")?;
	Ok(())
}

fn write_version_convert<F : Write, E : Into<GeneratorError>>(f: &mut F, prev_ver: &BigUint, version: &BigUint, type_name: &model::QualifiedName, field_type: &model::Type, value_writer: impl FnOnce(&mut F) -> Result<(), E>) -> Result<(), GeneratorError> {
	match field_type {
		model::Type::Defined(name) => {
			if name != type_name {
				write_import_name(f, name)?;
				write!(f, ".")?;
			}

			write!(f, "V{}.from_v{}(", version, prev_ver)?;
			value_writer(f).map_err(E::into)?;
			write!(f, ")")?;
		},

		model::Type::List(inner) if requires_conversion(inner) => {
			value_writer(f).map_err(E::into)?;
			write!(f, ".map(value => ")?;
			write_version_convert(f, prev_ver, version, type_name, inner, write_version_convert_list_inner)?;
			write!(f, ")")?;
		},

		model::Type::Option(inner) if requires_conversion(inner) => {
			write!(f, "(function(value: ")?;
			write_type(f, prev_ver, Some(type_name), field_type)?;
			write!(f, ") {{ if(value !== null) return ")?;
			write_version_convert(f, prev_ver, version, type_name, inner, write_version_convert_option_inner)?;
			write!(f, "; else return null; }})(")?;
			value_writer(f).map_err(E::into)?;
			write!(f, ")")?;
		},


		_ => value_writer(f).map_err(E::into)?,
	};

	Ok(())
}


pub fn write_codec<F : Write>(f: &mut F, version: &BigUint, type_name: Option<&model::QualifiedName>, t: &model::Type) -> Result<(), GeneratorError> {
	match t {
		model::Type::Nat => write!(f, "StandardCodecs.nat")?,
		model::Type::Int => write!(f, "StandardCodecs.int")?,
		model::Type::U8 => write!(f, "StandardCodecs.u8")?,
		model::Type::I8 => write!(f, "StandardCodecs.i8")?,
		model::Type::U16 => write!(f, "StandardCodecs.u16")?,
		model::Type::I16 => write!(f, "StandardCodecs.i16")?,
		model::Type::U32 => write!(f, "StandardCodecs.u32")?,
		model::Type::I32 => write!(f, "StandardCodecs.i32")?,
		model::Type::U64 => write!(f, "StandardCodecs.u64")?,
		model::Type::I64 => write!(f, "StandardCodecs.i64")?,
		model::Type::String => write!(f, "StandardCodecs.string")?,
		model::Type::List(inner) => {
			match **inner {
				model::Type::U8 => write!(f, "StandardCodecs.u8list")?,
				model::Type::I8 => write!(f, "StandardCodecs.i8list")?,
				model::Type::U16 => write!(f, "StandardCodecs.u16list")?,
				model::Type::I16 => write!(f, "StandardCodecs.i16list")?,
				model::Type::U32 => write!(f, "StandardCodecs.u32list")?,
				model::Type::I32 => write!(f, "StandardCodecs.i32list")?,
				model::Type::U64 => write!(f, "StandardCodecs.u64list")?,
				model::Type::I64 => write!(f, "StandardCodecs.i64list")?,
				_ => {
					write!(f, "StandardCodecs.list(")?;
					write_codec(f, version, type_name, inner)?;
					write!(f, ")")?
				},
			}
		},
		model::Type::Option(inner) => {
			write!(f, "StandardCodecs.option(")?;
			write_codec(f, version, type_name, inner)?;
			write!(f, ")")?
		},
		model::Type::Defined(_) => {
			write_type(f, version, type_name, t)?;
			write!(f, ".codec")?;
		},
	}

	Ok(())
}


struct TSConstGenerator<'opt, 'output, Output> {
	options: &'opt TSOptions,
	output: &'output mut Output,
}

impl <'opt, 'output, Output: OutputHandler> model::ConstantDefinitionHandler<GeneratorError> for TSConstGenerator<'opt, 'output, Output> {
	fn constant(&mut self, latest_version: &BigUint, name: &model::QualifiedName, constant: &model::Constant, referenced_types: HashSet<&model::QualifiedName>) -> Result<(), GeneratorError> {
		let mut file = open_ts_file(self.options, self.output, name)?;
		write_imports(&mut file, self.options, name, referenced_types)?;

		write!(file, "const {}: ", name.name)?;
		write_type(&mut file, latest_version, None, &constant.value_type)?;
		write!(file, " = ")?;
		write_constant_value(&mut file, &constant.value)?;
		writeln!(file, ";")?;
		writeln!(file, "export default {};", name.name)?;

		Ok(())
	}
}



struct TSTypeGenerator<'opt, 'output, Output> {
	options: &'opt TSOptions,
	output: &'output mut Output,
}

struct TSStructType {}
struct TSEnumType {}

struct TSTypeGeneratorState<'state, Output: OutputHandler, Extra> {
	file: Output::FileHandle<'state>,
	versions: HashSet<BigUint>,
	_extra: Extra,
}

trait TSExtraGeneratorOps {
	fn create_extra() -> Self;
	fn write_versioned_type<F: Write>(f: &mut F, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_from_prev_version<F: Write>(f: &mut F, prev_ver: &BigUint, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_codec_read<F: Write>(f: &mut F, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_codec_write<F: Write>(f: &mut F, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
}

impl <'model, 'opt, 'output, 'state, Output: OutputHandler, Extra: TSExtraGeneratorOps> model::TypeDefinitionHandlerState<'model, 'state, TSTypeGenerator<'opt, 'output, Output>, GeneratorError> for TSTypeGeneratorState<'state, Output, Extra> where 'model : 'state {
	
	fn begin(outer: &'state mut TSTypeGenerator<'opt, 'output, Output>, type_name: &model::QualifiedName, referenced_types: HashSet<&model::QualifiedName>) -> Result<Self, GeneratorError> {
		let mut file = open_ts_file(outer.options, outer.output, type_name)?;
		writeln!(file, "import {{Codec, FormatWriter, FormatReader, StandardCodecs}} from \"@verilization/runtime\";")?;
		write_imports(&mut file, outer.options, type_name, referenced_types)?;

		Ok(TSTypeGeneratorState {
			file: file,
			versions: HashSet::new(),
			_extra: Extra::create_extra(),
		})
	}

	fn versioned_type(&mut self, explicit_version: bool, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {

		Extra::write_versioned_type(&mut self.file, type_name, version, type_definition)?;

		let prev_ver: BigInt = BigInt::from_biguint(Sign::Plus, version.clone()) - 1;
		let prev_ver = prev_ver.magnitude();

		if explicit_version && !self.versions.is_empty() {
			writeln!(self.file, "import {{v{}_to_v{}}} from \"./{}.conv.js\";", prev_ver, version, type_name.name)?;
		}
		writeln!(self.file, "export namespace V{} {{", version)?;

		if !explicit_version {
			writeln!(self.file, "\texport function from_v{}(prev: V{}): V{} {{", prev_ver, prev_ver, version)?;
			Extra::write_from_prev_version(&mut self.file, prev_ver, type_name, version, type_definition)?;
			writeln!(self.file, "\t}}")?;
		}
		else if !self.versions.is_empty() {
			writeln!(self.file, "\texport const from_v{}: (prev: V{}) => V{} = v{}_to_v{};", prev_ver, prev_ver, version, prev_ver, version)?;
		}

		writeln!(self.file, "\texport const codec: Codec<V{}> = {{", version)?;

		writeln!(self.file, "\t\tasync read(reader: FormatReader): Promise<V{}> {{", version)?;
		Extra::write_codec_read(&mut self.file, type_name, version, type_definition)?;
		writeln!(self.file, "\t\t}},")?;

		writeln!(self.file, "\t\tasync write(writer: FormatWriter, value: V{}): Promise<void> {{", version)?;
		Extra::write_codec_write(&mut self.file, type_name, version, type_definition)?;
		writeln!(self.file, "\t\t}},")?;
		writeln!(self.file, "\t}};")?;

		


		writeln!(self.file, "}}")?;

		self.versions.insert(version.clone());

		Ok(())
	}
	
	fn end(self, _type_name: &model::QualifiedName) -> Result<(), GeneratorError> {
		Ok(())
	}

}

impl TSExtraGeneratorOps for TSStructType {
	fn create_extra() -> Self {
		TSStructType {}
	}

	fn write_versioned_type<F: Write>(f: &mut F, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		writeln!(f, "export interface V{} {{", version)?;
		for (field_name, field) in &type_definition.fields {
			write!(f, "\treadonly {}: ", field_name)?;
			write_type(f, version, Some(type_name), &field.field_type)?;
			writeln!(f, ";")?;
		}
		writeln!(f, "}}")?;
		Ok(())
	}

	fn write_from_prev_version<F: Write>(f: &mut F, prev_ver: &BigUint, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		writeln!(f, "\t\treturn {{")?;
		for (field_name, field) in &type_definition.fields {
			write!(f, "\t\t\t{}: ", field_name)?;
			write_version_convert(f, prev_ver, version, type_name, &field.field_type, |f| write!(f, "prev.{}", field_name))?;
			writeln!(f, ",")?;
		}
		writeln!(f, "\t\t}};")?;
		Ok(())
	}

	fn write_codec_read<F: Write>(f: &mut F, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		writeln!(f, "\t\t\treturn {{")?;
		for (field_name, field) in &type_definition.fields {
			write!(f, "\t\t\t\t{}: await ", field_name)?;
			write_codec(f, version, Some(type_name), &field.field_type)?;
			writeln!(f, ".read(reader),")?;
		}
		writeln!(f, "\t\t\t}};")?;
		Ok(())
	}

	fn write_codec_write<F: Write>(f: &mut F, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		for (field_name, field) in &type_definition.fields {
			write!(f, "\t\t\tawait ")?;
			write_codec(f, version, Some(type_name), &field.field_type)?;
			writeln!(f, ".write(writer, value.{});", field_name)?;
		}
		Ok(())
	}
}

impl TSExtraGeneratorOps for TSEnumType {
	fn create_extra() -> Self {
		TSEnumType {}
	}

	fn write_versioned_type<F: Write>(f: &mut F, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		write!(f, "export type V{} = ", version)?;
		let mut is_first = true;
		for (field_name, field) in &type_definition.fields {
			if !is_first {
				writeln!(f)?;
				write!(f, "\t| ")?;
			}
			else {
				is_first = false;
			}
			write!(f, "{{ readonly tag: \"{}\", readonly {}: ", field_name, field_name)?;
			write_type(f, version, Some(type_name), &field.field_type)?;
			write!(f, ", }}")?;
		}

		writeln!(f, ";")?;
		

		Ok(())
	}

	fn write_from_prev_version<F: Write>(f: &mut F, prev_ver: &BigUint, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		writeln!(f, "\t\tswitch(prev.tag) {{")?;
		for (field_name, field) in &type_definition.fields {
			write!(f, "\t\t\tcase \"{}\": return {{ tag: \"{}\", \"{}\": ", field_name, field_name, field_name)?;
			write_version_convert(f, prev_ver, version, type_name, &field.field_type, |f| write!(f, "prev.{}", field_name))?;
			writeln!(f, "}};")?;
		}
		writeln!(f, "\t\t\tdefault: return prev;")?;
		writeln!(f, "\t\t}}")?;
		Ok(())
	}

	fn write_codec_read<F: Write>(f: &mut F, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		writeln!(f, "\t\t\tconst tag = await StandardCodecs.nat.read(reader);")?;
		writeln!(f, "\t\t\tswitch(tag) {{")?;
		for (index, (field_name, field)) in type_definition.fields.iter().enumerate() {
			write!(f, "\t\t\t\tcase {}n: return {{ tag: \"{}\", \"{}\": await ", index, field_name, field_name)?;
			write_codec(f, version, Some(type_name), &field.field_type)?;
			writeln!(f, ".read(reader) }};")?;
		}
		writeln!(f, "\t\t\t\tdefault: throw new Error(\"Unknown tag\");")?;
		writeln!(f, "\t\t\t}}")?;
		Ok(())
	}

	fn write_codec_write<F: Write>(f: &mut F, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		writeln!(f, "\t\t\tswitch(value.tag) {{")?;
		for (index, (field_name, field)) in type_definition.fields.iter().enumerate() {
			writeln!(f, "\t\t\t\tcase \"{}\":", field_name)?;
			writeln!(f, "\t\t\t\t\tawait StandardCodecs.nat.write(writer, {}n);", index)?;
			write!(f, "\t\t\t\t\tawait ")?;
			write_codec(f, version, Some(type_name), &field.field_type)?;
			writeln!(f, ".write(writer, value.{});", field_name)?;
			writeln!(f, "\t\t\t\t\tbreak;")?;
		}
		writeln!(f, "\t\t\t\tdefault: throw new Error(\"Unknown tag\");")?;
		writeln!(f, "\t\t\t}}")?;
		Ok(())
	}
}


impl <'model, 'opt, 'output, Output: OutputHandler> model::TypeDefinitionHandler<'model, GeneratorError> for TSTypeGenerator<'opt, 'output, Output> {
	type StructHandlerState<'state> where 'model : 'state = TSTypeGeneratorState<'state, Output, TSStructType>;
	type EnumHandlerState<'state> where 'model : 'state = TSTypeGeneratorState<'state, Output, TSEnumType>;
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
		let mut const_gen = TSConstGenerator {
			options: &options,
			output: output,
		};

		model.iter_constants(&mut const_gen)?;

		let mut type_gen = TSTypeGenerator {
			options: &options,
			output: output,
		};

		model.iter_types(&mut type_gen)?;

		Ok(())
	}


	fn write_codec<F: Write>(file: &mut F, _options: &Self::Options, version: &BigUint, type_name: Option<&model::QualifiedName>, t: &model::Type) -> Result<(), GeneratorError> {
		write_codec(file, version, type_name, t)
	}
}
