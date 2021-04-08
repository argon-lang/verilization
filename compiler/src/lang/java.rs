use crate::model;
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


fn java_package<'a>(package_mapping: &'a PackageMap, package: &model::PackageName) -> Result<&'a model::PackageName, GeneratorError> {
	Ok(package_mapping.get(&package).ok_or(format!("Unmapped package: {}", package))?)
}


fn write_qual_name<F : Write>(f: &mut F, package_mapping: &PackageMap, name: &model::QualifiedName) -> Result<(), GeneratorError> {
	let pkg = java_package(&package_mapping, &name.package)?;
	for part in &pkg.package {
		write!(f, "{}.", part)?;
	}

	write!(f, "{}", &name.name)?;

	Ok(())
}




fn open_java_file<'a, Output: OutputHandler<'a>>(options: &JavaOptions, output: &'a mut Output, name: &model::QualifiedName) -> Result<Output::FileHandle, GeneratorError> {
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

fn write_type<F : Write>(f: &mut F, package_mapping: &PackageMap, version: &BigUint, t: &model::Type, erased: bool) -> Result<(), GeneratorError> {
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
					write_type(f, package_mapping, version, inner, false)?;
					write!(f, "[]")?;
				},
				_ => {
					write!(f, "java.util.List<")?;
					write_type(f, package_mapping, version, inner, true)?;
					write!(f, ">")?;
				}
			}
		},

		model::Type::Option(inner) => {
			write!(f, "java.util.Optional<")?;
			write_type(f, package_mapping, version, inner, true)?;
			write!(f, ">")?;
		},

		model::Type::Defined(t) => {
			write_qual_name(f, package_mapping, t)?;
			write!(f, ".V{}", version)?;
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
		model::Type::Defined(_) => true,
		_ => false,
	}
}

enum ConvertParam {
	FunctionObject,
	Expression(String),
}

fn write_version_convert<'a, F : Write>(f: &mut F, package_mapping: &PackageMap, prev_ver: &BigUint, version: &BigUint, field_type: &model::Type, param: ConvertParam) -> Result<(), GeneratorError> {
	match field_type {
		model::Type::Defined(_) => {
			write_type(f, package_mapping, version, field_type, false)?;
			write!(f, ".V{}", version)?;
			match param {
				ConvertParam::FunctionObject => write!(f, "::")?,
				ConvertParam::Expression(_) => write!(f, ".")?,
			}
			write!(f, "fromV{}", prev_ver)?;
			if let ConvertParam::Expression(param_str) = param {
				write!(f, "({})", param_str)?;
			}
		},

		model::Type::List(inner) if requires_conversion(inner) =>
			match param {
				ConvertParam::FunctionObject => {
					write!(f, "{}.Util.mapList(", RUNTIME_PACKAGE)?;
					write_version_convert(f, package_mapping, prev_ver, version, inner, ConvertParam::FunctionObject)?;
					write!(f, ")")?;
				},
				ConvertParam::Expression(param_str) => {
					write!(f, "{}.stream().map(", param_str)?;
					write_version_convert(f, package_mapping, prev_ver, version, inner, ConvertParam::FunctionObject)?;
					write!(f, ").collect(Collectors.toList())")?;
				},
			},

		model::Type::Option(inner) if requires_conversion(inner) => 
			match param {
				ConvertParam::FunctionObject => {
					write!(f, "{}.Util.mapOption(", RUNTIME_PACKAGE)?;
					write_version_convert(f, package_mapping, prev_ver, version, inner, ConvertParam::FunctionObject)?;
					write!(f, ")")?;
				},
				ConvertParam::Expression(param_str) => {
					write!(f, "{}.map(", param_str)?;
					write_version_convert(f, package_mapping, prev_ver, version, inner, ConvertParam::FunctionObject)?;
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


fn write_codec<F : Write>(f: &mut F, package_mapping: &PackageMap, version: &BigUint, t: &model::Type) -> Result<(), GeneratorError> {
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
					write_codec(f, package_mapping, version, inner)?;
					write!(f, ")")?
				},
			}
		},
		model::Type::Option(inner) => {
			write!(f, "{}.StandardCodecs.option(", RUNTIME_PACKAGE)?;
			write_codec(f, package_mapping, version, inner)?;
			write!(f, ")")?
		},
		model::Type::Defined(_) => {
			write_type(f, package_mapping, version, t, false)?;
			write!(f, ".codec")?;
		},
	}

	Ok(())
}

