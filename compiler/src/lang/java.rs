use crate::model;
use model::Named;
use crate::lang::{GeneratorError, Language, OutputHandler};
use std::ffi::OsString;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;
use num_bigint::{BigUint, BigInt, Sign};

type PackageMap = HashMap<model::PackageName, model::PackageName>;
const RUNTIME_PACKAGE: &str = "dev.argon.verilization.java_runtime";


pub struct JavaOptionsBuilder {
	output_dir: Option<OsString>,
	package_mapping: PackageMap,
}

pub struct JavaOptions {
	pub output_dir: OsString,
	pub package_mapping: PackageMap,
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


fn java_package_impl<'opt>(options: &'opt JavaOptions, package: &model::PackageName) -> Result<&'opt model::PackageName, GeneratorError> {
	Ok(options.package_mapping.get(&package).ok_or(format!("Unmapped package: {}", package))?)
}

fn open_java_file<'output, Output: OutputHandler>(options: &JavaOptions, output: &'output mut Output, name: &model::QualifiedName) -> Result<Output::FileHandle<'output>, GeneratorError> {
	let java_pkg = java_package_impl(options, &name.package)?;
	let mut path = PathBuf::from(&options.output_dir);
	for part in &java_pkg.package {
		path.push(part);
	}
	
	path.push(name.name.clone() + ".java");
	Ok(output.create_file(path)?)
}

pub trait JavaGenerator<'model, 'opt> {
	type GeneratorFile : Write;
	fn file(&mut self) -> &mut Self::GeneratorFile;
	fn options(&self) -> &'opt JavaOptions;
	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model>;
	fn scope(&self) -> &model::Scope<'model>;

