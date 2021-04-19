use crate::model;
use crate::lang::{GeneratorError, Language, OutputHandler};
use std::ffi::OsString;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;
use num_bigint::{BigUint, BigInt, Sign};
use crate::util::for_sep;

type PackageMap = HashMap<model::PackageName, model::PackageName>;
const RUNTIME_PACKAGE: &str = "dev.argon.verilization.scala_runtime";


pub struct ScalaOptionsBuilder {
	output_dir: Option<OsString>,
	package_mapping: PackageMap,
}

pub struct ScalaOptions {
	pub output_dir: OsString,
	pub package_mapping: PackageMap,
}


fn scala_package<'a>(package_mapping: &'a PackageMap, package: &model::PackageName) -> Result<&'a model::PackageName, GeneratorError> {
	Ok(package_mapping.get(&package).ok_or(format!("Unmapped package: {}", package))?)
}


fn write_qual_name<F : Write>(f: &mut F, package_mapping: &PackageMap, name: &model::QualifiedName) -> Result<(), GeneratorError> {
	let pkg = scala_package(&package_mapping, &name.package)?;
	for part in &pkg.package {
		write!(f, "{}.", part)?;
	}

	write!(f, "{}", &name.name)?;

	Ok(())
}




fn open_scala_file<'state, Output: OutputHandler>(options: &ScalaOptions, output: &'state mut Output, name: &model::QualifiedName) -> Result<Output::FileHandle<'state>, GeneratorError> {
	let java_pkg = scala_package(&options.package_mapping, &name.package)?;
	let mut path = PathBuf::from(&options.output_dir);
    for part in &java_pkg.package {
        path.push(part);
    }
	
	path.push(name.name.clone() + ".scala");
	Ok(output.create_file(path)?)
}

fn write_package<F : Write>(f: &mut F, package_mapping: &PackageMap, package: &model::PackageName) -> Result<(), GeneratorError> {
	
	let pkg = scala_package(package_mapping, package)?;

	let mut pkg_iter = pkg.package.iter();

	if let Some(part) = pkg_iter.next() {
		write!(f, "package {}", part)?;
		while let Some(part) = pkg_iter.next() {
			write!(f, ".{}", part)?;
		}
		writeln!(f, "")?;
	}

	Ok(())
}

fn write_type_params<F : Write>(f: &mut F, params: &Vec<String>) -> Result<(), GeneratorError> {
	let mut iter = params.iter();
	if let Some(param) = iter.next() {
		write!(f, "[{}", param)?;
		while let Some(param) = iter.next() {
			write!(f, ", {}", param)?;
		}
		write!(f, "]")?;
	}

	Ok(())
}

fn write_type_args<F : Write>(f: &mut F, package_mapping: &PackageMap, version: &BigUint, scope: &model::Scope, args: &Vec<model::Type>) -> Result<(), GeneratorError> {
	let mut iter = args.iter();
	if let Some(arg) = iter.next() {
		write!(f, "[")?;
		write_type(f, package_mapping, version, scope, arg)?;
		while let Some(arg) = iter.next() {
			write!(f, ", ")?;
			write_type(f, package_mapping, version, scope, arg)?;
		}
		write!(f, "]")?;
	}

	Ok(())
}


fn write_type<F : Write>(f: &mut F, package_mapping: &PackageMap, version: &BigUint, scope: &model::Scope, t: &model::Type) -> Result<(), GeneratorError> {
	Ok(match t {
		// Map built-in types to the equivalent Java type.
		model::Type::Nat | model::Type::Int => write!(f, "scala.math.BigInt")?,
		
        model::Type::U8 | model::Type::I8 => write!(f, "scala.Byte")?,
		
        model::Type::U16 | model::Type::I16 => write!(f, "scala.Short")?,

		model::Type::U32 | model::Type::I32 => write!(f, "scala.Int")?,

		model::Type::U64 | model::Type::I64 => write!(f, "scala.Long")?,

		model::Type::String => write!(f, "scala.String")?,


		model::Type::List(inner) => {
			write!(f, "zio.Chunk[")?;
			write_type(f, package_mapping, version, scope, inner)?;
			write!(f, "]")?;
		},
		model::Type::Option(inner) => {
			write!(f, "scala.Option[")?;
			write_type(f, package_mapping, version, scope, inner)?;
			write!(f, "]")?;
		},

		model::Type::Defined(t, args) => match scope.lookup(t.clone()) {
			model::ScopeLookup::NamedType(t) => {
				write_qual_name(f, package_mapping, &t)?;
				write!(f, ".V{}", version)?;
				write_type_args(f, package_mapping, version, scope, args)?;
			},
			model::ScopeLookup::TypeParameter(name) => {
				write!(f, "{}", name)?;
			}
		},
	})
}

