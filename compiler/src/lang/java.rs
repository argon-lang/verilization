use crate::model;
use crate::lang::{GeneratorError, Language, OutputHandler};
use std::ffi::OsString;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;
use num_bigint::{BigUint, BigInt, Sign};
use crate::util::for_sep;

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


fn java_package<'a>(package_mapping: &'a PackageMap, package: &model::PackageName) -> Result<&'a model::PackageName, GeneratorError> {
	Ok(package_mapping.get(&package).ok_or(format!("Unmapped package: {}", package))?)
}


pub fn write_qual_name<F : Write>(f: &mut F, package_mapping: &PackageMap, name: &model::QualifiedName) -> Result<(), GeneratorError> {
	let pkg = java_package(&package_mapping, &name.package)?;
	for part in &pkg.package {
		write!(f, "{}.", part)?;
	}

	write!(f, "{}", &name.name)?;

	Ok(())
}



fn open_java_file<'state, Output: OutputHandler>(options: &JavaOptions, output: &'state mut Output, name: &model::QualifiedName) -> Result<Output::FileHandle<'state>, GeneratorError> {
	let java_pkg = java_package(&options.package_mapping, &name.package)?;
	let mut path = PathBuf::from(&options.output_dir);
    for part in &java_pkg.package {
        path.push(part);
    }
	
	path.push(name.name.clone() + ".java");
	Ok(output.create_file(path)?)
}

fn write_package<F : Write>(f: &mut F, package_mapping: &PackageMap, package: &model::PackageName) -> Result<(), GeneratorError> {
	
	let pkg = java_package(package_mapping, package)?;

	let mut pkg_iter = pkg.package.iter();

	if let Some(part) = pkg_iter.next() {
		write!(f, "package {}", part)?;
		while let Some(part) = pkg_iter.next() {
			write!(f, ".{}", part)?;
		}
		writeln!(f, ";")?;
	}

	Ok(())
}

fn write_type_params<F : Write>(f: &mut F, params: &Vec<String>) -> Result<(), GeneratorError> {
	let mut iter = params.iter();
	if let Some(param) = iter.next() {
		write!(f, "<{}", param)?;
		while let Some(param) = iter.next() {
			write!(f, ", {}", param)?;
		}
		write!(f, ">")?;
	}

	Ok(())
}

fn write_type_args<F : Write>(f: &mut F, package_mapping: &PackageMap, version: &BigUint, scope: &model::Scope, args: &Vec<model::Type>) -> Result<(), GeneratorError> {
	let mut iter = args.iter();
	if let Some(arg) = iter.next() {
		write!(f, "<")?;
		write_type(f, package_mapping, version, scope, arg, true)?;
		while let Some(arg) = iter.next() {
			write!(f, ", ")?;
			write_type(f, package_mapping, version, scope, arg, true)?;
		}
		write!(f, ">")?;
	}

	Ok(())
}