	fn java_package(&self, package: &model::PackageName) -> Result<&'opt model::PackageName, GeneratorError> {
		java_package_impl(self.options(), package)
	}

	fn write_package(&mut self, package: &model::PackageName) -> Result<(), GeneratorError> {
	
		let pkg = self.java_package(package)?;
	
		let mut pkg_iter = pkg.package.iter();
	
		if let Some(part) = pkg_iter.next() {
			write!(self.file(), "package {}", part)?;
			while let Some(part) = pkg_iter.next() {
				write!(self.file(), ".{}", part)?;
			}
			writeln!(self.file(), ";")?;
		}
	
		Ok(())
	}

	fn write_qual_name(&mut self, name: &model::QualifiedName) -> Result<(), GeneratorError> {
		let pkg = self.java_package(&name.package)?;
		for part in &pkg.package {
			write!(self.file(), "{}.", part)?;
		}
	
		write!(self.file(), "{}", &name.name)?;
	
		Ok(())
	}
	
	fn write_type_args(&mut self, version: &BigUint, args: &Vec<model::Type>) -> Result<(), GeneratorError> {
		if !args.is_empty() {
			write!(self.file(), "<")?;
			for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
				self.write_type(version, arg, true)?;
			});
			write!(self.file(), ">")?;

		}
	
		Ok(())
	}
	
	
	fn write_type(&mut self, version: &BigUint, t: &model::Type, erased: bool) -> Result<(), GeneratorError> {
		Ok(match t {
			// Map built-in types to the equivalent Java type.
			model::Type::Nat | model::Type::Int => write!(self.file(), "java.math.BigInteger")?,
			
	
			model::Type::U8 | model::Type::I8 if erased => write!(self.file(), "java.lang.Byte")?,
			model::Type::U8 | model::Type::I8 => write!(self.file(), "byte")?,
			
			model::Type::U16 | model::Type::I16 if erased => write!(self.file(), "java.lang.Short")?,
			model::Type::U16 | model::Type::I16 => write!(self.file(), "short")?,
	
			model::Type::U32 | model::Type::I32 if erased => write!(self.file(), "java.lang.Integer")?,
			model::Type::U32 | model::Type::I32 => write!(self.file(), "int")?,
	
			model::Type::U64 | model::Type::I64 if erased => write!(self.file(), "java.lang.Long")?,
			model::Type::U64 | model::Type::I64 => write!(self.file(), "long")?,
	
			model::Type::String => write!(self.file(), "java.lang.String")?,
	
	
			model::Type::List(inner) => {
				match **inner {
					model::Type::U8 | model::Type::I8 |
					model::Type::U16 | model::Type::I16 |
					model::Type::U32 | model::Type::I32 |
					model::Type::U64 | model::Type::I64 => {
						self.write_type(version, inner, false)?;
						write!(self.file(), "[]")?;
					},
					_ => {
						write!(self.file(), "java.util.List<")?;
						self.write_type(version, inner, true)?;
						write!(self.file(), ">")?;
					}
				}
			},
	
			model::Type::Option(inner) => {
				write!(self.file(), "java.util.Optional<")?;
				self.write_type(version, inner, true)?;
				write!(self.file(), ">")?;
			},
	
			model::Type::Defined(t, args) => match self.scope().lookup(t.clone()) {
				model::ScopeLookup::NamedType(t) => {
					self.write_qual_name(&t)?;
					write!(self.file(), ".V{}", version)?;
					self.write_type_args(version, args)?;
				},
				model::ScopeLookup::TypeParameter(name) => {
					write!(self.file(), "{}", name)?;
				},
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
						ConvertParam::Expression(param_str) => write!(self.file(), ".apply({})", param_str)?,
					}
				},
				model::ScopeLookup::TypeParameter(name) => {
					write!(self.file(), "{}_conv", name)?;
					if let ConvertParam::Expression(param_str) = param {
						write!(self.file(), ".apply({})", param_str)?;
					}
				},
			},
	
			model::Type::List(inner) if requires_conversion(inner) =>
				match param {
					ConvertParam::FunctionObject => {
						write!(self.file(), "{}.Util.mapList(", RUNTIME_PACKAGE)?;
						self.write_version_convert(prev_ver, version, inner, ConvertParam::FunctionObject)?;
						write!(self.file(), ")")?;
					},
					ConvertParam::Expression(param_str) => {
						write!(self.file(), "{}.stream().map(", param_str)?;
						self.write_version_convert(prev_ver, version, inner, ConvertParam::FunctionObject)?;
						write!(self.file(), ").collect(Collectors.toList())")?;
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
				ConvertParam::FunctionObject => write!(self.file(), "java.util.function.Function::identity")?,
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
			model::Type::String => write!(self.file(), "{}.StandardCodecs.string", RUNTIME_PACKAGE)?,
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
						for_sep!(arg, args, { write!(self.file(), ", ")? },
							{ self.write_codec(version, arg)?; }
						);
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


struct JavaConstGenerator<'model, 'opt, 'output, Output: OutputHandler> {
	file: Output::FileHandle<'output>,
	options: &'opt JavaOptions,
	constant: Named<'model, model::Constant>,
	scope: model::Scope<'model>,
}

impl <'model, 'opt, 'output, Output: OutputHandler> JavaGenerator<'model, 'opt> for JavaConstGenerator<'model, 'opt, 'output, Output> {
	type GeneratorFile = Output::FileHandle<'output>;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}

	fn options(&self) -> &'opt JavaOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.constant.referenced_types()
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}

}

impl <'model, 'opt, 'output, Output: OutputHandler> JavaConstGenerator<'model, 'opt, 'output, Output> {

	fn open(options: &'opt JavaOptions, output: &'output mut Output, constant: Named<'model, model::Constant>) -> Result<Self, GeneratorError> {
		let file = open_java_file(options, output, constant.name())?;
		Ok(JavaConstGenerator {
			file: file,
			options: options,
			constant: constant,
			scope: constant.scope(),
		})
	}

	fn generate(&mut self) -> Result<(), GeneratorError> {
        self.write_package(&self.constant.name().package)?;

		writeln!(self.file, "public final class {} {{", self.constant.name().name)?;
		for ver in self.constant.versions() {
			write!(self.file, "\tpublic static final ")?;
			self.write_type(&ver.version, &self.constant.value_type(), false)?;
			write!(self.file, " VALUE = ")?;
			if let Some(value) = ver.value {
				self.write_constant_value(&ver.version, value)?;
			}
			else {
				let prev_ver: BigInt = BigInt::from_biguint(Sign::Plus, ver.version.clone()) - 1;
				let prev_ver = prev_ver.to_biguint().unwrap();
				self.write_version_convert(&prev_ver, &ver.version, self.constant.value_type(), ConvertParam::Expression(format!("v{}", prev_ver)))?;
			}
			writeln!(self.file, ";")?;
		}
		writeln!(self.file, "}}")?;

		Ok(())
	}

	fn write_constant_value(&mut self, _version: &BigUint, value: &model::ConstantValue) -> Result<(), GeneratorError> {
		Ok(match value {
			model::ConstantValue::Integer(n) => write!(self.file(), "{}", n)?,
		})
	}
}

#[derive(Default)]
struct JavaStructType {}

#[derive(Default)]
struct JavaEnumType {}

struct JavaTypeGenerator<'model, 'opt, 'output, Output: OutputHandler, Extra> {
	file: Output::FileHandle<'output>,
	options: &'opt JavaOptions,
	type_def: Named<'model, model::TypeDefinitionData>,
	scope: model::Scope<'model>,
	versions: HashSet<BigUint>,
	_extra: Extra,
}