fn write_constant_value<F : Write>(f: &mut F, value: &model::ConstantValue) -> Result<(), GeneratorError> {
	Ok(match value {
		model::ConstantValue::Integer(n) => write!(f, "{}", n)?,
	})
}

fn requires_conversion(field_type: &model::Type) -> bool {
	match field_type {
		model::Type::List(inner) => requires_conversion(inner),
		model::Type::Option(inner) => requires_conversion(inner),
		model::Type::Defined(_, _) => true,
		_ => false,
	}
}

enum ConvertParam {
	FunctionObject,
	Expression(String),
}

fn write_version_convert<'a, F : Write>(f: &mut F, package_mapping: &PackageMap, prev_ver: &BigUint, version: &BigUint, scope: &model::Scope, field_type: &model::Type, param: ConvertParam) -> Result<(), GeneratorError> {
	match field_type {
		model::Type::Defined(name, args) => match scope.lookup(name.clone()) {
			model::ScopeLookup::NamedType(name) => {
				write_qual_name(f, package_mapping, &name)?;
				write!(f, ".V{}", version)?;
				write!(f, ".fromV{}", prev_ver)?;
				if !args.is_empty() {
					write!(f, "(")?;
					for_sep(f, args, |f| write!(f, ", "),
						|f, arg| write_version_convert(f, package_mapping, prev_ver, version, scope, arg, ConvertParam::FunctionObject))?;
					write!(f, ")")?;
				}
				match param {
					ConvertParam::FunctionObject => (),
					ConvertParam::Expression(param_str) => write!(f, "({})", param_str)?,
				}
			},
			model::ScopeLookup::TypeParameter(name) => {
				write!(f, "{}_conv", name)?;
				if let ConvertParam::Expression(param_str) = param {
					write!(f, "({})", param_str)?;
				}
			}
		},

		model::Type::List(inner) if requires_conversion(inner) =>
			match param {
				ConvertParam::FunctionObject => {
					write!(f, "{}.Util.mapChunk(", RUNTIME_PACKAGE)?;
					write_version_convert(f, package_mapping, prev_ver, version, scope, inner, ConvertParam::FunctionObject)?;
					write!(f, ")")?;
				},
				ConvertParam::Expression(param_str) => {
					write!(f, "{}.map(", param_str)?;
					write_version_convert(f, package_mapping, prev_ver, version, scope, inner, ConvertParam::FunctionObject)?;
					write!(f, ")")?;
				},
			},

		model::Type::Option(inner) if requires_conversion(inner) => 
			match param {
				ConvertParam::FunctionObject => {
					write!(f, "{}.Util.mapOption(", RUNTIME_PACKAGE)?;
					write_version_convert(f, package_mapping, prev_ver, version, scope, inner, ConvertParam::FunctionObject)?;
					write!(f, ")")?;
				},
				ConvertParam::Expression(param_str) => {
					write!(f, "{}.map(", param_str)?;
					write_version_convert(f, package_mapping, prev_ver, version, scope, inner, ConvertParam::FunctionObject)?;
					write!(f, ")")?;
				},
			},


		_ => match param {
			ConvertParam::FunctionObject => write!(f, "scala.Predef.identity")?,
			ConvertParam::Expression(param_str) => write!(f, "{}", param_str)?,
		},
	};

	Ok(())
}


