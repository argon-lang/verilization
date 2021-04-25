use crate::model;
use model::Named;
use crate::lang::{GeneratorError, Language, OutputHandler};
use std::ffi::OsString;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;
use num_bigint::{BigUint, BigInt, Sign};
use super::generator::*;

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

pub trait JavaGenerator<'model, 'opt> : Generator<'model, JavaLanguage> + GeneratorWithFile {
	fn options(&self) -> &'opt JavaOptions;
	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model>;

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
	
	fn write_type_args(&mut self, args: &Vec<LangType<'model>>) -> Result<(), GeneratorError> {
		if !args.is_empty() {
			write!(self.file(), "<")?;
			for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
				self.write_type(arg, true)?;
			});
			write!(self.file(), ">")?;

		}
	
		Ok(())
	}
	
	
	fn write_type(&mut self, t: &LangType<'model>, erased: bool) -> Result<(), GeneratorError> {
		Ok(match t {
			// Map built-in types to the equivalent Java type.
			LangType::Nat | LangType::Int => write!(self.file(), "java.math.BigInteger")?,
			
	
			LangType::U8 | LangType::I8 if erased => write!(self.file(), "java.lang.Byte")?,
			LangType::U8 | LangType::I8 => write!(self.file(), "byte")?,
			
			LangType::U16 | LangType::I16 if erased => write!(self.file(), "java.lang.Short")?,
			LangType::U16 | LangType::I16 => write!(self.file(), "short")?,
	
			LangType::U32 | LangType::I32 if erased => write!(self.file(), "java.lang.Integer")?,
			LangType::U32 | LangType::I32 => write!(self.file(), "int")?,
	
			LangType::U64 | LangType::I64 if erased => write!(self.file(), "java.lang.Long")?,
			LangType::U64 | LangType::I64 => write!(self.file(), "long")?,
	
			LangType::String => write!(self.file(), "java.lang.String")?,
	
	
			LangType::List(inner) => {
				write!(self.file(), "{}.List<", RUNTIME_PACKAGE)?;
				self.write_type(inner, true)?;
				write!(self.file(), ">")?;
			},
	
			LangType::Option(inner) => {
				write!(self.file(), "java.util.Optional<")?;
				self.write_type(inner, true)?;
				write!(self.file(), ">")?;
			},
	
			LangType::Versioned(name, version, args) => {
				self.write_qual_name(name)?;
				write!(self.file(), ".V{}", version)?;
				self.write_type_args(args)?;
			},

			LangType::TypeParameter(name) => {
				write!(self.file(), "{}", name)?;
			},

			LangType::Converter(from, to) => {
				write!(self.file(), "{}.Converter<", RUNTIME_PACKAGE)?;
				self.write_type(&*from, true)?;
				write!(self.file(), ", ")?;
				self.write_type(&*to, true)?;
				write!(self.file(), ">")?;
			},

			LangType::Codec(t) => {
				write!(self.file(), "{}.Codec<", RUNTIME_PACKAGE)?;
				self.write_type(&*t, true)?;
				write!(self.file(), ">")?;
			},
		})
	}

	fn write_type_no_params(&mut self, t: &LangType<'model>) -> Result<(), GeneratorError> {
		Ok(match t {
			// Map built-in types to the equivalent Java type.
			LangType::Nat | LangType::Int => write!(self.file(), "java.math.BigInteger")?,
			
	
			LangType::U8 | LangType::I8 => write!(self.file(), "java.lang.Byte")?,
			LangType::U16 | LangType::I16 => write!(self.file(), "java.lang.Short")?,
			LangType::U32 | LangType::I32 => write!(self.file(), "java.lang.Integer")?,
			LangType::U64 | LangType::I64 => write!(self.file(), "java.lang.Long")?,
	
			LangType::String => write!(self.file(), "java.lang.String")?,
	
			LangType::List(_) => write!(self.file(), "{}.List", RUNTIME_PACKAGE)?,
			LangType::Option(_) => write!(self.file(), "java.util.Optional<")?,
	
			LangType::Versioned(name, version, _) => {
				self.write_qual_name(name)?;
				write!(self.file(), ".V{}", version)?;
			},

			LangType::TypeParameter(name) => {
				write!(self.file(), "{}", name)?;
			},

			LangType::Converter(_, _) => write!(self.file(), "{}.Converter", RUNTIME_PACKAGE)?,

			LangType::Codec(_) => write!(self.file(), "{}.Codec", RUNTIME_PACKAGE)?,
		})
	}

	fn write_args(&mut self, args: &Vec<LangExpr<'model>>) -> Result<(), GeneratorError> {
		if !args.is_empty() {
			write!(self.file(), "(")?;
			for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
				self.write_expr(&arg)?;
			});
			write!(self.file(), ")")?;
		}

		Ok(())
	}

	fn write_operation_name(&mut self, op: &Operation) -> Result<(), GeneratorError> {
		match op {
			Operation::FromPreviousVersion(prev_ver) => write!(self.file(), "fromV{}", prev_ver)?,
			Operation::FinalTypeConverter => write!(self.file(), "converter")?,
			Operation::VersionedTypeCodec => write!(self.file(), "codec")?,
		}

		Ok(())
	}
	
	fn write_expr(&mut self, expr: &LangExpr<'model>) -> Result<(), GeneratorError> {
		match expr {
			LangExpr::Identifier(name) => write!(self.file(), "{}", name)?,
			LangExpr::InvokeConverter { converter, value } => {
				self.write_expr(&*converter)?;
				write!(self.file(), ".convert(")?;
				self.write_expr(&*value)?;
				write!(self.file(), ")")?;
			},
			LangExpr::IdentityConverter(t) => {
				write!(self.file(), "{}.Converter.<", RUNTIME_PACKAGE)?;
				self.write_type(t, true)?;
				write!(self.file(), ">identity()")?;
			},
			LangExpr::MapListConverter { from_type, to_type, element_converter } => {
				write!(self.file(), "{}.List.<", RUNTIME_PACKAGE)?;
				self.write_type(from_type, true)?;
				write!(self.file(), ", ")?;
				self.write_type(to_type, true)?;
				write!(self.file(), ">converter(")?;
				self.write_expr(element_converter)?;
				write!(self.file(), ")")?;
				
			},
			LangExpr::MapOptionConverter { from_type, to_type, element_converter } => {
				write!(self.file(), "{}.Option.<", RUNTIME_PACKAGE)?;
				self.write_type(from_type, true)?;
				write!(self.file(), ", ")?;
				self.write_type(to_type, true)?;
				write!(self.file(), ">converter(")?;
				self.write_expr(element_converter)?;
				write!(self.file(), ")")?;
			},
			LangExpr::NatCodec => write!(self.file(), "{}.StandardCodecs.natCodec", RUNTIME_PACKAGE)?,
			LangExpr::IntCodec => write!(self.file(), "{}.StandardCodecs.intCodec", RUNTIME_PACKAGE)?,
			LangExpr::U8Codec | LangExpr::I8Codec => write!(self.file(), "{}.StandardCodecs.i8Codec", RUNTIME_PACKAGE)?,
			LangExpr::U16Codec | LangExpr::I16Codec => write!(self.file(), "{}.StandardCodecs.i16Codec", RUNTIME_PACKAGE)?,
			LangExpr::U32Codec | LangExpr::I32Codec => write!(self.file(), "{}.StandardCodecs.i32Codec", RUNTIME_PACKAGE)?,
			LangExpr::U64Codec | LangExpr::I64Codec => write!(self.file(), "{}.StandardCodecs.i64Codec", RUNTIME_PACKAGE)?,
			LangExpr::StringCodec => write!(self.file(), "{}.StandardCodecs.stringCodec", RUNTIME_PACKAGE)?,
			LangExpr::ListCodec(inner) => {
				write!(self.file(), "{}.List.codec(", RUNTIME_PACKAGE)?;
				self.write_expr(&*inner)?;
				write!(self.file(), ")")?
			},
			LangExpr::OptionCodec(inner) => {
				write!(self.file(), "{}.StandardCodecs.option(", RUNTIME_PACKAGE)?;
				self.write_expr(&*inner)?;
				write!(self.file(), ")")?
			},
			LangExpr::ReadDiscriminator => write!(self.file(), "{}.StandardCodecs.natCodec.read(reader)", RUNTIME_PACKAGE)?,
			LangExpr::WriteDiscriminator(value) => write!(self.file(), "{}.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf({}))", RUNTIME_PACKAGE, value)?,
			LangExpr::CodecRead { codec } => {
				self.write_expr(&*codec)?;
				write!(self.file(), ".read(reader)")?;
			},
			LangExpr::CodecWrite { codec, value } => {
				self.write_expr(&*codec)?;
				write!(self.file(), ".write(writer, ")?;
				self.write_expr(value)?;
				write!(self.file(), ")")?;
			},
			LangExpr::InvokeOperation(op, name, version, type_args, args) => {
				self.write_qual_name(name)?;
				write!(self.file(), ".V{}.", version)?;
				self.write_type_args(type_args)?;
				self.write_operation_name(op)?;
				self.write_args(args)?;
			},
			LangExpr::InvokeUserConverter { name, prev_ver, version, type_args, args } => {
				self.write_qual_name(name)?;
				write!(self.file(), "_Conversions.")?;
				self.write_type_args(type_args)?;
				write!(self.file(), "v{}ToV{}", prev_ver, version)?;
				self.write_args(args)?;
			},
			LangExpr::ConstantValue(name, version) => {
				self.write_qual_name(name)?;
				write!(self.file(), ".{}", Self::constant_version_name(version))?;
			},
			LangExpr::CreateStruct(name, version, type_args, fields) => {
				write!(self.file(), "new ")?;
				self.write_qual_name(name)?;
				write!(self.file(), ".V{}", version)?;
				self.write_type_args(type_args)?;
				write!(self.file(), "(")?;
				for_sep!((_, value), fields, { write!(self.file(), ", ")?; }, {
					self.write_expr(value)?;
				});
				write!(self.file(), ")")?;
			},
			LangExpr::CreateEnum(name, version, type_args, field_name, value) => {
				write!(self.file(), "new ")?;
				self.write_qual_name(name)?;
				write!(self.file(), ".V{}.{}", version, field_name)?;
				self.write_type_args(type_args)?;
				write!(self.file(), "(")?;
				self.write_expr(value)?;
				write!(self.file(), ")")?;
			},
			LangExpr::StructField(_, _, field_name, value) => {
				self.write_expr(value)?;
				write!(self.file(), ".{}", field_name)?;
			},
		}

		Ok(())
	}
}

