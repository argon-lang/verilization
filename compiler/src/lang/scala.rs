use crate::model;
use model::Named;
use crate::lang::{GeneratorError, Language, OutputHandler};
use std::ffi::OsString;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;
use num_bigint::{BigUint, BigInt, Sign};

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


fn scala_package_impl<'a>(package_mapping: &'a PackageMap, package: &model::PackageName) -> Result<&'a model::PackageName, GeneratorError> {
	Ok(package_mapping.get(&package).ok_or(format!("Unmapped package: {}", package))?)
}





fn open_scala_file<'output, Output: OutputHandler>(options: &ScalaOptions, output: &'output mut Output, name: &model::QualifiedName) -> Result<Output::FileHandle<'output>, GeneratorError> {
	let java_pkg = scala_package_impl(&options.package_mapping, &name.package)?;
	let mut path = PathBuf::from(&options.output_dir);
    for part in &java_pkg.package {
        path.push(part);
    }
	
	path.push(name.name.clone() + ".scala");
	Ok(output.create_file(path)?)
}


fn requires_conversion(field_type: &model::Type) -> bool {
	match field_type {
		model::Type::List(inner) => requires_conversion(inner),
		model::Type::Option(inner) => requires_conversion(inner),
		model::Type::Defined(_, _) => true,
		_ => false,
	}
}

pub enum ConvertParam {
	FunctionObject,
	Expression(String),
}

pub trait ScalaGenerator<'model, 'opt> {
	type GeneratorFile : Write;
	fn file(&mut self) -> &mut Self::GeneratorFile;
	fn options(&self) -> &'opt ScalaOptions;
	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model>;
	fn scope(&self) -> &model::Scope<'model>;