trait JavaExtraGeneratorOps {
	fn version_class_modifier() -> &'static str;
	fn write_versioned_type_data(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
	fn write_from_prev_version(&mut self, ver_type: &model::TypeVersionInfo, prev_ver: &BigUint) -> Result<(), GeneratorError>;
	fn write_codec_read(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
	fn write_codec_write(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> JavaGenerator<'model, 'opt> for JavaTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	type GeneratorFile = Output::FileHandle<'output>;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}

	fn options(&self) -> &'opt JavaOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.type_def.referenced_types()
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}

}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra: Default> JavaTypeGenerator<'model, 'opt, 'output, Output, Extra> where JavaTypeGenerator<'model, 'opt, 'output, Output, Extra> : JavaExtraGeneratorOps {


	fn open(options: &'opt JavaOptions, output: &'output mut Output, type_def: Named<'model, model::TypeDefinitionData>) -> Result<Self, GeneratorError> {
		let file = open_java_file(options, output, type_def.name())?;
		Ok(JavaTypeGenerator {
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
		writeln!(self.file, "public abstract class {} {{", self.type_def.name().name)?;
		writeln!(self.file, "\tprivate {}() {{}}", self.type_def.name().name)?;

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

		write!(self.file, "\tpublic static {} class V{}", Self::version_class_modifier(), version)?;
		self.write_type_params()?;
		writeln!(self.file, " extends {} {{", self.type_def.name().name)?;
		
		self.write_versioned_type_data(ver_type)?;

		if !self.versions.is_empty() {
			write!(self.file, "\t\tpublic static ")?;
			if self.type_def.type_params().is_empty() {
				write!(self.file, "final ")?;
			}
			else {
				write!(self.file, "<")?;
				for_sep!(param, self.type_def.type_params(), { write!(self.file, ", ")?; }, {
					write!(self.file, "{}_1, {}_2", param, param)?;
				});
				write!(self.file, "> ")?;
			}
			write!(self.file, "java.util.function.Function<V{}", prev_ver)?;
			if !self.type_def.type_params().is_empty() {
				write!(self.file, "<")?;
				for_sep!(param, self.type_def.type_params(), { write!(self.file, ", ")?; }, {
					write!(self.file, "{}_1", param)?;
				});
				write!(self.file, ">")?;
			}
			write!(self.file, ", V{}", version)?;
			if !self.type_def.type_params().is_empty() {
				write!(self.file, "<")?;
				for_sep!(param, self.type_def.type_params(), { write!(self.file, ", ")?; }, {
					write!(self.file, "{}_2", param)?;
				});
				write!(self.file, ">")?;
			}
			write!(self.file, "> fromV{}", prev_ver)?;
			if self.type_def.type_params().is_empty() {
				writeln!(self.file, " =")?;
				write!(self.file, "\t\t\t")?;
			}
			else {
				write!(self.file, "(")?;
				for_sep!(param, self.type_def.type_params(), { write!(self.file, ", ")?; }, {
					write!(self.file, "{}_conv", param)?;
				});
				writeln!(self.file, ") {{")?;
				write!(self.file, "\t\t\treturn ")?;
			}

			writeln!(self.file, "prev -> {{")?;

			if !ver_type.explicit_version {
				self.write_from_prev_version(ver_type, prev_ver)?;
			}
			else {
				write!(self.file, "\t\t\t\treturn ")?;
				self.write_qual_name(self.type_def.name())?;
				writeln!(self.file, "_Conversions.v{}ToV{}(prev);", prev_ver, version)?;
			}

			writeln!(self.file, "\t\t\t}};")?;
			if !self.type_def.type_params().is_empty() {
				writeln!(self.file, "\t\t}}")?;
			}

		}

		write!(self.file, "\t\tprivate static final class CodecImpl")?;
		self.write_type_params()?;
		write!(self.file, " implements {}.Codec<V{}", RUNTIME_PACKAGE, version)?;
		self.write_type_params()?;
		writeln!(self.file, "> {{")?;
		writeln!(self.file, "\t\t\t@Override")?;
		write!(self.file, "\t\t\tpublic V{}", version)?;
		self.write_type_params()?;
		writeln!(self.file, " read({}.FormatReader reader) throws java.io.IOException {{", RUNTIME_PACKAGE)?;
		self.write_codec_read(ver_type)?;
		writeln!(self.file, "\t\t\t}}")?;
		writeln!(self.file, "\t\t\t@Override")?;
		write!(self.file, "\t\t\tpublic void write({}.FormatWriter writer, V{}", RUNTIME_PACKAGE, version)?;
		self.write_type_params()?;
		writeln!(self.file, " value) throws java.io.IOException {{")?;
		self.write_codec_write(ver_type)?;
		writeln!(self.file, "\t\t\t}}")?;
		writeln!(self.file, "\t\t}}")?;

		if self.type_def.type_params().is_empty() {
			writeln!(self.file, "\t\tpublic static final {}.Codec<V{}> codec = new CodecImpl();", RUNTIME_PACKAGE, version)?;
		}
		else {
			write!(self.file, "\t\tpublic static ")?;
			self.write_type_params()?;
			write!(self.file, "{}.Codec<V{}", RUNTIME_PACKAGE, version)?;
			self.write_type_params()?;
			writeln!(self.file, "> codec = new CodecImpl();")?;
		}

		writeln!(self.file, "\t}}")?;

		self.versions.insert(version.clone());

		Ok(())
	}

	fn write_type_params(&mut self) -> Result<(), GeneratorError> {
		if !self.type_def.type_params().is_empty() {
			write!(self.file, "<")?;
			for_sep!(param, self.type_def.type_params(), { write!(self.file, ", ")?; }, {
				write!(self.file, "{}", param)?;
			});
			write!(self.file, ">")?;
		}
	
		Ok(())
	}
	
	fn write_value_read(&mut self, version: &BigUint, t: &model::Type) -> Result<(), GeneratorError> {
		match t {
			model::Type::U8 | model::Type::I8 => write!(self.file, "reader.readByte()")?,
			model::Type::U16 | model::Type::I16 => write!(self.file, "reader.readShort()")?,
			model::Type::U32 | model::Type::I32 => write!(self.file, "reader.readInt()")?,
			model::Type::U64 | model::Type::I64 => write!(self.file, "reader.readLong()")?,
			_ => {
				self.write_codec(version, t)?;
				write!(self.file, ".read(reader)")?;
			},
		}
	
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

impl <'model, 'opt, 'output, 'state, Output: OutputHandler> JavaExtraGeneratorOps for JavaTypeGenerator<'model, 'opt, 'state, Output, JavaStructType> {
	fn version_class_modifier() -> &'static str {
		"final"
	}

	fn write_versioned_type_data(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		write!(self.file, "\t\tpublic V{}(", ver_type.version)?;

		for_sep!((field_name, field), &ver_type.ver_type.fields, { write!(self.file, ",")?; }, {
			writeln!(self.file, "")?;
			write!(self.file, "\t\t\t")?;
			self.write_type(&ver_type.version, &field.field_type, false)?;
			write!(self.file, " {}", field_name)?;
		});

		if !ver_type.ver_type.fields.is_empty() {
			writeln!(self.file, "")?;
			write!(self.file, "\t\t")?;
		}

		writeln!(self.file, ") {{")?;
		for (field_name, _) in &ver_type.ver_type.fields {
			writeln!(self.file, "\t\t\tthis.{} = {};", field_name, field_name)?;
		}

		writeln!(self.file, "\t\t}}")?;

		for (field_name, field) in &ver_type.ver_type.fields {
			write!(self.file, "\t\tpublic final ")?;
			self.write_type(&ver_type.version, &field.field_type, false)?;
			writeln!(self.file, " {};", field_name)?;
		}

		writeln!(self.file, "\t\t@Override")?;
		writeln!(self.file, "\t\tpublic int hashCode() {{")?;
		write!(self.file, "\t\t\treturn java.util.Objects.hash(")?;
		for_sep!((field_name, _), &ver_type.ver_type.fields, { write!(self.file, ", ")?; }, {
			write!(self.file, "{}", field_name)?;
		});
		writeln!(self.file, ");")?;
		writeln!(self.file, "\t\t}}")?;

		writeln!(self.file, "\t\t@Override")?;
		writeln!(self.file, "\t\tpublic boolean equals(Object obj) {{")?;
		writeln!(self.file, "\t\t\tif(!(obj instanceof V{})) return false;", ver_type.version)?;
		writeln!(self.file, "\t\t\tV{} other = (V{})obj;", ver_type.version, ver_type.version)?;
		for (field_name, _) in &ver_type.ver_type.fields {
			writeln!(self.file, "\t\t\tif(!java.util.Objects.deepEquals(this.{}, other.{})) return false;", field_name, field_name)?;
		}
		writeln!(self.file, "\t\t\treturn true;")?;
		writeln!(self.file, "\t\t}}")?;

		Ok(())
	}

	fn write_from_prev_version(&mut self, ver_type: &model::TypeVersionInfo, prev_ver: &BigUint) -> Result<(), GeneratorError> {
		write!(self.file, "\t\t\t\treturn new V{}(", ver_type.version)?;
		for_sep!((field_name, field), &ver_type.ver_type.fields, { write!(self.file, ",")?; }, {
			writeln!(self.file, "")?;
			write!(self.file, "\t\t\t\t\t")?;
			self.write_version_convert(prev_ver, &ver_type.version, &field.field_type, ConvertParam::Expression(format!("prev.{}", field_name)))?;
		});
		if !ver_type.ver_type.fields.is_empty() {
			writeln!(self.file, "")?;
			write!(self.file, "\t\t\t\t")?;
		}
		writeln!(self.file, ");")?;

		Ok(())
	}

	fn write_codec_read(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		write!(self.file, "\t\t\t\treturn new V{}(", ver_type.version)?;
		for_sep!((_, field), &ver_type.ver_type.fields, { write!(self.file, ",")?; }, {
			writeln!(self.file, "")?;
			write!(self.file, "\t\t\t\t\t")?;
			self.write_value_read(&ver_type.version, &field.field_type)?;
		});
		if !ver_type.ver_type.fields.is_empty() {
			writeln!(self.file, "")?;
			write!(self.file, "\t\t\t\t")?;
		}
		writeln!(self.file, ");")?;

		Ok(())
	}

	fn write_codec_write(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		for (field_name, field) in &ver_type.ver_type.fields {
			write!(self.file, "\t\t\t\t")?;
			self.write_value_write(&ver_type.version, &field.field_type, format!("value.{}", field_name))?;
			writeln!(self.file, ";")?;
		}

		Ok(())
	}
}

impl <'model, 'opt, 'output, 'state, Output: OutputHandler> JavaExtraGeneratorOps for JavaTypeGenerator<'model, 'opt, 'state, Output, JavaEnumType> {
	fn version_class_modifier() -> &'static str {
		"abstract"
	}

	fn write_versioned_type_data(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		writeln!(self.file, "\t\tprivate V{}() {{}}", ver_type.version)?;

		for (index, (field_name, field)) in ver_type.ver_type.fields.iter().enumerate() {
			writeln!(self.file, "\t\tpublic static final class {} extends V{} {{", field_name, ver_type.version)?;
			write!(self.file, "\t\t\tpublic {}(", field_name)?;
			self.write_type(&ver_type.version, &field.field_type, false)?;
			writeln!(self.file, " {}) {{", field_name)?;
			writeln!(self.file, "\t\t\t\tthis.{} = {};", field_name, field_name)?;
			writeln!(self.file, "\t\t\t}}")?;
			write!(self.file, "\t\t\tpublic final ")?;
			self.write_type(&ver_type.version, &field.field_type, false)?;
			writeln!(self.file, " {};", field_name)?;
			
			writeln!(self.file, "\t\t\t@Override")?;
			writeln!(self.file, "\t\t\tpublic int hashCode() {{")?;
			writeln!(self.file, "\t\t\t\treturn java.util.Objects.hash({}, this.{});", index, field_name)?;
			writeln!(self.file, "\t\t\t}}")?;
			
			writeln!(self.file, "\t\t\t@Override")?;
			writeln!(self.file, "\t\t\tpublic boolean equals(Object obj) {{")?;
			writeln!(self.file, "\t\t\t\tif(!(obj instanceof {})) return false;", field_name)?;
			writeln!(self.file, "\t\t\t\t{} other = ({})obj;", field_name, field_name)?;
			writeln!(self.file, "\t\t\t\treturn java.util.Objects.deepEquals(this.{}, other.{});", field_name, field_name)?;
			writeln!(self.file, "\t\t\t}}")?;
	

			writeln!(self.file, "\t\t}}")?;
		}

		Ok(())
	}

	fn write_from_prev_version(&mut self, ver_type: &model::TypeVersionInfo, prev_ver: &BigUint) -> Result<(), GeneratorError> {
		write!(self.file, "\t\t\t\t")?;
		for (field_name, field) in &ver_type.ver_type.fields {
			writeln!(self.file, "if(prev instanceof V{}.{}) {{", prev_ver, field_name)?;
			write!(self.file, "\t\t\t\t\treturn new V{}.{}(", ver_type.version, field_name)?;
			self.write_version_convert(prev_ver, &ver_type.version, &field.field_type, ConvertParam::Expression(format!("((V{}.{})prev).{}", prev_ver, field_name, field_name)))?;
			writeln!(self.file, ");")?;
			writeln!(self.file, "\t\t\t\t}}")?;
			write!(self.file, "\t\t\t\telse ")?;
		}
		if !ver_type.ver_type.fields.is_empty() {
			writeln!(self.file, "{{")?;
			write!(self.file, "\t\t")?;
		}
		writeln!(self.file, "\t\t\t\tthrow new IllegalArgumentException();")?;
		if !ver_type.ver_type.fields.is_empty() {
			writeln!(self.file, "\t\t\t\t}}")?;
		}

		Ok(())
	}

	fn write_codec_read(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		writeln!(self.file, "\t\t\t\tjava.math.BigInteger tag = {}.StandardCodecs.natCodec.read(reader);", RUNTIME_PACKAGE)?;
		writeln!(self.file, "\t\t\t\tif(tag.compareTo(java.math.BigInteger.valueOf(java.lang.Integer.MAX_VALUE)) > 0) throw new java.lang.ArithmeticException();")?;
		writeln!(self.file, "\t\t\t\tswitch(tag.intValue()) {{")?;
		for (index, (field_name, field)) in ver_type.ver_type.fields.iter().enumerate() {
			writeln!(self.file, "\t\t\t\t\tcase {}:", index)?;
			write!(self.file, "\t\t\t\t\t\treturn new V{}.{}(", ver_type.version, field_name)?;
			self.write_value_read(&ver_type.version, &field.field_type)?;
			writeln!(self.file, ");")?;
		}
		writeln!(self.file, "\t\t\t\t\tdefault:")?;
		writeln!(self.file, "\t\t\t\t\t\tthrow new java.io.IOException(\"Invalid tag number.\");")?;
		writeln!(self.file, "\t\t\t\t}}")?;

		Ok(())
	}

	fn write_codec_write(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		write!(self.file, "\t\t\t\t")?;
		for (index, (field_name, field)) in ver_type.ver_type.fields.iter().enumerate() {
			writeln!(self.file, "if(value instanceof V{}.{}) {{", ver_type.version, field_name)?;
			write!(self.file, "\t\t\t\t\t{}.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf({}))", RUNTIME_PACKAGE, index)?;
			writeln!(self.file, ";")?;
			write!(self.file, "\t\t\t\t\t")?;
			self.write_value_write(&ver_type.version, &field.field_type, format!("((V{}.{})value).{}", ver_type.version, field_name, field_name))?;
			writeln!(self.file, ";")?;
			writeln!(self.file, "\t\t\t\t}}")?;
			write!(self.file, "\t\t\t\telse ")?;
		}
		if !ver_type.ver_type.fields.is_empty() {
			writeln!(self.file, "{{")?;
			write!(self.file, "\t")?;
		}
		writeln!(self.file, "\t\t\t\tthrow new IllegalArgumentException();")?;
		if !ver_type.ver_type.fields.is_empty() {
			writeln!(self.file, "\t\t\t\t}}")?;
		}

		Ok(())
	}
}


pub struct JavaLanguage {}

impl Language for JavaLanguage {
	type OptionsBuilder = JavaOptionsBuilder;
	type Options = JavaOptions;

	fn empty_options() -> JavaOptionsBuilder {
		JavaOptionsBuilder {
			output_dir: None,
			package_mapping: HashMap::new(),
		}
	}

	fn add_option(builder: &mut JavaOptionsBuilder, name: &str, value: OsString) -> Result<(), GeneratorError> {
		if name == "out_dir" {
			if builder.output_dir.is_some() {
				return Err(GeneratorError::from("Output directory already specified"))
			}

			builder.output_dir = Some(value);
			Ok(())
		}
		else if let Some(pkg) = name.strip_prefix("pkg:") {
			let package = model::PackageName::from_str(pkg);

            let java_package = model::PackageName::from_str(value.to_str().unwrap());

			if builder.package_mapping.insert(package, java_package).is_some() {
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
		Ok(JavaOptions {
			output_dir: output_dir,
			package_mapping: builder.package_mapping,
		})
	}

	fn generate<Output : OutputHandler>(model: &model::Verilization, options: Self::Options, output: &mut Output) -> Result<(), GeneratorError> {
		for constant in model.constants() {
			let mut const_gen = JavaConstGenerator::open(&options, output, constant)?;
			const_gen.generate()?;
		}

		for t in model.types() {
			match t {
				model::NamedTypeDefinition::StructType(t) => {
					let mut type_gen: JavaTypeGenerator<_, JavaStructType> = JavaTypeGenerator::open(&options, output, t)?;
					type_gen.generate()?;		
				},
				model::NamedTypeDefinition::EnumType(t) => {
					let mut type_gen: JavaTypeGenerator<_, JavaEnumType> = JavaTypeGenerator::open(&options, output, t)?;
					type_gen.generate()?;		
				},
			}
		}

		Ok(())
	}

}