impl <'model, 'opt, TImpl> GeneratorNameMapping<JavaLanguage> for TImpl where TImpl : JavaGenerator<'model, 'opt> {
	fn convert_prev_type_param(param: &str) -> String {
		format!("{}_1", param)
	}

	fn convert_current_type_param(param: &str) -> String {
		format!("{}_2", param)
	}

	fn convert_conv_param_name(param: &str) -> String {
		format!("{}_conv", param)
	}

	fn convert_prev_param_name() -> &'static str {
		"prev"
	}

	fn codec_write_value_name() -> &'static str {
		"value"
	}

	fn codec_codec_param_name(param: &str) -> String {
		format!("{}_codec", param)
	}

	fn constant_version_name(version: &BigUint) -> String {
		format!("v{}", version)
	}
}


struct JavaConstGenerator<'model, 'opt, 'output, Output: OutputHandler> {
	file: Output::FileHandle<'output>,
	model: &'model model::Verilization,
	options: &'opt JavaOptions,
	constant: Named<'model, model::Constant>,
	scope: model::Scope<'model>,
}

impl <'model, 'opt, 'output, Output: OutputHandler> Generator<'model, JavaLanguage> for JavaConstGenerator<'model, 'opt, 'output, Output> {
	fn model(&self) -> &'model model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> GeneratorWithFile for JavaConstGenerator<'model, 'opt, 'output, Output> {
	type GeneratorFile = Output::FileHandle<'output>;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> JavaGenerator<'model, 'opt> for JavaConstGenerator<'model, 'opt, 'output, Output> {
	fn options(&self) -> &'opt JavaOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.constant.referenced_types()
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> ConstGenerator<'model, JavaLanguage> for JavaConstGenerator<'model, 'opt, 'output, Output> {
	fn constant(&self) -> Named<'model, model::Constant> {
		self.constant
	}

	fn write_header(&mut self) -> Result<(), GeneratorError> {
        self.write_package(&self.constant.name().package)?;

		writeln!(self.file, "public final class {} {{", self.constant.name().name)?;

		Ok(())
	}

	fn write_constant(&mut self, version_name: String, t: LangType<'model>, value: LangExpr<'model>) -> Result<(), GeneratorError> {
		write!(self.file, "\tpublic static final ")?;
		self.write_type(&t, false)?;
		write!(self.file, " {} = ", version_name)?;
		self.write_expr(&value)?;
		writeln!(self.file, ";")?;

		Ok(())
	}

	fn write_footer(&mut self) -> Result<(), GeneratorError> {
		writeln!(self.file, "}}")?;
		Ok(())
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> JavaConstGenerator<'model, 'opt, 'output, Output> {

	fn open(model: &'model model::Verilization, options: &'opt JavaOptions, output: &'output mut Output, constant: Named<'model, model::Constant>) -> Result<Self, GeneratorError> {
		let file = open_java_file(options, output, constant.name())?;
		Ok(JavaConstGenerator {
			file: file,
			model: model,
			options: options,
			constant: constant,
			scope: constant.scope(),
		})
	}
}

struct JavaTypeGenerator<'model, 'opt, 'output, Output: OutputHandler, Extra> {
	file: Output::FileHandle<'output>,
	model: &'model model::Verilization,
	options: &'opt JavaOptions,
	type_def: Named<'model, model::TypeDefinitionData>,
	scope: model::Scope<'model>,
	versions: HashSet<BigUint>,
	indentation_level: u32,
	_extra: Extra,
}

trait JavaExtraGeneratorOps {
	fn version_class_modifier() -> &'static str;
	fn write_versioned_type_data(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> Generator<'model, JavaLanguage> for JavaTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	fn model(&self) -> &'model model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> GeneratorWithFile for JavaTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	type GeneratorFile = Output::FileHandle<'output>;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> Indentation for JavaTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	fn indentation_size(&mut self) -> &mut u32 {
		&mut self.indentation_level
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> JavaGenerator<'model, 'opt> for JavaTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	fn options(&self) -> &'opt JavaOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.type_def.referenced_types()
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler, GenTypeKind> VersionedTypeGenerator<'model, JavaLanguage, GenTypeKind> for JavaTypeGenerator<'model, 'opt, 'output, Output, GenTypeKind>
	where JavaTypeGenerator<'model, 'opt, 'output, Output, GenTypeKind> : JavaExtraGeneratorOps
{
	fn type_def(&self) -> Named<'model, model::TypeDefinitionData> {
		self.type_def
	}

	fn write_header(&mut self) -> Result<(), GeneratorError> {
		self.write_package(&self.type_def.name().package)?;
		writeln!(self.file, "public abstract class {} {{", self.type_def.name().name)?;
		self.indent_increase();
		self.write_indent()?;
		writeln!(self.file, "private {}() {{}}", self.type_def.name().name)?;
		
		Ok(())
	}

	fn write_version_header(&mut self, ver_type: &model::TypeVersionInfo<'model>) -> Result<(), GeneratorError> {
		let version = &ver_type.version;

		let prev_ver: BigInt = BigInt::from_biguint(Sign::Plus, version.clone()) - 1;
		let prev_ver = prev_ver.magnitude();

		self.write_indent()?;
		write!(self.file, "public static {} class V{}", Self::version_class_modifier(), version)?;
		self.write_type_params(self.type_def().type_params())?;
		writeln!(self.file, " extends {} {{", self.type_def.name().name)?;
		self.indent_increase();
		
		self.write_versioned_type_data(ver_type)?;

		Ok(())
	}

	fn write_operation(&mut self, operation: OperationInfo<'model>) -> Result<(), GeneratorError> {
		let is_func = !operation.type_params.is_empty() || !operation.params.is_empty();

		self.write_indent()?;
		write!(self.file, "public static ")?;
		if !is_func {
			write!(self.file, "final ")?;
		}
		self.write_type_params(&operation.type_params)?;
		if !operation.type_params.is_empty() {
			write!(self.file, " ")?;
		}

		self.write_type(&operation.result, false)?;
		write!(self.file, " ")?;

		self.write_operation_name(&operation.operation)?;

		if is_func {
			write!(self.file, "(")?;
			for_sep!((param_name, param), operation.params, { write!(self.file, ", ")?; }, {
				self.write_type(&param, true)?;
				write!(self.file, " {}", param_name)?;
			});
			writeln!(self.file, ") {{")?;
			self.indent_increase();
		}
		else {
			write!(self.file, " = ")?;
		}

		self.write_expr_statement(&operation.implementation, !is_func)?;

		if is_func {
			self.indent_decrease();
			self.write_indent()?;
			writeln!(self.file, "}}")?;
		}

		Ok(())
	}

	fn write_version_footer(&mut self, _ver_type: &model::TypeVersionInfo<'model>) -> Result<(), GeneratorError> {
		self.indent_decrease();
		writeln!(self.file, "}}")?;

		Ok(())
	}

	fn write_footer(&mut self) -> Result<(), GeneratorError> {
		self.indent_decrease();
		writeln!(self.file, "}}")?;
		
		Ok(())
	}

}


fn write_enum_case_type<'model, 'opt, Gen>(gen: &mut Gen, value_type: &LangType<'model>, case_name: &str, wildcard: bool) -> Result<(), GeneratorError> where
	Gen : JavaGenerator<'model, 'opt> + GeneratorWithFile
{
	match value_type {
		LangType::Versioned(name, version, args) => {
			gen.write_qual_name(name)?;
			write!(gen.file(), ".V{}.{}", version, case_name)?;
			if !args.is_empty() {
				write!(gen.file(), "<")?;
				for_sep!(arg, args, { write!(gen.file(), ", ")?; }, {
					if wildcard {
						write!(gen.file(), "?")?;
					}
					else {
						gen.write_type(arg, true)?;
					}
				});
				write!(gen.file(), ">")?;
			}
		},
		_ => Err("Invalid enum type.")?,
	}

	Ok(())
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> JavaTypeGenerator<'model, 'opt, 'output, Output, Extra> where JavaTypeGenerator<'model, 'opt, 'output, Output, Extra> : JavaExtraGeneratorOps {


	fn open(model: &'model model::Verilization, options: &'opt JavaOptions, output: &'output mut Output, type_def: Named<'model, model::TypeDefinitionData>) -> Result<Self, GeneratorError> where Extra : Default {
		let file = open_java_file(options, output, type_def.name())?;
		Ok(JavaTypeGenerator {
			file: file,
			model: model,
			options: options,
			type_def: type_def,
			scope: type_def.scope(),
			versions: HashSet::new(),
			indentation_level: 0,
			_extra: Extra::default(),
		})
	}

	fn write_expr_statement(&mut self, stmt: &LangExprStmt<'model>, is_expr: bool) -> Result<(), GeneratorError> {
		if !is_expr {
			self.write_indent()?;
			write!(self.file, "return ")?;
		}

		match stmt {
			LangExprStmt::Expr(expr) => {
				self.write_expr(expr)?;
				writeln!(self.file, ";")?;
			},

			LangExprStmt::CreateCodec { t, read, write } => {
				write!(self.file, "new {}.Codec<", RUNTIME_PACKAGE)?;
				self.write_type(t, true)?;
				writeln!(self.file, ">() {{")?;
				self.indent_increase();


				self.write_indent()?;
				writeln!(self.file, "@Override")?;

				self.write_indent()?;
				write!(self.file, "public ")?;
				self.write_type(t, true)?;
				writeln!(self.file, " read({}.FormatReader reader) throws java.io.IOException {{", RUNTIME_PACKAGE)?;
				self.indent_increase();
				
				self.write_statement(read)?;

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}}")?;

				self.write_indent()?;
				writeln!(self.file, "@Override")?;

				self.write_indent()?;
				write!(self.file, "public void write({}.FormatWriter writer, ", RUNTIME_PACKAGE)?;
				self.write_type(t, true)?;
				writeln!(self.file, " value) throws java.io.IOException {{")?;
				self.indent_increase();

				self.write_statement(write)?;

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}}")?;

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}};")?;
			},

			LangExprStmt::CreateConverter { from_type, to_type, body } => {
				write!(self.file, "new {}.Converter<", RUNTIME_PACKAGE)?;
				self.write_type(from_type, true)?;
				write!(self.file, ", ")?;
				self.write_type(to_type, true)?;
				writeln!(self.file, ">() {{")?;
				self.indent_increase();


				self.write_indent()?;
				writeln!(self.file, "@Override")?;

				self.write_indent()?;
				write!(self.file, "public ")?;
				self.write_type(to_type, true)?;
				write!(self.file, " convert(")?;
				self.write_type(from_type, true)?;
				writeln!(self.file, " {}) {{", Self::convert_prev_param_name())?;
				self.indent_increase();

				self.write_statement(body)?;

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}}")?;

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}};")?;
			},

		}

		Ok(())
	}
	
	fn write_statement(&mut self, stmt: &LangStmt<'model>) -> Result<(), GeneratorError> {
		match stmt {
			LangStmt::Expr(exprs, result_expr) => {
				for expr in exprs {
					self.write_indent()?;
					self.write_expr(expr)?;
					writeln!(self.file, ";")?;
				}

				if let Some(result_expr) = result_expr {
					self.write_indent()?;
					write!(self.file, "return ")?;
					self.write_expr(result_expr)?;
					writeln!(self.file, ";")?;
				}
			},

			LangStmt::MatchEnum { value, value_type, cases } => {
				self.write_indent()?;
				for MatchCase { binding_name, case_name, body } in cases {
					write!(self.file, "if(")?;
					self.write_expr(value)?;
					write!(self.file, " instanceof ")?;
					write_enum_case_type(self, value_type, case_name, true)?;
					writeln!(self.file, ") {{")?;
					self.indent_increase();

					self.write_indent()?;
					write!(self.file, "var {} = ((", binding_name)?;
					write_enum_case_type(self, value_type, case_name, false)?;
					write!(self.file, ")")?;
					self.write_expr(value)?;
					writeln!(self.file, ").{};", case_name)?;

					self.write_statement(body)?;

					self.indent_decrease();
					self.write_indent()?;
					writeln!(self.file, "}}")?;

					self.write_indent()?;
					write!(self.file, "else ")?;
				}
				if !cases.is_empty() {
					writeln!(self.file, "{{")?;
					self.indent_increase();
					self.write_indent()?;
				}

				writeln!(self.file, "throw new IllegalArgumentException();")?;
				
				if !cases.is_empty() {
					self.indent_decrease();
					self.write_indent()?;
					writeln!(self.file, "}}")?;
				}
			},

			LangStmt::MatchDiscriminator { value, cases } => {
				self.write_indent()?;
				write!(self.file, "switch(")?;
				self.write_expr(value)?;
				writeln!(self.file, ".intValueExact()) {{")?;
				self.indent_increase();

				for (n, body) in cases {
					self.write_indent()?;
					writeln!(self.file, "case {}:", n)?;
					self.write_indent()?;
					writeln!(self.file, "{{")?;
					self.indent_increase();

					self.write_statement(body)?;


					if !body.has_value() {
						self.write_indent()?;
						writeln!(self.file, "break;")?;
					}
					self.indent_decrease();

					self.write_indent()?;
					writeln!(self.file, "}}")?;
				}

				self.write_indent()?;
				writeln!(self.file, "default: throw new java.io.IOException(\"Invalid tag number.\");")?;

				self.indent_decrease();

				self.write_indent()?;
				writeln!(self.file, "}}")?;
			},
			

		}

		Ok(())
	}

	fn write_type_params(&mut self, params: &Vec<String>) -> Result<(), GeneratorError> {
		if !params.is_empty() {
			write!(self.file, "<")?;
			for_sep!(param, params, { write!(self.file, ", ")?; }, {
				write!(self.file, "{}", param)?;
			});
			write!(self.file, ">")?;
		}
	
		Ok(())
	}
	
}