fn write_value_read<F : Write>(f: &mut F, package_mapping: &PackageMap, version: &BigUint, t: &model::Type) -> Result<(), GeneratorError> {
	match t {
		model::Type::U8 | model::Type::I8 => write!(f, "reader.readByte()")?,
		model::Type::U16 | model::Type::I16 => write!(f, "reader.readShort()")?,
		model::Type::U32 | model::Type::I32 => write!(f, "reader.readInt()")?,
		model::Type::U64 | model::Type::I64 => write!(f, "reader.readLong()")?,
		_ => {
			write_codec(f, package_mapping, version, t)?;
			write!(f, ".read(reader)")?;
		},
	}

	Ok(())
}

fn write_value_write<F : Write>(f: &mut F, package_mapping: &PackageMap, version: &BigUint, t: &model::Type, value: String) -> Result<(), GeneratorError> {
	match t {
		model::Type::U8 | model::Type::I8 => write!(f, "writer.writeByte({})", value)?,
		model::Type::U16 | model::Type::I16 => write!(f, "writer.writeShort({})", value)?,
		model::Type::U32 | model::Type::I32 => write!(f, "writer.writeInt({})", value)?,
		model::Type::U64 | model::Type::I64 => write!(f, "writer.writeLong({})", value)?,
		_ => {
			write_codec(f, package_mapping, version, t)?;
			write!(f, ".write(writer, {})", value)?;
		},
	}

	Ok(())
}



struct JavaConstGenerator<'model, Output> {
	options: &'model JavaOptions,
	output: &'model mut Output,
}


impl <'model, Output: for<'a> OutputHandler<'a>> model::ConstantDefinitionHandler<GeneratorError> for JavaConstGenerator<'model, Output> {
	fn constant(&mut self, latest_version: &BigUint, name: &model::QualifiedName, constant: &model::Constant, _referenced_types: HashSet<&model::QualifiedName>) -> Result<(), GeneratorError> {
		let mut file = open_java_file(self.options, self.output, name)?;

        write_package(&mut file, &self.options.package_mapping, &name.package)?;

        writeln!(file, "public final class {} {{", name.name)?;
        write!(file, "\tpublic static final ")?;
        write_type(&mut file, &self.options.package_mapping, latest_version, &constant.value_type, false)?;
        write!(file, " VALUE = ")?;
		write_constant_value(&mut file, &constant.value)?;
		writeln!(file, ";")?;
		writeln!(file, "}}")?;

		Ok(())
	}
}


struct JavaTypeGenerator<'model, Output> {
	options: &'model JavaOptions,
	output: &'model mut Output,
}

struct JavaStructType {}
struct JavaEnumType {}

struct JavaTypeGeneratorState<'model, 'state, Output: OutputHandler<'state>, Extra> {
	options: &'model JavaOptions,
	file: Output::FileHandle,
	versions: HashSet<BigUint>,
	_extra: Extra,
}

trait JavaExtraGeneratorOps {
	fn create_extra() -> Self;
	fn version_class_modifier() -> &'static str;
	fn write_versioned_type_data<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_from_prev_version<F: Write>(f: &mut F, options: &JavaOptions, prev_ver: &BigUint, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_codec_read<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
	fn write_codec_write<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError>;
}

impl <'model, 'state, Output: for<'a> OutputHandler<'a>, Extra: JavaExtraGeneratorOps> model::TypeDefinitionHandlerState<'model, 'state, JavaTypeGenerator<'model, Output>, GeneratorError> for JavaTypeGeneratorState<'model, 'state, Output, Extra> {
	fn begin(outer: &'state mut JavaTypeGenerator<'model, Output>, type_name: &'model model::QualifiedName, _referenced_types: HashSet<&'model model::QualifiedName>) -> Result<Self, GeneratorError> where 'model : 'state {
		let mut file = open_java_file(outer.options, outer.output, type_name)?;


		write_package(&mut file, &outer.options.package_mapping, &type_name.package)?;
		writeln!(file, "public abstract class {} {{", type_name.name)?;
		writeln!(file, "\tprivate {}() {{}}", type_name.name)?;

		Ok(JavaTypeGeneratorState {
			options: outer.options,
			file: file,
			versions: HashSet::new(),
			_extra: Extra::create_extra(),
		})
	}