pub fn write_codec<F : Write>(f: &mut F, package_mapping: &PackageMap, version: &BigUint, scope: &model::Scope, t: &model::Type) -> Result<(), GeneratorError> {
	match t {
		model::Type::Nat => write!(f, "{}.StandardCodecs.natCodec", RUNTIME_PACKAGE)?,
		model::Type::Int => write!(f, "{}.StandardCodecs.intCodec", RUNTIME_PACKAGE)?,
		model::Type::U8 | model::Type::I8 => write!(f, "{}.StandardCodecs.i8Codec", RUNTIME_PACKAGE)?,
		model::Type::U16 | model::Type::I16 => write!(f, "{}.StandardCodecs.i16Codec", RUNTIME_PACKAGE)?,
		model::Type::U32 | model::Type::I32 => write!(f, "{}.StandardCodecs.i32Codec", RUNTIME_PACKAGE)?,
		model::Type::U64 | model::Type::I64 => write!(f, "{}.StandardCodecs.i64Codec", RUNTIME_PACKAGE)?,
		model::Type::String => write!(f, "{}.StandardCodecs.stringCodec", RUNTIME_PACKAGE)?,
		model::Type::List(inner) => {
			match **inner {
				model::Type::U8 | model::Type::I8 => write!(f, "{}.StandardCodecs.i8ListCodec", RUNTIME_PACKAGE)?,
				model::Type::U16 | model::Type::I16 => write!(f, "{}.StandardCodecs.i16ListCodec", RUNTIME_PACKAGE)?,
				model::Type::U32 | model::Type::I32 => write!(f, "{}.StandardCodecs.i32ListCodec", RUNTIME_PACKAGE)?,
				model::Type::U64 | model::Type::I64 => write!(f, "{}.StandardCodecs.i64ListCodec", RUNTIME_PACKAGE)?,
				_ => {
					write!(f, "{}.StandardCodecs.listCodec(", RUNTIME_PACKAGE)?;
					write_codec(f, package_mapping, version, scope, inner)?;
					write!(f, ")")?
				},
			}
		},
		model::Type::Option(inner) => {
			write!(f, "{}.StandardCodecs.option(", RUNTIME_PACKAGE)?;
			write_codec(f, package_mapping, version, scope, inner)?;
			write!(f, ")")?
		},
		model::Type::Defined(name, args) => match scope.lookup(name.clone()) {
			model::ScopeLookup::NamedType(name) => {
				write_qual_name(f, package_mapping, &name)?;
				write!(f, ".V{}", version)?;
				write!(f, ".codec")?;
				if !args.is_empty() {
					write!(f, "(")?;
					for_sep(f, args, |f| write!(f, ", "),
						|f, arg| write_codec(f, package_mapping, version, scope, arg)
					)?;
					write!(f, ")")?;
				}
			},
			model::ScopeLookup::TypeParameter(name) => {
				write!(f, "{}_codec", name)?
			},
		},
	}

	Ok(())
}

fn write_value_read<F : Write>(f: &mut F, package_mapping: &PackageMap, version: &BigUint, scope: &model::Scope, t: &model::Type) -> Result<(), GeneratorError> {
	write_codec(f, package_mapping, version, scope, t)?;
	write!(f, ".read(reader)")?;

	Ok(())
}

fn write_value_write<F : Write>(f: &mut F, package_mapping: &PackageMap, version: &BigUint, scope: &model::Scope, t: &model::Type, value: String) -> Result<(), GeneratorError> {
	match t {
		model::Type::U8 | model::Type::I8 => write!(f, "writer.writeByte({})", value)?,
		model::Type::U16 | model::Type::I16 => write!(f, "writer.writeShort({})", value)?,
		model::Type::U32 | model::Type::I32 => write!(f, "writer.writeInt({})", value)?,
		model::Type::U64 | model::Type::I64 => write!(f, "writer.writeLong({})", value)?,
		_ => {
			write_codec(f, package_mapping, version, scope, t)?;
			write!(f, ".write(writer, {})", value)?;
		},
	}

	Ok(())
}



struct ScalaConstGenerator<'opt, 'output, Output> {
	options: &'opt ScalaOptions,
	output: &'output mut Output,
}


impl <'opt, 'output, Output: OutputHandler> model::ConstantDefinitionHandler<GeneratorError> for ScalaConstGenerator<'opt, 'output, Output> {
	fn constant(&mut self, latest_version: &BigUint, name: &model::QualifiedName, scope: &model::Scope, constant: &model::Constant, _referenced_types: HashSet<&model::QualifiedName>) -> Result<(), GeneratorError> {
		let mut file = open_scala_file(self.options, self.output, name)?;

        write_package(&mut file, &self.options.package_mapping, &name.package)?;

        writeln!(file, "object {} {{", name.name)?;
        write!(file, "\tval value: ")?;
        write_type(&mut file, &self.options.package_mapping, latest_version, scope, &constant.value_type)?;
        write!(file, " = ")?;
		write_constant_value(&mut file, &constant.value)?;
		writeln!(file, ";")?;
		writeln!(file, "}}")?;

		Ok(())
	}
}


struct ScalaTypeGenerator<'opt, 'output, Output> {
	options: &'opt ScalaOptions,
	output: &'output mut Output,
}

struct ScalaStructType {}
struct ScalaEnumType {}