impl <'model, 'opt, 'output, 'state, Output: OutputHandler> JavaExtraGeneratorOps for JavaTypeGenerator<'model, 'opt, 'state, Output, GenStructType> {
	fn version_class_modifier() -> &'static str {
		"final"
	}

	fn write_versioned_type_data(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		self.write_indent()?;
		write!(self.file, "public V{}(", ver_type.version)?;

		for_sep!((field_name, field), &ver_type.ver_type.fields, { write!(self.file, ",")?; }, {
			self.write_type(&self.build_type(&ver_type.version, &field.field_type)?, false)?;
			write!(self.file, " {}", field_name)?;
		});

		writeln!(self.file, ") {{")?;
		self.indent_increase();
		
		for (field_name, _) in &ver_type.ver_type.fields {
			self.write_indent()?;
			writeln!(self.file, "this.{} = {};", field_name, field_name)?;
		}

		self.indent_decrease();
		self.write_indent()?;
		writeln!(self.file, "}}")?;

		for (field_name, field) in &ver_type.ver_type.fields {
			self.write_indent()?;
			write!(self.file, "public final ")?;
			self.write_type(&self.build_type(&ver_type.version, &field.field_type)?, false)?;
			writeln!(self.file, " {};", field_name)?;
		}

		self.write_indent()?;
		writeln!(self.file, "@Override")?;

		self.write_indent()?;
		writeln!(self.file, "public int hashCode() {{")?;
		self.indent_increase();

		self.write_indent()?;
		write!(self.file, "return java.util.Objects.hash(")?;
		for_sep!((field_name, _), &ver_type.ver_type.fields, { write!(self.file, ", ")?; }, {
			write!(self.file, "{}", field_name)?;
		});
		writeln!(self.file, ");")?;
		self.indent_decrease();
		self.write_indent()?;
		writeln!(self.file, "}}")?;

		self.write_indent()?;
		writeln!(self.file, "@Override")?;
		self.write_indent()?;
		writeln!(self.file, "public boolean equals(Object obj) {{")?;
		self.indent_increase();
		self.write_indent()?;
		writeln!(self.file, "if(!(obj instanceof V{})) return false;", ver_type.version)?;
		self.write_indent()?;
		writeln!(self.file, "V{} other = (V{})obj;", ver_type.version, ver_type.version)?;
		for (field_name, _) in &ver_type.ver_type.fields {
			self.write_indent()?;
			writeln!(self.file, "if(!java.util.Objects.deepEquals(this.{}, other.{})) return false;", field_name, field_name)?;
		}
		self.write_indent()?;
		writeln!(self.file, "return true;")?;
		self.indent_decrease();
		self.write_indent()?;
		writeln!(self.file, "}}")?;

		Ok(())
	}
}