	fn versioned_type(&mut self, explicit_version: bool, type_name: &model::QualifiedName, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {

		let prev_ver: BigInt = BigInt::from_biguint(Sign::Plus, version.clone()) - 1;
		let prev_ver = prev_ver.magnitude();

		writeln!(self.file, "\tpublic static {} class V{} extends {} {{", Extra::version_class_modifier(), version, type_name.name)?;
		
		Extra::write_versioned_type_data(&mut self.file, &self.options, version, type_definition)?;

		if !self.versions.is_empty() {
			writeln!(self.file, "\t\tpublic static V{} fromV{}(V{} prev) {{", version, prev_ver, prev_ver)?;
			if !explicit_version {
				Extra::write_from_prev_version(&mut self.file, &self.options, prev_ver, version, type_definition)?;
			}
			else {
				write!(self.file, "\t\t\treturn ")?;
				write_qual_name(&mut self.file, &self.options.package_mapping, type_name)?;
				writeln!(self.file, "_Conversions.v{}ToV{}(prev);", prev_ver, version)?;
			}
			writeln!(self.file, "\t\t}}")?;
		}

		writeln!(self.file, "\t\tprivate static final class CodecImpl implements {}.Codec<V{}> {{", RUNTIME_PACKAGE, version)?;
		writeln!(self.file, "\t\t\t@Override")?;
		writeln!(self.file, "\t\t\tpublic V{} read({}.FormatReader reader) throws java.io.IOException {{", version, RUNTIME_PACKAGE)?;
		Extra::write_codec_read(&mut self.file, &self.options, version, type_definition)?;
		writeln!(self.file, "\t\t\t}}")?;
		writeln!(self.file, "\t\t\t@Override")?;
		writeln!(self.file, "\t\t\tpublic void write({}.FormatWriter writer, V{} value) throws java.io.IOException {{", RUNTIME_PACKAGE, version)?;
		Extra::write_codec_write(&mut self.file, &self.options, version, type_definition)?;
		writeln!(self.file, "\t\t\t}}")?;
		writeln!(self.file, "\t\t}}")?;

		writeln!(self.file, "\t\tpublic static final {}.Codec<V{}> codec = new CodecImpl();", RUNTIME_PACKAGE, version)?;

		writeln!(self.file, "\t}}")?;

		self.versions.insert(version.clone());

		Ok(())
	}
	
	fn end(mut self, _type_name: &model::QualifiedName) -> Result<(), GeneratorError> {
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

	fn write_versioned_type_data<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		write!(f, "\t\tpublic V{}(", version)?;
		{
			let mut iter = type_definition.fields.iter();
			let mut next_field = iter.next();
			while let Some((field_name, field)) = next_field {
				next_field = iter.next();

				writeln!(f, "")?;
				write!(f, "\t\t\t")?;
				write_type(f, &options.package_mapping, version, &field.field_type, false)?;
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
			write_type(f, &options.package_mapping, version, &field.field_type, false)?;
			writeln!(f, " {};", field_name)?;
		}

		Ok(())
	}

	fn write_from_prev_version<F: Write>(f: &mut F, options: &JavaOptions, prev_ver: &BigUint, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		write!(f, "\t\t\treturn new V{}(", version)?;
		{
			let mut iter = type_definition.fields.iter();
			let mut next_field = iter.next();
			while let Some((field_name, field)) = next_field {
				next_field = iter.next();

				writeln!(f, "")?;
				write!(f, "\t\t\t\t")?;
				write_version_convert(f, &options.package_mapping, prev_ver, version, &field.field_type, ConvertParam::Expression(format!("prev.{}", field_name)))?;
				if next_field.is_some() {
					write!(f, ",")?;
				}
			}
		}
		if !type_definition.fields.is_empty() {
			writeln!(f, "")?;
			write!(f, "\t\t\t")?;
		}
		writeln!(f, ");")?;

		Ok(())
	}

	fn write_codec_read<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		write!(f, "\t\t\t\treturn new V{}(", version)?;
		{
			let mut iter = type_definition.fields.iter();
			let mut next_field = iter.next();
			while let Some((_, field)) = next_field {
				next_field = iter.next();

				writeln!(f, "")?;
				write!(f, "\t\t\t\t\t")?;
				write_value_read(f, &options.package_mapping, version, &field.field_type)?;
				
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

	fn write_codec_write<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		for (field_name, field) in &type_definition.fields {
			write!(f, "\t\t\t\t")?;
			write_value_write(f, &options.package_mapping, version, &field.field_type, format!("value.{}", field_name))?;
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

	fn write_versioned_type_data<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		writeln!(f, "\t\tprivate V{}() {{}}", version)?;

		for (field_name, field) in &type_definition.fields {
			writeln!(f, "\t\tpublic static final class {} extends V{} {{", field_name, version)?;
			write!(f, "\t\t\tpublic {}(", field_name)?;
			write_type(f, &options.package_mapping, version, &field.field_type, false)?;
			writeln!(f, " {}) {{", field_name)?;
			writeln!(f, "\t\t\t\tthis.{} = {};", field_name, field_name)?;
			writeln!(f, "\t\t\t}}")?;
			write!(f, "\t\t\tpublic final ")?;
			write_type(f, &options.package_mapping, version, &field.field_type, false)?;
			writeln!(f, " {};", field_name)?;
			writeln!(f, "\t\t}}")?;
		}

		Ok(())
	}

	fn write_from_prev_version<F: Write>(f: &mut F, options: &JavaOptions, prev_ver: &BigUint, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		write!(f, "\t\t\t")?;
		for (field_name, field) in &type_definition.fields {
			writeln!(f, "if(prev instanceof V{}.{}) {{", prev_ver, field_name)?;
			write!(f, "\t\t\t\treturn new V{}.{}(", version, field_name)?;
			write_version_convert(f, &options.package_mapping, prev_ver, version, &field.field_type, ConvertParam::Expression(format!("((V{}.{})prev).{}", prev_ver, field_name, field_name)))?;
			writeln!(f, ");")?;
			writeln!(f, "\t\t\t}}")?;
			write!(f, "\t\t\telse ")?;
		}
		if !type_definition.fields.is_empty() {
			writeln!(f, "{{")?;
			write!(f, "\t")?;
		}
		writeln!(f, "\t\t\tthrow new IllegalArgumentException();")?;
		if !type_definition.fields.is_empty() {
			writeln!(f, "\t\t\t}}")?;
		}

		Ok(())
	}
	fn write_codec_read<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		writeln!(f, "\t\t\t\tjava.math.BigInteger tag = {}.StandardCodecs.natCodec.read(reader);", RUNTIME_PACKAGE)?;
		writeln!(f, "\t\t\t\tif(tag.compareTo(java.math.BigInteger.valueOf(java.lang.Integer.MAX_VALUE)) > 0) throw new java.lang.ArithmeticException();")?;
		writeln!(f, "\t\t\t\tswitch(tag.intValue()) {{")?;
		for (index, (field_name, field)) in type_definition.fields.iter().enumerate() {
			writeln!(f, "\t\t\t\t\tcase {}:", index)?;
			write!(f, "\t\t\t\t\t\treturn new V{}.{}(", version, field_name)?;
			write_value_read(f, &options.package_mapping, version, &field.field_type)?;
			writeln!(f, ");")?;
		}
		writeln!(f, "\t\t\t\t\tdefault:")?;
		writeln!(f, "\t\t\t\t\t\tthrow new java.io.IOException(\"Invalid tag number.\");")?;
		writeln!(f, "\t\t\t\t}}")?;

		Ok(())
	}
	fn write_codec_write<F: Write>(f: &mut F, options: &JavaOptions, version: &BigUint, type_definition: &model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
		write!(f, "\t\t\t\t")?;
		for (index, (field_name, field)) in type_definition.fields.iter().enumerate() {
			writeln!(f, "if(value instanceof V{}.{}) {{", version, field_name)?;
			write!(f, "\t\t\t\t\t{}.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf({}))", RUNTIME_PACKAGE, index)?;
			writeln!(f, ";")?;
			write!(f, "\t\t\t\t\t")?;
			write_value_write(f, &options.package_mapping, version, &field.field_type, format!("((V{}.{})value).{}", version, field_name, field_name))?;
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


impl <'model, 'state, Output: for<'a> OutputHandler<'a>> model::TypeDefinitionHandler<'model, 'state, GeneratorError> for JavaTypeGenerator<'model, Output> {
	type StructHandlerState = JavaTypeGeneratorState<'model, 'state, Output, JavaStructType>;
	type EnumHandlerState = JavaTypeGeneratorState<'model, 'state, Output, JavaEnumType>;
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

	fn generate<Output : for<'a> OutputHandler<'a>>(model: &model::Verilization, options: Self::Options, output: &mut Output) -> Result<(), GeneratorError> {
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

	fn write_codec<F: Write>(file: &mut F, options: &Self::Options, version: &BigUint, _type_name: Option<&model::QualifiedName>, t: &model::Type) -> Result<(), GeneratorError> {
		write_codec(file, &options.package_mapping, version, t)
	}

}