struct ScalaTypeGeneratorState<'model, 'opt, 'state, 'scope, Output: OutputHandler, Extra> {
	options: &'opt ScalaOptions,
	file: Output::FileHandle<'state>,
	type_name: &'model model::QualifiedName,
	type_params: &'model Vec<String>,
	scope: &'scope model::Scope<'model>,
	versions: HashSet<BigUint>,
	_extra: Extra,
}

trait ScalaExtraGeneratorOps {
	fn create_extra() -> Self;
	fn write_versioned_type<F: Write>(f: &mut F, options: &ScalaOptions, type_name: &model::QualifiedName, version: &BigUint, type_params: &Vec<String>, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_versioned_type_object_data<F: Write>(f: &mut F, options: &ScalaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_from_prev_version<F: Write>(f: &mut F, options: &ScalaOptions, prev_ver: &BigUint, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_codec_read<F: Write>(f: &mut F, options: &ScalaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_codec_write<F: Write>(f: &mut F, options: &ScalaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
}

impl <'model, 'opt, 'output, 'state, 'scope, Output: OutputHandler, Extra: ScalaExtraGeneratorOps> model::TypeDefinitionHandlerState<'model, 'state, 'scope, ScalaTypeGenerator<'opt, 'output, Output>, GeneratorError> for ScalaTypeGeneratorState<'model, 'opt, 'state, 'scope, Output, Extra> where 'model : 'state {
	fn begin(outer: &'state mut ScalaTypeGenerator<'opt, 'output, Output>, type_name: &'model model::QualifiedName, type_params: &'model Vec<String>, scope: &'scope model::Scope<'model>, _referenced_types: HashSet<&'model model::QualifiedName>) -> Result<Self, GeneratorError> {
		let mut file = open_scala_file(outer.options, outer.output, type_name)?;


		write_package(&mut file, &outer.options.package_mapping, &type_name.package)?;
		writeln!(file, "sealed abstract class {}", type_name.name)?;
		writeln!(file, "object {} {{", type_name.name)?;

		Ok(ScalaTypeGeneratorState {
			options: outer.options,
			file: file,
			type_name: type_name,
			type_params: type_params,
			scope: scope,
			versions: HashSet::new(),
			_extra: Extra::create_extra(),
		})
	}

	fn versioned_type(&mut self, explicit_version: bool, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {

		let prev_ver: BigInt = BigInt::from_biguint(Sign::Plus, version.clone()) - 1;
		let prev_ver = prev_ver.magnitude();

		Extra::write_versioned_type(&mut self.file, &self.options, self.type_name, version, self.type_params, self.scope, type_definition)?;

		writeln!(self.file, "\tobject V{} {{", version)?;
		Extra::write_versioned_type_object_data(&mut self.file, &self.options, version, self.scope, type_definition)?;


		if !self.versions.is_empty() {
			writeln!(self.file, "\t\tdef fromV{}(prev: V{}): V{} =", prev_ver, prev_ver, version)?;
			if !explicit_version {
				Extra::write_from_prev_version(&mut self.file, &self.options, prev_ver, version, self.scope, type_definition)?;
			}
			else {
				write!(self.file, "\t\t\t")?;
				write_qual_name(&mut self.file, &self.options.package_mapping, self.type_name)?;
				writeln!(self.file, "_Conversions.v{}ToV{}(prev);", prev_ver, version)?;
			}
		}

		writeln!(self.file, "\t\tval codec: {}.Codec[V{}] = new {}.Codec[V{}] {{", RUNTIME_PACKAGE, version, RUNTIME_PACKAGE, version)?;
		writeln!(self.file, "\t\t\toverride def read[R, E](reader: {}.FormatReader[R, E]): zio.ZIO[R, E, V{}] =", RUNTIME_PACKAGE, version)?;
		Extra::write_codec_read(&mut self.file, &self.options, version, self.scope, type_definition)?;


		writeln!(self.file, "\t\t\toverride def write[R, E](writer: {}.FormatWriter[R, E], value: V{}): zio.ZIO[R, E, Unit] = ", RUNTIME_PACKAGE, version)?;
		Extra::write_codec_write(&mut self.file, &self.options, version, self.scope, type_definition)?;

		writeln!(self.file, "\t\t}}")?;

		writeln!(self.file, "\t}}")?;

		self.versions.insert(version.clone());

		Ok(())
	}
	
	fn end(mut self) -> Result<(), GeneratorError> {
		writeln!(self.file, "}}")?;
		Ok(())
	}
}

impl ScalaExtraGeneratorOps for ScalaStructType {
	fn create_extra() -> Self {
		ScalaStructType {}
	}

	fn write_versioned_type<F: Write>(f: &mut F, options: &ScalaOptions, type_name: &model::QualifiedName, version: &BigUint, type_params: &Vec<String>, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		write!(f, "\tfinal case class V{}", version)?;
		write_type_params(f, type_params)?;
		writeln!(f, "(")?;

		for (field_name, field) in &type_definition.fields {
			write!(f, "\t\t{}: ", field_name)?;
			write_type(f, &options.package_mapping, version, scope, &field.field_type)?;
			writeln!(f, ",")?;
		}

		writeln!(f, "\t) extends {}", type_name.name)?;

		Ok(())
	}

	fn write_versioned_type_object_data<F: Write>(_f: &mut F, _options: &ScalaOptions, _version: &BigUint, _scope: &model::Scope, _type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		Ok(())
	}

	fn write_from_prev_version<F: Write>(f: &mut F, options: &ScalaOptions, prev_ver: &BigUint, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		if type_definition.fields.is_empty() {
			writeln!(f, "\t\t\tV{}()", version)?;
		}
		else {
			writeln!(f, "\t\t\tV{}(", version)?;
			for (field_name, field) in &type_definition.fields {
				write!(f, "\t\t\t\t")?;
				write_version_convert(f, &options.package_mapping, prev_ver, version, scope, &field.field_type, ConvertParam::Expression(format!("prev.{}", field_name)))?;
				writeln!(f, ",")?;
			}
			writeln!(f, "\t\t\t)")?;
		}
		
		Ok(())
	}

	fn write_codec_read<F: Write>(f: &mut F, options: &ScalaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		if type_definition.fields.is_empty() {
			writeln!(f, "\t\t\tzio.IO.succeed(V{}())", version)?;
		}
		else {
			writeln!(f, "\t\t\t\tfor {{")?;
			for (field_name, field) in &type_definition.fields {
				write!(f, "\t\t\t\t\tfield_{} <- ", field_name)?;
				write_value_read(f, &options.package_mapping, version, scope, &field.field_type)?;
				writeln!(f, "")?;
			}
			writeln!(f, "\t\t\t\t}} yield V{}(", version)?;
			for (field_name, _) in &type_definition.fields {
				writeln!(f, "\t\t\t\t\tfield_{},", field_name)?;
			}
			writeln!(f, "\t\t\t\t)")?;
		}

		Ok(())
	}

	fn write_codec_write<F: Write>(f: &mut F, options: &ScalaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		if type_definition.fields.is_empty() {
			writeln!(f, "\t\t\t\tzio.IO.unit")?;
		}
		else {
			writeln!(f, "\t\t\t\tfor {{")?;
			for (field_name, field) in &type_definition.fields {
				write!(f, "\t\t\t\t\t_ <- ")?;
				write_value_write(f, &options.package_mapping, version, scope, &field.field_type, format!("value.{}", field_name))?;
				writeln!(f, "")?;
			}
			writeln!(f, "\t\t\t\t}} yield ()")?;
		}

		Ok(())
	}
}

impl ScalaExtraGeneratorOps for ScalaEnumType {
	fn create_extra() -> Self {
		ScalaEnumType {}
	}

	fn write_versioned_type<F: Write>(f: &mut F, _options: &ScalaOptions, type_name: &model::QualifiedName, version: &BigUint, type_params: &Vec<String>, _scope: &model::Scope, _type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		writeln!(f, "\tsealed abstract class V{}", version)?;
		write_type_params(f, type_params)?;
		writeln!(f, " extends {}", type_name.name)?;
		Ok(())
	}

	fn write_versioned_type_object_data<F: Write>(f: &mut F, options: &ScalaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		for (field_name, field) in &type_definition.fields {
			write!(f, "\t\tfinal case class {}({}: ", field_name, field_name)?;
			write_type(f, &options.package_mapping, version, scope, &field.field_type)?;
			writeln!(f, ") extends V{}", version)?;
		}

		Ok(())
	}

	fn write_from_prev_version<F: Write>(f: &mut F, options: &ScalaOptions, prev_ver: &BigUint, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		if type_definition.fields.is_empty() {
			writeln!(f, "\t\t\tthrow new IllegalArgumentException();")?;
		}
		else {
			writeln!(f, "\t\t\tprev match {{")?;
			for (field_name, field) in &type_definition.fields {
				write!(f, "\t\t\t\tcase prev: V{}.{} => V{}.{}(", prev_ver, field_name, version, field_name)?;
				write_version_convert(f, &options.package_mapping, prev_ver, version, scope, &field.field_type, ConvertParam::Expression(format!("prev.{}", field_name)))?;
				writeln!(f, ")")?;
			}
			writeln!(f, "\t\t\t}}")?;
		}

		Ok(())
	}

	fn write_codec_read<F: Write>(f: &mut F, options: &ScalaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		writeln!(f, "\t\t\t\t{}.StandardCodecs.natCodec.read(reader).flatMap {{", RUNTIME_PACKAGE)?;
		for (index, (field_name, field)) in type_definition.fields.iter().enumerate() {
			writeln!(f, "\t\t\t\t\tcase {}.Util.BigIntValue({}) =>", RUNTIME_PACKAGE, index)?;
			write!(f, "\t\t\t\t\t\t")?;
			write_value_read(f, &options.package_mapping, version, scope, &field.field_type)?;
			writeln!(f, ".map(V{}.{}.apply)", version, field_name)?;
		}
		writeln!(f, "\t\t\t\t\tcase _ => zio.IO.die(new java.lang.RuntimeException(\"Invalid tag number.\"))")?;
		writeln!(f, "\t\t\t\t}}")?;

		Ok(())
	}

	fn write_codec_write<F: Write>(f: &mut F, options: &ScalaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		if type_definition.fields.is_empty() {
			writeln!(f, "\t\t\t\tzio.IO.die(new IllegalArgumentException())")?;
		}
		else {
			writeln!(f, "\t\t\t\tvalue match {{")?;
			for (index, (field_name, field)) in type_definition.fields.iter().enumerate() {
				writeln!(f, "\t\t\t\t\tcase value: V{}.{} =>", version, field_name)?;
				writeln!(f, "\t\t\t\t\t\tfor {{")?;
				writeln!(f, "\t\t\t\t\t\t\t_ <- {}.StandardCodecs.natCodec.write(writer, {})", RUNTIME_PACKAGE, index)?;
				write!(f, "\t\t\t\t\t\t\t_ <- ")?;
				write_value_write(f, &options.package_mapping, version, scope, &field.field_type, format!("value.{}", field_name))?;
				writeln!(f, "")?;
				writeln!(f, "\t\t\t\t\t\t}} yield ()")?;
			}
			writeln!(f, "\t\t\t\t}}")?;
		}

		Ok(())
	}
}


impl <'model, 'opt, 'output, Output: OutputHandler> model::TypeDefinitionHandler<'model, GeneratorError> for ScalaTypeGenerator<'opt, 'output, Output> {
	type StructHandlerState<'state, 'scope> where 'model : 'scope, 'scope : 'state = ScalaTypeGeneratorState<'model, 'opt, 'state, 'scope, Output, ScalaStructType>;
	type EnumHandlerState<'state, 'scope> where 'model : 'scope, 'scope : 'state = ScalaTypeGeneratorState<'model, 'opt, 'state, 'scope, Output, ScalaEnumType>;
}


pub struct ScalaLanguage {}

impl Language for ScalaLanguage {
	type OptionsBuilder = ScalaOptionsBuilder;
	type Options = ScalaOptions;

	fn empty_options() -> ScalaOptionsBuilder {
		ScalaOptionsBuilder {
			output_dir: None,
			package_mapping: HashMap::new(),
		}
	}

	fn add_option(builder: &mut ScalaOptionsBuilder, name: &str, value: OsString) -> Result<(), GeneratorError> {
		if name == "out_dir" {
			if builder.output_dir.is_some() {
				return Err(GeneratorError::from("Output directory already specified"))
			}

			builder.output_dir = Some(value);
			Ok(())
		}
		else if let Some(pkg) = name.strip_prefix("pkg:") {
			let package = model::PackageName::from_str(pkg);

            let scala_package = model::PackageName::from_str(value.to_str().unwrap());

			if builder.package_mapping.insert(package, scala_package).is_some() {
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
		Ok(ScalaOptions {
			output_dir: output_dir,
			package_mapping: builder.package_mapping,
		})
	}

	fn generate<Output: OutputHandler>(model: &model::Verilization, options: Self::Options, output: &mut Output) -> Result<(), GeneratorError> {
		let mut const_gen = ScalaConstGenerator {
			options: &options,
			output: output,
		};

		model.iter_constants(&mut const_gen)?;

		let mut type_gen = ScalaTypeGenerator {
			options: &options,
			output: output,
		};

		model.iter_types(&mut type_gen)?;

		Ok(())
	}

}