impl <'model, 'opt, 'output, 'state, Output: OutputHandler> JavaExtraGeneratorOps for JavaTypeGenerator<'model, 'opt, 'state, Output, GenEnumType> {
	fn version_class_modifier() -> &'static str {
		"abstract"
	}

	fn write_versioned_type_data(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		self.write_indent()?;
		writeln!(self.file, "private V{}() {{}}", ver_type.version)?;

		for (index, (field_name, field)) in ver_type.ver_type.fields.iter().enumerate() {
			self.write_indent()?;
			write!(self.file, "public static final class {}", field_name)?;
			self.write_type_params(self.type_def().type_params())?;
			write!(self.file, " extends V{}", ver_type.version)?;
			self.write_type_params(self.type_def().type_params())?;
			writeln!(self.file, " {{")?;

			self.indent_increase();
			self.write_indent()?;
			write!(self.file, "public {}(", field_name)?;
			self.write_type(&self.build_type(&ver_type.version, &field.field_type)?, false)?;
			writeln!(self.file, " {}) {{", field_name)?;
			self.indent_increase();
			self.write_indent()?;
			writeln!(self.file, "this.{} = {};", field_name, field_name)?;
			self.indent_decrease();
			self.write_indent()?;
			writeln!(self.file, "}}")?;
			self.write_indent()?;
			write!(self.file, "public final ")?;
			self.write_type(&self.build_type(&ver_type.version, &field.field_type)?, false)?;
			writeln!(self.file, " {};", field_name)?;
			
			self.write_indent()?;
			writeln!(self.file, "@Override")?;
			self.write_indent()?;
			writeln!(self.file, "public int hashCode() {{")?;
			self.indent_increase();
			self.write_indent()?;
			writeln!(self.file, "return java.util.Objects.hash({}, this.{});", index, field_name)?;
			self.indent_decrease();
			self.write_indent()?;
			writeln!(self.file, "}}")?;
			
			self.write_indent()?;
			writeln!(self.file, "@Override")?;
			self.write_indent()?;
			writeln!(self.file, "public boolean equals(Object obj) {{")?;
			self.indent_increase();
			self.write_indent()?;
			writeln!(self.file, "if(!(obj instanceof {})) return false;", field_name)?;
			self.write_indent()?;
			writeln!(self.file, "{} other = ({})obj;", field_name, field_name)?;
			self.write_indent()?;
			writeln!(self.file, "return java.util.Objects.deepEquals(this.{}, other.{});", field_name, field_name)?;
			self.indent_decrease();
			self.write_indent()?;
			writeln!(self.file, "}}")?;
	
			self.indent_decrease();
			self.write_indent()?;
			writeln!(self.file, "}}")?;
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
			let mut const_gen = JavaConstGenerator::open(model, &options, output, constant)?;
			const_gen.generate()?;
		}

		for t in model.types() {
			match t {
				model::NamedTypeDefinition::StructType(t) => {
					let mut type_gen: JavaTypeGenerator<_, GenStructType> = JavaTypeGenerator::open(model, &options, output, t)?;
					type_gen.generate()?;		
				},
				model::NamedTypeDefinition::EnumType(t) => {
					let mut type_gen: JavaTypeGenerator<_, GenEnumType> = JavaTypeGenerator::open(model, &options, output, t)?;
					type_gen.generate()?;		
				},
			}
		}

		Ok(())
	}

}