	fn scala_package(&self, package: &model::PackageName) -> Result<&'opt model::PackageName, GeneratorError> {
		scala_package_impl(&self.options().package_mapping, package)
	}

	fn write_package(&mut self, package: &model::PackageName) -> Result<(), GeneratorError> {
		
		let pkg = self.scala_package(package)?;

		let mut pkg_iter = pkg.package.iter();

		if let Some(part) = pkg_iter.next() {
			write!(self.file(), "package {}", part)?;
			while let Some(part) = pkg_iter.next() {
				write!(self.file(), ".{}", part)?;
			}
			writeln!(self.file())?;
		}

		Ok(())
	}

	fn write_qual_name(&mut self, name: &model::QualifiedName) -> Result<(), GeneratorError> {
		let pkg = self.scala_package(&name.package)?;
		for part in &pkg.package {
			write!(self.file(), "{}.", part)?;
		}
	
		write!(self.file(), "{}", &name.name)?;
	
		Ok(())
	}

	fn write_type_args(&mut self, version: &BigUint, args: &Vec<model::Type>) -> Result<(), GeneratorError> {
		if !args.is_empty() {
			write!(self.file(), "[")?;
			for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
				self.write_type(version, arg)?;
			});
			write!(self.file(), "]")?;
		}
	
		Ok(())
	}
	
	
	fn write_type(&mut self, version: &BigUint, t: &model::Type) -> Result<(), GeneratorError> {
		Ok(match t {
			// Map built-in types to the equivalent Java type.
			model::Type::Nat | model::Type::Int => write!(self.file(), "scala.math.BigInt")?,
			
			model::Type::U8 | model::Type::I8 => write!(self.file(), "scala.Byte")?,
			
			model::Type::U16 | model::Type::I16 => write!(self.file(), "scala.Short")?,
	
			model::Type::U32 | model::Type::I32 => write!(self.file(), "scala.Int")?,
	
			model::Type::U64 | model::Type::I64 => write!(self.file(), "scala.Long")?,
	
			model::Type::String => write!(self.file(), "scala.String")?,
	
	
			model::Type::List(inner) => {
				write!(self.file(), "zio.Chunk[")?;
				self.write_type(version, inner)?;
				write!(self.file(), "]")?;
			},
			model::Type::Option(inner) => {
				write!(self.file(), "scala.Option[")?;
				self.write_type(version, inner)?;
				write!(self.file(), "]")?;
			},
	
			model::Type::Defined(t, args) => match self.scope().lookup(t.clone()) {
				model::ScopeLookup::NamedType(t) => {
					self.write_qual_name(&t)?;
					write!(self.file(), ".V{}", version)?;
					self.write_type_args(version, args)?;
				},
				model::ScopeLookup::TypeParameter(name) => {
					write!(self.file(), "{}", name)?;
				}
			},
		})
	}

	fn write_version_convert(&mut self, prev_ver: &BigUint, version: &BigUint, field_type: &model::Type, param: ConvertParam) -> Result<(), GeneratorError> {
		match field_type {
			model::Type::Defined(name, args) => match self.scope().lookup(name.clone()) {
				model::ScopeLookup::NamedType(name) => {
					self.write_qual_name(&name)?;
					write!(self.file(), ".V{}", version)?;
					write!(self.file(), ".fromV{}", prev_ver)?;
					if !args.is_empty() {
						write!(self.file(), "(")?;
						for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
							self.write_version_convert(prev_ver, version, arg, ConvertParam::FunctionObject)?;
						});
						write!(self.file(), ")")?;
					}
					match param {
						ConvertParam::FunctionObject => (),
						ConvertParam::Expression(param_str) => write!(self.file(), "({})", param_str)?,
					}
				},
				model::ScopeLookup::TypeParameter(name) => {
					write!(self.file(), "{}_conv", name)?;
					if let ConvertParam::Expression(param_str) = param {
						write!(self.file(), "({})", param_str)?;
					}
				}
			},
	
			model::Type::List(inner) if requires_conversion(inner) =>
				match param {
					ConvertParam::FunctionObject => {
						write!(self.file(), "{}.Util.mapChunk(", RUNTIME_PACKAGE)?;
						self.write_version_convert(prev_ver, version, inner, ConvertParam::FunctionObject)?;
						write!(self.file(), ")")?;
					},
					ConvertParam::Expression(param_str) => {
						write!(self.file(), "{}.map(", param_str)?;
						self.write_version_convert(prev_ver, version, inner, ConvertParam::FunctionObject)?;
						write!(self.file(), ")")?;
					},
				},
	
			model::Type::Option(inner) if requires_conversion(inner) => 
				match param {
					ConvertParam::FunctionObject => {
						write!(self.file(), "{}.Util.mapOption(", RUNTIME_PACKAGE)?;
						self.write_version_convert(prev_ver, version, inner, ConvertParam::FunctionObject)?;
						write!(self.file(), ")")?;
					},
					ConvertParam::Expression(param_str) => {
						write!(self.file(), "{}.map(", param_str)?;
						self.write_version_convert(prev_ver, version, inner, ConvertParam::FunctionObject)?;
						write!(self.file(), ")")?;
					},
				},
	
	
			_ => match param {
				ConvertParam::FunctionObject => write!(self.file(), "scala.Predef.identity")?,
				ConvertParam::Expression(param_str) => write!(self.file(), "{}", param_str)?,
			},
		};
	
		Ok(())
	}


	fn write_codec(&mut self, version: &BigUint, t: &model::Type) -> Result<(), GeneratorError> {
		match t {
			model::Type::Nat => write!(self.file(), "{}.StandardCodecs.natCodec", RUNTIME_PACKAGE)?,
			model::Type::Int => write!(self.file(), "{}.StandardCodecs.intCodec", RUNTIME_PACKAGE)?,
			model::Type::U8 | model::Type::I8 => write!(self.file(), "{}.StandardCodecs.i8Codec", RUNTIME_PACKAGE)?,
			model::Type::U16 | model::Type::I16 => write!(self.file(), "{}.StandardCodecs.i16Codec", RUNTIME_PACKAGE)?,
			model::Type::U32 | model::Type::I32 => write!(self.file(), "{}.StandardCodecs.i32Codec", RUNTIME_PACKAGE)?,
			model::Type::U64 | model::Type::I64 => write!(self.file(), "{}.StandardCodecs.i64Codec", RUNTIME_PACKAGE)?,
			model::Type::String => write!(self.file(), "{}.StandardCodecs.stringCodec", RUNTIME_PACKAGE)?,
			model::Type::List(inner) => {
				match **inner {
					model::Type::U8 | model::Type::I8 => write!(self.file(), "{}.StandardCodecs.i8ListCodec", RUNTIME_PACKAGE)?,
					model::Type::U16 | model::Type::I16 => write!(self.file(), "{}.StandardCodecs.i16ListCodec", RUNTIME_PACKAGE)?,
					model::Type::U32 | model::Type::I32 => write!(self.file(), "{}.StandardCodecs.i32ListCodec", RUNTIME_PACKAGE)?,
					model::Type::U64 | model::Type::I64 => write!(self.file(), "{}.StandardCodecs.i64ListCodec", RUNTIME_PACKAGE)?,
					_ => {
						write!(self.file(), "{}.StandardCodecs.listCodec(", RUNTIME_PACKAGE)?;
						self.write_codec(version, inner)?;
						write!(self.file(), ")")?
					},
				}
			},
			model::Type::Option(inner) => {
				write!(self.file(), "{}.StandardCodecs.option(", RUNTIME_PACKAGE)?;
				self.write_codec(version, inner)?;
				write!(self.file(), ")")?
			},
			model::Type::Defined(name, args) => match self.scope().lookup(name.clone()) {
				model::ScopeLookup::NamedType(name) => {
					self.write_qual_name(&name)?;
					write!(self.file(), ".V{}", version)?;
					write!(self.file(), ".codec")?;
					if !args.is_empty() {
						write!(self.file(), "(")?;
						for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
							self.write_codec(version, arg)?;
						});
						write!(self.file(), ")")?;
					}
				},
				model::ScopeLookup::TypeParameter(name) => {
					write!(self.file(), "{}_codec", name)?
				},
			},
		}
	
		Ok(())
	}
	
}