fn write_type<F : Write>(f: &mut F, package_mapping: &PackageMap, version: &BigUint, scope: &model::Scope, t: &model::Type, erased: bool) -> Result<(), GeneratorError> {
	Ok(match t {
		// Map built-in types to the equivalent Java type.
		model::Type::Nat | model::Type::Int => write!(f, "java.math.BigInteger")?,
		

        model::Type::U8 | model::Type::I8 if erased => write!(f, "java.lang.Byte")?,
		model::Type::U8 | model::Type::I8 => write!(f, "byte")?,
		
        model::Type::U16 | model::Type::I16 if erased => write!(f, "java.lang.Short")?,
        model::Type::U16 | model::Type::I16 => write!(f, "short")?,

		model::Type::U32 | model::Type::I32 if erased => write!(f, "java.lang.Integer")?,
		model::Type::U32 | model::Type::I32 => write!(f, "int")?,

		model::Type::U64 | model::Type::I64 if erased => write!(f, "java.lang.Long")?,
		model::Type::U64 | model::Type::I64 => write!(f, "long")?,

		model::Type::String => write!(f, "java.lang.String")?,


		model::Type::List(inner) => {
			match **inner {
				model::Type::U8 | model::Type::I8 |
				model::Type::U16 | model::Type::I16 |
				model::Type::U32 | model::Type::I32 |
				model::Type::U64 | model::Type::I64 => {
					write_type(f, package_mapping, version, scope, inner, false)?;
					write!(f, "[]")?;
				},
				_ => {
					write!(f, "java.util.List<")?;
					write_type(f, package_mapping, version, scope, inner, true)?;
					write!(f, ">")?;
				}
			}
		},

		model::Type::Option(inner) => {
			write!(f, "java.util.Optional<")?;
			write_type(f, package_mapping, version, scope, inner, true)?;
			write!(f, ">")?;
		},

		model::Type::Defined(t, args) => match scope.lookup(t.clone()) {
			model::ScopeLookup::NamedType(t) => {
				write_qual_name(f, package_mapping, &t)?;
				write!(f, ".V{}", version)?;
				write_type_args(f, package_mapping, version, scope, args)?;
			},
			model::ScopeLookup::TypeParameter(name) => {
				write!(f, "{}", name)?;
			},
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
					ConvertParam::Expression(param_str) => write!(f, ".apply({})", param_str)?,
				}
			},
			model::ScopeLookup::TypeParameter(name) => {
				write!(f, "{}_conv", name)?;
				if let ConvertParam::Expression(param_str) = param {
					write!(f, ".apply({})", param_str)?;
				}
			},
		},

		model::Type::List(inner) if requires_conversion(inner) =>
			match param {
				ConvertParam::FunctionObject => {
					write!(f, "{}.Util.mapList(", RUNTIME_PACKAGE)?;
					write_version_convert(f, package_mapping, prev_ver, version, scope, inner, ConvertParam::FunctionObject)?;
					write!(f, ")")?;
				},
				ConvertParam::Expression(param_str) => {
					write!(f, "{}.stream().map(", param_str)?;
					write_version_convert(f, package_mapping, prev_ver, version, scope, inner, ConvertParam::FunctionObject)?;
					write!(f, ").collect(Collectors.toList())")?;
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
			ConvertParam::FunctionObject => write!(f, "java.util.function.Function::identity")?,
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
		model::Type::String => write!(f, "{}.StandardCodecs.string", RUNTIME_PACKAGE)?,
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
	match t {
		model::Type::U8 | model::Type::I8 => write!(f, "reader.readByte()")?,
		model::Type::U16 | model::Type::I16 => write!(f, "reader.readShort()")?,
		model::Type::U32 | model::Type::I32 => write!(f, "reader.readInt()")?,
		model::Type::U64 | model::Type::I64 => write!(f, "reader.readLong()")?,
		_ => {
			write_codec(f, package_mapping, version, scope, t)?;
			write!(f, ".read(reader)")?;
		},
	}

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



struct JavaConstGenerator<'opt, 'output, Output> {
	options: &'opt JavaOptions,
	output: &'output mut Output,
}


impl <'opt, 'output, Output: OutputHandler> model::ConstantDefinitionHandler<GeneratorError> for JavaConstGenerator<'opt, 'output, Output> {
	fn constant(&mut self, latest_version: &BigUint, name: &model::QualifiedName, scope: &model::Scope, constant: &model::Constant,_referenced_types: HashSet<&model::QualifiedName>) -> Result<(), GeneratorError> {
		let mut file = open_java_file(self.options, self.output, name)?;

        write_package(&mut file, &self.options.package_mapping, &name.package)?;

        writeln!(file, "public final class {} {{", name.name)?;
        write!(file, "\tpublic static final ")?;
        write_type(&mut file, &self.options.package_mapping, latest_version, scope, &constant.value_type, false)?;
        write!(file, " VALUE = ")?;
		write_constant_value(&mut file, &constant.value)?;
		writeln!(file, ";")?;
		writeln!(file, "}}")?;

		Ok(())
	}
}


struct JavaTypeGenerator<'model, 'output, Output> {
	options: &'model JavaOptions,
	output: &'output mut Output,
}

struct JavaStructType {}
struct JavaEnumType {}

struct JavaTypeGeneratorState<'model, 'opt, 'state, 'scope, Output: OutputHandler, Extra> {
	options: &'opt JavaOptions,
	file: Output::FileHandle<'state>,
	type_name: &'model model::QualifiedName,
	type_params: &'model Vec<String>,
	scope: &'scope model::Scope<'model>,
	versions: HashSet<BigUint>,
	_extra: Extra,
}

trait JavaExtraGeneratorOps {
	fn create_extra() -> Self;
	fn version_class_modifier() -> &'static str;
	fn write_versioned_type_data<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_from_prev_version<F: Write>(f: &mut F, options: &JavaOptions, prev_ver: &BigUint, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_codec_read<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_codec_write<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
}

impl <'model, 'opt, 'output, 'state, 'scope, Output: OutputHandler, Extra: JavaExtraGeneratorOps> model::TypeDefinitionHandlerState<'model, 'state, 'scope, JavaTypeGenerator<'opt, 'output, Output>, GeneratorError> for JavaTypeGeneratorState<'model, 'opt, 'state, 'scope, Output, Extra> where 'model : 'state {
	fn begin(outer: &'state mut JavaTypeGenerator<'opt, 'output, Output>, type_name: &'model model::QualifiedName, type_params: &'model Vec<String>, scope: &'scope model::Scope<'model>,_referenced_types: HashSet<&'model model::QualifiedName>) -> Result<Self, GeneratorError> {
		let mut file = open_java_file(outer.options, outer.output, type_name)?;


		write_package(&mut file, &outer.options.package_mapping, &type_name.package)?;
		writeln!(file, "public abstract class {} {{", type_name.name)?;
		writeln!(file, "\tprivate {}() {{}}", type_name.name)?;

		Ok(JavaTypeGeneratorState {
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

		write!(self.file, "\tpublic static {} class V{}", Extra::version_class_modifier(), version)?;
		write_type_params(&mut self.file, self.type_params)?;
		writeln!(self.file, " extends {} {{", self.type_name.name)?;
		
		Extra::write_versioned_type_data(&mut self.file, &self.options, version, self.scope, type_definition)?;

		if !self.versions.is_empty() {
			write!(self.file, "\t\tpublic static ")?;
			if self.type_params.is_empty() {
				write!(self.file, "final ")?;
			}
			else {
				write!(self.file, "<")?;
				for_sep(&mut self.file, self.type_params, |f| write!(f, ", "),
					|f, param| write!(f, "{}_1, {}_2", param, param)
				)?;
				write!(self.file, "> ")?;
			}
			write!(self.file, "java.util.function.Function<V{}", prev_ver)?;
			if !self.type_params.is_empty() {
				write!(self.file, "<")?;
				for_sep(&mut self.file, self.type_params, |f| write!(f, ", "),
					|f, param| write!(f, "{}_1", param)
				)?;
				write!(self.file, ">")?;
			}
			write!(self.file, ", V{}", version)?;
			if !self.type_params.is_empty() {
				write!(self.file, "<")?;
				for_sep(&mut self.file, self.type_params, |f| write!(f, ", "),
					|f, param| write!(f, "{}_2", param)
				)?;
				write!(self.file, ">")?;
			}
			write!(self.file, "> fromV{}", prev_ver)?;
			if self.type_params.is_empty() {
				writeln!(self.file, " =")?;
				write!(self.file, "\t\t\t")?;
			}
			else {
				write!(self.file, "(")?;
				for_sep(&mut self.file, self.type_params, |f| write!(f, ", "),
					|f, param| write!(f, "{}_conv", param)
				)?;
				writeln!(self.file, ") {{")?;
				write!(self.file, "\t\t\treturn ")?;
			}

			writeln!(self.file, "prev -> {{")?;

			if !explicit_version {
				Extra::write_from_prev_version(&mut self.file, &self.options, prev_ver, version, self.scope, type_definition)?;
			}
			else {
				write!(self.file, "\t\t\t\treturn ")?;
				write_qual_name(&mut self.file, &self.options.package_mapping, self.type_name)?;
				writeln!(self.file, "_Conversions.v{}ToV{}(prev);", prev_ver, version)?;
			}

			writeln!(self.file, "\t\t\t}};")?;
			if !self.type_params.is_empty() {
				writeln!(self.file, "\t\t}}")?;
			}

		}

		write!(self.file, "\t\tprivate static final class CodecImpl")?;
		write_type_params(&mut self.file, &self.type_params)?;
		write!(self.file, " implements {}.Codec<V{}", RUNTIME_PACKAGE, version)?;
		write_type_params(&mut self.file, &self.type_params)?;
		writeln!(self.file, "> {{")?;
		writeln!(self.file, "\t\t\t@Override")?;
		write!(self.file, "\t\t\tpublic V{}", version)?;
		write_type_params(&mut self.file, &self.type_params)?;
		writeln!(self.file, " read({}.FormatReader reader) throws java.io.IOException {{", RUNTIME_PACKAGE)?;
		Extra::write_codec_read(&mut self.file, &self.options, version, self.scope, type_definition)?;
		writeln!(self.file, "\t\t\t}}")?;
		writeln!(self.file, "\t\t\t@Override")?;
		write!(self.file, "\t\t\tpublic void write({}.FormatWriter writer, V{}", RUNTIME_PACKAGE, version)?;
		write_type_params(&mut self.file, &self.type_params)?;
		writeln!(self.file, " value) throws java.io.IOException {{")?;
		Extra::write_codec_write(&mut self.file, &self.options, version, self.scope, type_definition)?;
		writeln!(self.file, "\t\t\t}}")?;
		writeln!(self.file, "\t\t}}")?;

		if self.type_params.is_empty() {
			writeln!(self.file, "\t\tpublic static final {}.Codec<V{}> codec = new CodecImpl();", RUNTIME_PACKAGE, version)?;
		}
		else {
			write!(self.file, "\t\tpublic static ")?;
			write_type_params(&mut self.file, self.type_params)?;
			write!(self.file, "{}.Codec<V{}", RUNTIME_PACKAGE, version)?;
			write_type_params(&mut self.file, self.type_params)?;
			writeln!(self.file, "> codec = new CodecImpl();")?;
		}

		writeln!(self.file, "\t}}")?;

		self.versions.insert(version.clone());

		Ok(())
	}
	
	fn end(mut self) -> Result<(), GeneratorError> {
		writeln!(self.file, "}}")?;
		Ok(())
	}
}

impl JavaExtraGeneratorOps for JavaStructType {
	fn create_extra() -> Self {
		JavaStructType {}
	}

	fn version_class_modifier() -> &'static str {
		"final"
	}

	fn write_versioned_type_data<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		write!(f, "\t\tpublic V{}(", version)?;
		{
			let mut iter = type_definition.fields.iter();
			let mut next_field = iter.next();
			while let Some((field_name, field)) = next_field {
				next_field = iter.next();

				writeln!(f, "")?;
				write!(f, "\t\t\t")?;
				write_type(f, &options.package_mapping, version, scope, &field.field_type, false)?;
				write!(f, " {}", field_name)?;
				if next_field.is_some() {
					write!(f, ",")?;
				}
			}
		}
		if !type_definition.fields.is_empty() {
			writeln!(f, "")?;
			write!(f, "\t\t")?;
		}

		writeln!(f, ") {{")?;
		for (field_name, _) in &type_definition.fields {
			writeln!(f, "\t\t\tthis.{} = {};", field_name, field_name)?;
		}

		writeln!(f, "\t\t}}")?;

		for (field_name, field) in &type_definition.fields {
			write!(f, "\t\tpublic final ")?;
			write_type(f, &options.package_mapping, version, scope, &field.field_type, false)?;
			writeln!(f, " {};", field_name)?;
		}

		writeln!(f, "\t\t@Override")?;
		writeln!(f, "\t\tpublic int hashCode() {{")?;
		write!(f, "\t\t\treturn java.util.Objects.hash(")?;
		{
			let mut iter = type_definition.fields.iter();
			if let Some((field_name, _)) = iter.next() {
				write!(f, "{}", field_name)?;
				while let Some((field_name, _)) = iter.next() {
					write!(f, ", {}", field_name)?;
				}
			}
		}
		writeln!(f, ");")?;
		writeln!(f, "\t\t}}")?;

		writeln!(f, "\t\t@Override")?;
		writeln!(f, "\t\tpublic boolean equals(Object obj) {{")?;
		writeln!(f, "\t\t\tif(!(obj instanceof V{})) return false;", version)?;
		writeln!(f, "\t\t\tV{} other = (V{})obj;", version, version)?;
		for (field_name, _) in &type_definition.fields {
			writeln!(f, "\t\t\tif(!java.util.Objects.deepEquals(this.{}, other.{})) return false;", field_name, field_name)?;
		}
		writeln!(f, "\t\t\treturn true;")?;
		writeln!(f, "\t\t}}")?;

		Ok(())
	}

	fn write_from_prev_version<F: Write>(f: &mut F, options: &JavaOptions, prev_ver: &BigUint, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		write!(f, "\t\t\t\treturn new V{}(", version)?;
		{
			let mut iter = type_definition.fields.iter();
			let mut next_field = iter.next();
			while let Some((field_name, field)) = next_field {
				next_field = iter.next();

				writeln!(f, "")?;
				write!(f, "\t\t\t\t\t")?;
				write_version_convert(f, &options.package_mapping, prev_ver, version, scope, &field.field_type, ConvertParam::Expression(format!("prev.{}", field_name)))?;
				if next_field.is_some() {
					write!(f, ",")?;
				}
			}
		}
		if !type_definition.fields.is_empty() {
			writeln!(f, "")?;
			write!(f, "\t\t\t\t")?;
		}
		writeln!(f, ");")?;

		Ok(())
	}

	fn write_codec_read<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		write!(f, "\t\t\t\treturn new V{}(", version)?;
		{
			let mut iter = type_definition.fields.iter();
			let mut next_field = iter.next();
			while let Some((_, field)) = next_field {
				next_field = iter.next();

				writeln!(f, "")?;
				write!(f, "\t\t\t\t\t")?;
				write_value_read(f, &options.package_mapping, version, scope, &field.field_type)?;
				
				if next_field.is_some() {
					write!(f, ",")?;
				}
			}
		}
		if !type_definition.fields.is_empty() {
			writeln!(f, "")?;
			write!(f, "\t\t\t\t")?;
		}
		writeln!(f, ");")?;

		Ok(())
	}

	fn write_codec_write<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		for (field_name, field) in &type_definition.fields {
			write!(f, "\t\t\t\t")?;
			write_value_write(f, &options.package_mapping, version, scope, &field.field_type, format!("value.{}", field_name))?;
			writeln!(f, ";")?;
		}

		Ok(())
	}
}

impl JavaExtraGeneratorOps for JavaEnumType {
	fn create_extra() -> Self {
		JavaEnumType {}
	}
	
	fn version_class_modifier() -> &'static str {
		"abstract"
	}

	fn write_versioned_type_data<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		writeln!(f, "\t\tprivate V{}() {{}}", version)?;

		for (index, (field_name, field)) in type_definition.fields.iter().enumerate() {
			writeln!(f, "\t\tpublic static final class {} extends V{} {{", field_name, version)?;
			write!(f, "\t\t\tpublic {}(", field_name)?;
			write_type(f, &options.package_mapping, version, scope, &field.field_type, false)?;
			writeln!(f, " {}) {{", field_name)?;
			writeln!(f, "\t\t\t\tthis.{} = {};", field_name, field_name)?;
			writeln!(f, "\t\t\t}}")?;
			write!(f, "\t\t\tpublic final ")?;
			write_type(f, &options.package_mapping, version, scope, &field.field_type, false)?;
			writeln!(f, " {};", field_name)?;
			
			writeln!(f, "\t\t\t@Override")?;
			writeln!(f, "\t\t\tpublic int hashCode() {{")?;
			writeln!(f, "\t\t\t\treturn java.util.Objects.hash({}, this.{});", index, field_name)?;
			writeln!(f, "\t\t\t}}")?;
			
			writeln!(f, "\t\t\t@Override")?;
			writeln!(f, "\t\t\tpublic boolean equals(Object obj) {{")?;
			writeln!(f, "\t\t\t\tif(!(obj instanceof {})) return false;", field_name)?;
			writeln!(f, "\t\t\t\t{} other = ({})obj;", field_name, field_name)?;
			writeln!(f, "\t\t\t\treturn java.util.Objects.deepEquals(this.{}, other.{});", field_name, field_name)?;
			writeln!(f, "\t\t\t}}")?;
	

			writeln!(f, "\t\t}}")?;
		}

		Ok(())
	}

	fn write_from_prev_version<F: Write>(f: &mut F, options: &JavaOptions, prev_ver: &BigUint, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		write!(f, "\t\t\t\t")?;
		for (field_name, field) in &type_definition.fields {
			writeln!(f, "if(prev instanceof V{}.{}) {{", prev_ver, field_name)?;
			write!(f, "\t\t\t\t\treturn new V{}.{}(", version, field_name)?;
			write_version_convert(f, &options.package_mapping, prev_ver, version, scope, &field.field_type, ConvertParam::Expression(format!("((V{}.{})prev).{}", prev_ver, field_name, field_name)))?;
			writeln!(f, ");")?;
			writeln!(f, "\t\t\t\t}}")?;
			write!(f, "\t\t\t\telse ")?;
		}
		if !type_definition.fields.is_empty() {
			writeln!(f, "{{")?;
			write!(f, "\t\t")?;
		}
		writeln!(f, "\t\t\t\tthrow new IllegalArgumentException();")?;
		if !type_definition.fields.is_empty() {
			writeln!(f, "\t\t\t\t}}")?;
		}

		Ok(())
	}
	fn write_codec_read<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		writeln!(f, "\t\t\t\tjava.math.BigInteger tag = {}.StandardCodecs.natCodec.read(reader);", RUNTIME_PACKAGE)?;
		writeln!(f, "\t\t\t\tif(tag.compareTo(java.math.BigInteger.valueOf(java.lang.Integer.MAX_VALUE)) > 0) throw new java.lang.ArithmeticException();")?;
		writeln!(f, "\t\t\t\tswitch(tag.intValue()) {{")?;
		for (index, (field_name, field)) in type_definition.fields.iter().enumerate() {
			writeln!(f, "\t\t\t\t\tcase {}:", index)?;
			write!(f, "\t\t\t\t\t\treturn new V{}.{}(", version, field_name)?;
			write_value_read(f, &options.package_mapping, version, scope, &field.field_type)?;
			writeln!(f, ");")?;
		}
		writeln!(f, "\t\t\t\t\tdefault:")?;
		writeln!(f, "\t\t\t\t\t\tthrow new java.io.IOException(\"Invalid tag number.\");")?;
		writeln!(f, "\t\t\t\t}}")?;

		Ok(())
	}
	fn write_codec_write<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, scope: &model::Scope, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		write!(f, "\t\t\t\t")?;
		for (index, (field_name, field)) in type_definition.fields.iter().enumerate() {
			writeln!(f, "if(value instanceof V{}.{}) {{", version, field_name)?;
			write!(f, "\t\t\t\t\t{}.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf({}))", RUNTIME_PACKAGE, index)?;
			writeln!(f, ";")?;
			write!(f, "\t\t\t\t\t")?;
			write_value_write(f, &options.package_mapping, version, scope, &field.field_type, format!("((V{}.{})value).{}", version, field_name, field_name))?;
			writeln!(f, ";")?;
			writeln!(f, "\t\t\t\t}}")?;
			write!(f, "\t\t\t\telse ")?;
		}
		if !type_definition.fields.is_empty() {
			writeln!(f, "{{")?;
			write!(f, "\t")?;
		}
		writeln!(f, "\t\t\t\tthrow new IllegalArgumentException();")?;
		if !type_definition.fields.is_empty() {
			writeln!(f, "\t\t\t\t}}")?;
		}

		Ok(())
	}
}


impl <'model, 'opt, 'output, Output: OutputHandler> model::TypeDefinitionHandler<'model, GeneratorError> for JavaTypeGenerator<'opt, 'output, Output> {
	type StructHandlerState<'state, 'scope> where 'model : 'scope, 'scope : 'state = JavaTypeGeneratorState<'model, 'opt, 'state, 'scope, Output, JavaStructType>;
	type EnumHandlerState<'state, 'scope> where 'model : 'scope, 'scope : 'state = JavaTypeGeneratorState<'model, 'opt, 'state, 'scope, Output, JavaEnumType>;
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
		let mut const_gen = JavaConstGenerator {
			options: &options,
			output: output,
		};

		model.iter_constants(&mut const_gen)?;

		let mut type_gen = JavaTypeGenerator {
			options: &options,
			output: output,
		};

		model.iter_types(&mut type_gen)?;

		Ok(())
	}

}