struct ScalaConstGenerator<'model, 'opt, 'output, Output: OutputHandler> {
	file: Output::FileHandle<'output>,
	options: &'opt ScalaOptions,
	constant: Named<'model, model::Constant>,
	scope: model::Scope<'model>,
}

impl <'model, 'opt, 'output, Output: OutputHandler> ScalaGenerator<'model, 'opt> for ScalaConstGenerator<'model, 'opt, 'output, Output> {
	type GeneratorFile = Output::FileHandle<'output>;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}

	fn options(&self) -> &'opt ScalaOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.constant.referenced_types()
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}

}


impl <'model, 'opt, 'output, Output: OutputHandler> ScalaConstGenerator<'model, 'opt, 'output, Output> {

	fn open(options: &'opt ScalaOptions, output: &'output mut Output, constant: Named<'model, model::Constant>) -> Result<Self, GeneratorError> {
		let file = open_scala_file(options, output, constant.name())?;
		Ok(ScalaConstGenerator {
			file: file,
			options: options,
			constant: constant,
			scope: constant.scope(),
		})
	}


	fn generate(&mut self) -> Result<(), GeneratorError> {
        self.write_package(&self.constant.name().package)?;

		writeln!(self.file, "object {} {{", self.constant.name().name)?;
		for ver in self.constant.versions() {
			write!(self.file, "\tval value: ")?;
			self.write_type(&ver.version, &self.constant.value_type())?;
			write!(self.file, " = ")?;
			if let Some(value) = ver.value {
				self.write_constant_value(&ver.version, value)?;
			}
			else {
				let prev_ver: BigInt = BigInt::from_biguint(Sign::Plus, ver.version.clone()) - 1;
				let prev_ver = prev_ver.to_biguint().unwrap();
				self.write_version_convert(&prev_ver, &ver.version, self.constant.value_type(), ConvertParam::Expression(format!("v{}", prev_ver)))?;
			}
			writeln!(self.file)?;
		}
		writeln!(self.file, "}}")?;

		Ok(())
	}

	fn write_constant_value(&mut self, _version: &BigUint, value: &model::ConstantValue) -> Result<(), GeneratorError> {
		Ok(match value {
			model::ConstantValue::Integer(n) => write!(self.file, "{}", n)?,
		})
	}
}

#[derive(Default)]
struct ScalaStructType {}

#[derive(Default)]
struct ScalaEnumType {}

struct ScalaTypeGenerator<'model, 'opt, 'output, Output: OutputHandler, Extra> {
	options: &'opt ScalaOptions,
	file: Output::FileHandle<'output>,
	type_def: Named<'model, model::TypeDefinitionData>,
	scope: model::Scope<'model>,
	versions: HashSet<BigUint>,
	_extra: Extra,
}

trait ScalaExtraGeneratorOps {
	fn write_versioned_type(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
	fn write_versioned_type_object_data(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
	fn write_from_prev_version(&mut self, ver_type: &model::TypeVersionInfo, prev_ver: &BigUint) -> Result<(), GeneratorError>;
	fn write_codec_read(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
	fn write_codec_write(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra: Default> ScalaGenerator<'model, 'opt> for ScalaTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	type GeneratorFile = Output::FileHandle<'output>;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}

	fn options(&self) -> &'opt ScalaOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.type_def.referenced_types()
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra: Default> ScalaTypeGenerator<'model, 'opt, 'output, Output, Extra> where ScalaTypeGenerator<'model, 'opt, 'output, Output, Extra> : ScalaExtraGeneratorOps {


	fn open(options: &'opt ScalaOptions, output: &'output mut Output, type_def: Named<'model, model::TypeDefinitionData>) -> Result<Self, GeneratorError> {
		let file = open_scala_file(options, output, type_def.name())?;
		Ok(ScalaTypeGenerator {
			file: file,
			options: options,
			type_def: type_def,
			scope: type_def.scope(),
			versions: HashSet::new(),
			_extra: Extra::default(),
		})
	}

	fn generate(&mut self) -> Result<(), GeneratorError> {
		self.write_package(&self.type_def.name().package)?;
		writeln!(self.file, "sealed abstract class {}", self.type_def.name().name)?;
		writeln!(self.file, "object {} {{", self.type_def.name().name)?;

		for ver in self.type_def.versions() {
			self.versioned_type(&ver)?;
		}

		writeln!(self.file, "}}")?;

		Ok(())
	}

	fn versioned_type(&mut self, ver_type: &model::TypeVersionInfo<'model>) -> Result<(), GeneratorError> {

		let version = &ver_type.version;

		let prev_ver: BigInt = BigInt::from_biguint(Sign::Plus, version.clone()) - 1;
		let prev_ver = prev_ver.magnitude();

		self.write_versioned_type(ver_type)?;

		writeln!(self.file, "\tobject V{} {{", version)?;
		self.write_versioned_type_object_data(ver_type)?;


		if !self.versions.is_empty() {
			writeln!(self.file, "\t\tdef fromV{}(prev: V{}): V{} =", prev_ver, prev_ver, version)?;
			if !ver_type.explicit_version {
				self.write_from_prev_version(ver_type, prev_ver)?;
			}
			else {
				write!(self.file, "\t\t\t")?;
				self.write_qual_name(self.type_def.name())?;
				writeln!(self.file, "_Conversions.v{}ToV{}(prev);", prev_ver, version)?;
			}
		}

		writeln!(self.file, "\t\tval codec: {}.Codec[V{}] = new {}.Codec[V{}] {{", RUNTIME_PACKAGE, version, RUNTIME_PACKAGE, version)?;
		writeln!(self.file, "\t\t\toverride def read[R, E](reader: {}.FormatReader[R, E]): zio.ZIO[R, E, V{}] =", RUNTIME_PACKAGE, version)?;
		self.write_codec_read(ver_type)?;


		writeln!(self.file, "\t\t\toverride def write[R, E](writer: {}.FormatWriter[R, E], value: V{}): zio.ZIO[R, E, Unit] = ", RUNTIME_PACKAGE, version)?;
		self.write_codec_write(ver_type)?;

		writeln!(self.file, "\t\t}}")?;

		writeln!(self.file, "\t}}")?;

		self.versions.insert(version.clone());

		Ok(())
	}

	fn write_type_params(&mut self) -> Result<(), GeneratorError> {
		if !self.type_def.type_params().is_empty() {
			write!(self.file, "[")?;
			for_sep!(param, self.type_def.type_params(), { write!(self.file, ", ")?; }, {
				write!(self.file, "{}", param)?;
			});
			write!(self.file, "]")?;
		}
	
		Ok(())
	}
	
	fn write_value_read(&mut self, version: &BigUint, t: &model::Type) -> Result<(), GeneratorError> {
		self.write_codec(version, t)?;
		write!(self.file, ".read(reader)")?;
	
		Ok(())
	}
	
	fn write_value_write(&mut self, version: &BigUint, t: &model::Type, value: String) -> Result<(), GeneratorError> {
		match t {
			model::Type::U8 | model::Type::I8 => write!(self.file, "writer.writeByte({})", value)?,
			model::Type::U16 | model::Type::I16 => write!(self.file, "writer.writeShort({})", value)?,
			model::Type::U32 | model::Type::I32 => write!(self.file, "writer.writeInt({})", value)?,
			model::Type::U64 | model::Type::I64 => write!(self.file, "writer.writeLong({})", value)?,
			_ => {
				self.write_codec(version, t)?;
				write!(self.file, ".write(writer, {})", value)?;
			},
		}
	
		Ok(())
	}
}

impl <'model, 'opt, 'output, 'state, Output: OutputHandler> ScalaExtraGeneratorOps for ScalaTypeGenerator<'model, 'opt, 'state, Output, ScalaStructType> {

	fn write_versioned_type(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		write!(self.file, "\tfinal case class V{}", ver_type.version)?;
		self.write_type_params()?;
		writeln!(self.file, "(")?;

		for (field_name, field) in &ver_type.ver_type.fields {
			write!(self.file, "\t\t{}: ", field_name)?;
			self.write_type(&ver_type.version, &field.field_type)?;
			writeln!(self.file, ",")?;
		}

		writeln!(self.file, "\t) extends {}", self.type_def.name().name)?;

		Ok(())
	}

	fn write_versioned_type_object_data(&mut self, _ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		Ok(())
	}

	fn write_from_prev_version(&mut self, ver_type: &model::TypeVersionInfo, prev_ver: &BigUint) -> Result<(), GeneratorError> {
		if ver_type.ver_type.fields.is_empty() {
			writeln!(self.file, "\t\t\tV{}()", ver_type.version)?;
		}
		else {
			writeln!(self.file, "\t\t\tV{}(", ver_type.version)?;
			for (field_name, field) in &ver_type.ver_type.fields {
				write!(self.file, "\t\t\t\t")?;
				self.write_version_convert(prev_ver, &ver_type.version, &field.field_type, ConvertParam::Expression(format!("prev.{}", field_name)))?;
				writeln!(self.file, ",")?;
			}
			writeln!(self.file, "\t\t\t)")?;
		}
		
		Ok(())
	}

	fn write_codec_read(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		if ver_type.ver_type.fields.is_empty() {
			writeln!(self.file, "\t\t\tzio.IO.succeed(V{}())", ver_type.version)?;
		}
		else {
			writeln!(self.file, "\t\t\t\tfor {{")?;
			for (field_name, field) in &ver_type.ver_type.fields {
				write!(self.file, "\t\t\t\t\tfield_{} <- ", field_name)?;
				self.write_value_read(&ver_type.version, &field.field_type)?;
				writeln!(self.file, "")?;
			}
			writeln!(self.file, "\t\t\t\t}} yield V{}(", ver_type.version)?;
			for (field_name, _) in &ver_type.ver_type.fields {
				writeln!(self.file, "\t\t\t\t\tfield_{},", field_name)?;
			}
			writeln!(self.file, "\t\t\t\t)")?;
		}

		Ok(())
	}

	fn write_codec_write(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		if ver_type.ver_type.fields.is_empty() {
			writeln!(self.file, "\t\t\t\tzio.IO.unit")?;
		}
		else {
			writeln!(self.file, "\t\t\t\tfor {{")?;
			for (field_name, field) in &ver_type.ver_type.fields {
				write!(self.file, "\t\t\t\t\t_ <- ")?;
				self.write_value_write(&ver_type.version, &field.field_type, format!("value.{}", field_name))?;
				writeln!(self.file, "")?;
			}
			writeln!(self.file, "\t\t\t\t}} yield ()")?;
		}

		Ok(())
	}
}

impl <'model, 'opt, 'output, 'state, Output: OutputHandler> ScalaExtraGeneratorOps for ScalaTypeGenerator<'model, 'opt, 'state, Output, ScalaEnumType> {
	fn write_versioned_type(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		writeln!(self.file, "\tsealed abstract class V{}", ver_type.version)?;
		self.write_type_params()?;
		writeln!(self.file, " extends {}", self.type_def.name().name)?;
		Ok(())
	}

	fn write_versioned_type_object_data(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		for (field_name, field) in &ver_type.ver_type.fields {
			write!(self.file, "\t\tfinal case class {}({}: ", field_name, field_name)?;
			self.write_type(&ver_type.version, &field.field_type)?;
			writeln!(self.file, ") extends V{}", ver_type.version)?;
		}

		Ok(())
	}

	fn write_from_prev_version(&mut self, ver_type: &model::TypeVersionInfo, prev_ver: &BigUint) -> Result<(), GeneratorError> {
		if ver_type.ver_type.fields.is_empty() {
			writeln!(self.file, "\t\t\tthrow new IllegalArgumentException();")?;
		}
		else {
			writeln!(self.file, "\t\t\tprev match {{")?;
			for (field_name, field) in &ver_type.ver_type.fields {
				write!(self.file, "\t\t\t\tcase prev: V{}.{} => V{}.{}(", prev_ver, field_name, ver_type.version, field_name)?;
				self.write_version_convert(prev_ver, &ver_type.version, &field.field_type, ConvertParam::Expression(format!("prev.{}", field_name)))?;
				writeln!(self.file, ")")?;
			}
			writeln!(self.file, "\t\t\t}}")?;
		}

		Ok(())
	}

	fn write_codec_read(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		writeln!(self.file, "\t\t\t\t{}.StandardCodecs.natCodec.read(reader).flatMap {{", RUNTIME_PACKAGE)?;
		for (index, (field_name, field)) in ver_type.ver_type.fields.iter().enumerate() {
			writeln!(self.file, "\t\t\t\t\tcase {}.Util.BigIntValue({}) =>", RUNTIME_PACKAGE, index)?;
			write!(self.file, "\t\t\t\t\t\t")?;
			self.write_value_read(&ver_type.version, &field.field_type)?;
			writeln!(self.file, ".map(V{}.{}.apply)", ver_type.version, field_name)?;
		}
		writeln!(self.file, "\t\t\t\t\tcase _ => zio.IO.die(new java.lang.RuntimeException(\"Invalid tag number.\"))")?;
		writeln!(self.file, "\t\t\t\t}}")?;

		Ok(())
	}

	fn write_codec_write(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		if ver_type.ver_type.fields.is_empty() {
			writeln!(self.file, "\t\t\t\tzio.IO.die(new IllegalArgumentException())")?;
		}
		else {
			writeln!(self.file, "\t\t\t\tvalue match {{")?;
			for (index, (field_name, field)) in ver_type.ver_type.fields.iter().enumerate() {
				writeln!(self.file, "\t\t\t\t\tcase value: V{}.{} =>", ver_type.version, field_name)?;
				writeln!(self.file, "\t\t\t\t\t\tfor {{")?;
				writeln!(self.file, "\t\t\t\t\t\t\t_ <- {}.StandardCodecs.natCodec.write(writer, {})", RUNTIME_PACKAGE, index)?;
				write!(self.file, "\t\t\t\t\t\t\t_ <- ")?;
				self.write_value_write(&ver_type.version, &field.field_type, format!("value.{}", field_name))?;
				writeln!(self.file, "")?;
				writeln!(self.file, "\t\t\t\t\t\t}} yield ()")?;
			}
			writeln!(self.file, "\t\t\t\t}}")?;
		}

		Ok(())
	}
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
		for constant in model.constants() {
			let mut const_gen = ScalaConstGenerator::open(&options, output, constant)?;
			const_gen.generate()?;
		}

		for t in model.types() {
			match t {
				model::NamedTypeDefinition::StructType(t) => {
					let mut type_gen: ScalaTypeGenerator<_, ScalaStructType> = ScalaTypeGenerator::open(&options, output, t)?;
					type_gen.generate()?;		
				},
				model::NamedTypeDefinition::EnumType(t) => {
					let mut type_gen: ScalaTypeGenerator<_, ScalaEnumType> = ScalaTypeGenerator::open(&options, output, t)?;
					type_gen.generate()?;		
				},
			}
		}

		Ok(())
	}

}
