use verilization_compiler::{model, lang, util, for_sep};

use model::Named;
use crate::lang::{GeneratorError, Language, LanguageOptions, LanguageOptionsBuilder, OutputHandler};
use std::ffi::OsString;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use num_bigint::BigUint;
use lang::generator::*;
use util::{capitalize_identifier, uncapitalize_identifier};
use num_traits::ToPrimitive;

type PackageMap = HashMap<model::PackageName, model::PackageName>;
type ExternMap = HashMap<model::QualifiedName, model::QualifiedName>;
const RUNTIME_PACKAGE: &str = "dev.argon.verilization.runtime";


pub struct JavaOptionsBuilder {
	output_dir: Option<OsString>,
	package_mapping: PackageMap,
	library_mapping: PackageMap,
	extern_mapping: ExternMap,
}

pub struct JavaOptions {
	pub output_dir: OsString,
	pub package_mapping: PackageMap,
	pub library_mapping: PackageMap,
	pub extern_mapping: ExternMap,
}

fn make_type_name(name: &str) -> String {
	let mut name = String::from(name);
	capitalize_identifier(&mut name);
	name
}

fn make_field_name(field_name: &str) -> String {
	let mut name = String::from(field_name);
	uncapitalize_identifier(&mut name);
	name
}


fn java_package_impl<'a>(options: &'a JavaOptions, package: &model::PackageName) -> Result<&'a model::PackageName, GeneratorError> {
	options.package_mapping.get(&package)
		.or_else(|| options.library_mapping.get(&package))
		.ok_or_else(|| GeneratorError::UnmappedPackage(package.clone()))
}

fn open_java_file<'a, Output: OutputHandler<'a>>(options: &JavaOptions, output: &'a mut Output, name: &model::QualifiedName) -> Result<Output::FileHandle, GeneratorError> {
	let java_pkg = java_package_impl(options, &name.package)?;
	let mut path = PathBuf::from(&options.output_dir);
	for part in &java_pkg.package {
		path.push(part);
	}
	
	path.push(make_type_name(&name.name) + ".java");
	Ok(output.create_file(path)?)
}


fn write_operation_target<'a, Gen : JavaGenerator<'a>>(gen: &mut Gen, target: &OperationTarget) -> Result<(), GeneratorError> {
	match target {
		OperationTarget::VersionedType(name, version) | OperationTarget::InterfaceType(name, version) => {
			gen.write_qual_name(name)?;
			write!(gen.file(), ".V{}", version)?;
		},
		OperationTarget::ExternType(name) => {
			gen.write_qual_name(name)?;
		},
	}

	Ok(())
}


#[derive(Copy, Clone)]
enum ResultHandling {
	Return,
	Yield,
}

pub trait JavaGenerator<'a> : Generator<'a> + GeneratorWithFile {
	fn options(&self) -> &'a JavaOptions;

	fn java_package(&self, package: &model::PackageName) -> Result<&'a model::PackageName, GeneratorError> {
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
	
		write!(self.file(), "{}", make_type_name(&name.name))?;
	
		Ok(())
	}
	
	fn write_type_args(&mut self, args: &Vec<LangType<'a>>) -> Result<(), GeneratorError> {
		if !args.is_empty() {
			write!(self.file(), "<")?;
			for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
				self.write_type(arg, true)?;
			});
			write!(self.file(), ">")?;

		}
	
		Ok(())
	}
	
	fn extern_type_name(&self, name: &model::QualifiedName, erased: bool) -> Result<model::QualifiedName, GeneratorError> {
		Ok(if let Some(mapped_name) = self.options().extern_mapping.get(name) {
			if erased {
				match (&mapped_name.package.package[..], mapped_name.name.as_ref()) {
					([], "byte") => model::QualifiedName::from_parts(&["java", "lang"], "Byte"),
					([], "short") => model::QualifiedName::from_parts(&["java", "lang"], "Short"),
					([], "int") => model::QualifiedName::from_parts(&["java", "lang"], "Integer"),
					([], "long") => model::QualifiedName::from_parts(&["java", "lang"], "Long"),
					([], "float") => model::QualifiedName::from_parts(&["java", "lang"], "Float"),
					([], "double") => model::QualifiedName::from_parts(&["java", "lang"], "Double"),
					([], "boolean") => model::QualifiedName::from_parts(&["java", "lang"], "Boolean"),
					([], "char") => model::QualifiedName::from_parts(&["java", "lang"], "Character"),
					_ => mapped_name.clone(),
				}
			}
			else {
				mapped_name.clone()
			}
		}
		else {
			model::QualifiedName {
				package: self.java_package(&name.package)?.clone(),
				name: make_type_name(&name.name),
			}
		})
	}
	
	fn write_type(&mut self, t: &LangType<'a>, erased: bool) -> Result<(), GeneratorError> {
		Ok(match t {
			LangType::Versioned(_, name, version, args, _) | LangType::Interface(name, version, args, _) => {
				self.write_qual_name(name)?;
				write!(self.file(), ".V{}", version)?;
				self.write_type_args(args)?;
			},

			LangType::Extern(name, args, _) => {
				let mapped_name = self.extern_type_name(name, erased)?;

				for part in &mapped_name.package.package {
					write!(self.file(), "{}.", part)?;
				}
			
				write!(self.file(), "{}", mapped_name.name)?;


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

			LangType::RemoteObjectId => write!(self.file(), "{}.RemoteObjectId", RUNTIME_PACKAGE)?,
			LangType::RemoteConnection => write!(self.file(), "{}.RemoteConnection", RUNTIME_PACKAGE)?,
		})
	}

	fn write_args(&mut self, args: &Vec<LangExpr<'a>>) -> Result<(), GeneratorError> {
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
			Operation::TypeCodec => write!(self.file(), "codec")?,
			Operation::FromInteger => write!(self.file(), "fromInteger")?,
			Operation::FromString => write!(self.file(), "fromString")?,
			Operation::FromSequence => write!(self.file(), "fromSequence")?,
			Operation::FromRecord(_) => write!(self.file(), "fromRecord")?,
			Operation::FromCase(name) => write!(self.file(), "fromCase{}", make_type_name(name))?,
			Operation::CreateRemoteWrapper => write!(self.file(), "createRemoteWrapper")?,
		}

		Ok(())
	}
	
	fn write_expr(&mut self, expr: &LangExpr<'a>) -> Result<(), GeneratorError> {
		match expr {
			LangExpr::Identifier(name) => write!(self.file(), "{}", name)?,
			LangExpr::IntegerLiteral(n) => {
				if let Some(n) = n.to_i32() {
					write!(self.file(), "{}", n)?;
				}
				else if let Some(n) = n.to_i64() {
					write!(self.file(), "{}L", n)?;
				}
				else {
					write!(self.file(), "java.math.BigInteger.valueOf(\"{}\")", n)?;
				}
			},
			LangExpr::StringLiteral(s) => {
				write!(self.file(), "\"")?;
				for codepoint in s.chars() {
					match codepoint {
						'"' => write!(self.file(), "\\\"")?,
						'\\' => write!(self.file(), "\\\\")?,
						'\n' => write!(self.file(), "\\n")?,
						'\r' => write!(self.file(), "\\r")?,
						_ => write!(self.file(), "{}", codepoint)?,
					}
				}
				write!(self.file(), "\"")?;
			},
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
			LangExpr::ReadDiscriminator => write!(self.file(), "{}.Nat.codec.read(reader)", RUNTIME_PACKAGE)?,
			LangExpr::WriteDiscriminator(value) => write!(self.file(), "{}.Nat.codec.write(writer, java.math.BigInteger.valueOf({}))", RUNTIME_PACKAGE, value)?,
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
			LangExpr::InvokeOperation(op, target, type_args, args) => {
				write_operation_target(self, target)?;
				write!(self.file(), ".")?;
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
				write!(self.file(), ".{}", JavaLanguage::constant_version_name(version))?;
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
				write!(self.file(), ".V{}.{}", version, make_type_name(field_name))?;
				self.write_type_args(type_args)?;
				write!(self.file(), "(")?;
				self.write_expr(value)?;
				write!(self.file(), ")")?;
			},
			LangExpr::StructField(_, _, field_name, value) => {
				self.write_expr(value)?;
				write!(self.file(), ".{}", make_field_name(field_name))?;
			},
			LangExpr::ReadRemoteObject { object_type_target, connection } => {
				self.write_expr(connection)?;
				write!(self.file(), ".readObject(")?;
				write_operation_target(self, object_type_target)?;
				write!(self.file(), ".createRemoteWrapper(")?;
				self.write_expr(connection)?;
				write!(self.file(), "))")?;
			},
			LangExpr::WriteRemoteObject { object, connection } => {
				self.write_expr(connection)?;
				write!(self.file(), ".writeObject(")?;
				self.write_expr(object)?;
				write!(self.file(), ")")?;
			},
		}

		Ok(())
	}
}

impl GeneratorNameMapping for JavaLanguage {
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

	fn format_writer_name() -> &'static str {
		"writer"
	}

	fn format_reader_name() -> &'static str {
		"reader"
	}

	fn connection_name() -> &'static str {
		"connection"
	}

	fn object_id_name() -> &'static str {
		"objectId"
	}

	fn codec_codec_param_name(param: &str) -> String {
		format!("{}_codec", param)
	}

	fn constant_version_name(version: &BigUint) -> String {
		format!("v{}", version)
	}
}


struct JavaConstGenerator<'a, Output: OutputHandler<'a>> {
	file: Output::FileHandle,
	model: &'a model::Verilization,
	options: &'a JavaOptions,
	constant: Named<'a, model::Constant>,
	scope: model::Scope<'a>,
}

impl <'a, Output: OutputHandler<'a>> Generator<'a> for JavaConstGenerator<'a, Output> {
	type Lang = JavaLanguage;

	fn model(&self) -> &'a model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'a> {
		&self.scope
	}
}

impl <'a, Output: OutputHandler<'a>> GeneratorWithFile for JavaConstGenerator<'a, Output> {
	type GeneratorFile = Output::FileHandle;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'a, Output: OutputHandler<'a>> JavaGenerator<'a> for JavaConstGenerator<'a, Output> {
	fn options(&self) -> &'a JavaOptions {
		self.options
	}
}

impl <'a, Output: OutputHandler<'a>> ConstGenerator<'a> for JavaConstGenerator<'a, Output> {
	fn constant(&self) -> Named<'a, model::Constant> {
		self.constant
	}

	fn write_header(&mut self) -> Result<(), GeneratorError> {
        self.write_package(&self.constant.name().package)?;

		writeln!(self.file, "public final class {} {{", make_type_name(&self.constant.name().name))?;
		writeln!(self.file, "\tprivate {}() {{}}", make_type_name(&self.constant.name().name))?;

		Ok(())
	}

	fn write_constant(&mut self, version_name: String, t: LangType<'a>, value: LangExpr<'a>) -> Result<(), GeneratorError> {
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

impl <'a, Output: OutputHandler<'a>> JavaConstGenerator<'a, Output> {

	fn open(model: &'a model::Verilization, options: &'a JavaOptions, output: &'a mut Output, constant: Named<'a, model::Constant>) -> Result<Self, GeneratorError> {
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

struct JavaTypeGenerator<'a, Output: OutputHandler<'a>, TypeDef> {
	file: Output::FileHandle,
	model: &'a model::Verilization,
	options: &'a JavaOptions,
	type_def: Named<'a, TypeDef>,
	scope: model::Scope<'a>,
	indentation_level: u32,
}

impl <'a, Output: OutputHandler<'a>, TypeDef> Generator<'a> for JavaTypeGenerator<'a, Output, TypeDef> {
	type Lang = JavaLanguage;

	fn model(&self) -> &'a model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'a> {
		&self.scope
	}
}

impl <'a, Output: OutputHandler<'a>, TypeDef> GeneratorWithFile for JavaTypeGenerator<'a, Output, TypeDef> {
	type GeneratorFile = Output::FileHandle;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'a, Output: OutputHandler<'a>, TypeDef> Indentation for JavaTypeGenerator<'a, Output, TypeDef> {
	fn indentation_size(&mut self) -> &mut u32 {
		&mut self.indentation_level
	}
}

impl <'a, Output: OutputHandler<'a>, TypeDef> JavaGenerator<'a> for JavaTypeGenerator<'a, Output, TypeDef> {
	fn options(&self) -> &'a JavaOptions {
		self.options
	}
}

impl <'a, Output: OutputHandler<'a>, TypeDef: 'a + model::GeneratableType<'a>> TypeGenerator<'a> for JavaTypeGenerator<'a, Output, TypeDef> {
	type TypeDefinition = TypeDef;

	fn type_def(&self) -> Named<'a, TypeDef> {
		self.type_def
	}

	fn write_header(&mut self) -> Result<(), GeneratorError> {
		self.write_package(&self.type_def.name().package)?;
		writeln!(self.file, "public abstract class {} {{", make_type_name(&self.type_def.name().name))?;
		self.indent_increase();
		self.write_indent()?;
		writeln!(self.file, "private {}() {{}}", make_type_name(&self.type_def.name().name))?;
		
		Ok(())
	}

	fn write_version_header(&mut self, t: LangType<'a>) -> Result<(), GeneratorError> {
		match t {
			LangType::Versioned(VersionedTypeKind::Struct, _, version, _, fields) => {
				self.write_indent()?;
				write!(self.file, "public static record V{}", version)?;
				self.write_type_params(self.type_def().type_params())?;
				write!(self.file, "(")?;

				let fields = fields.build()?;

				for_sep!(field, &fields, { write!(self.file, ",")?; }, {
					self.write_type(&field.field_type, false)?;
					write!(self.file, " {}", make_field_name(field.name))?;
				});

				writeln!(self.file, ") {{")?;
				self.indent_increase();
			},
			LangType::Versioned(VersionedTypeKind::Enum, _, version, _, fields) => {
				self.write_indent()?;
				write!(self.file, "public static sealed interface V{}", version)?;
				self.write_type_params(self.type_def().type_params())?;
				writeln!(self.file, " {{")?;

				self.indent_increase();

				let fields = fields.build()?;
		
				for field in fields {
					self.write_indent()?;
					write!(self.file, "public static record {}", make_type_name(field.name))?;
					self.write_type_params(self.type_def().type_params())?;
					write!(self.file, "(")?;
					self.write_type(&field.field_type, false)?;
					write!(self.file, " {}) implements V{}", make_field_name(field.name), version)?;
					self.write_type_params(self.type_def().type_params())?;
					writeln!(self.file, " {{}}")?;
				}
			},
			LangType::Interface(_, version, _, methods) => {
				self.write_indent()?;
				write!(self.file, "public static interface V{}", version)?;
				self.write_type_params(self.type_def().type_params())?;
				writeln!(self.file, " {{")?;

				self.indent_increase();

				let methods = methods.build()?;

				for method in methods {
					self.write_indent()?;
					write!(self.file, "public ")?;
					self.write_type_params(&method.type_params)?;
					if !method.type_params.is_empty() {
						write!(self.file, " ")?;
					}
					self.write_type(&method.return_type, false)?;
					write!(self.file, " {}", make_field_name(method.name))?;
					write!(self.file, "(")?;
					for_sep!(type_param, method.type_params, { write!(self.file, ", ")? }, {
						write!(self.file, "{}.Codec<{}> {}_codec", RUNTIME_PACKAGE, type_param, type_param)?;
					});
					if !method.type_params.is_empty() && !method.parameters.is_empty() {
						write!(self.file, ", ")?;
					}
					for_sep!(param, method.parameters, { write!(self.file, ", ")? }, {
						self.write_type(&param.param_type, false)?;
						write!(self.file, " {}", param.name)?;
					});
					writeln!(self.file, ") throws java.io.IOException;")?;
				}
			},
			_ => return Err(GeneratorError::CouldNotGenerateType)
		}

		Ok(())
	}

	fn write_operation(&mut self, operation: OperationInfo<'a>) -> Result<(), GeneratorError> {
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

	fn write_version_footer(&mut self) -> Result<(), GeneratorError> {
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


fn write_enum_case_type<'a, Gen>(gen: &mut Gen, value_type: &LangType<'a>, case_name: &str) -> Result<(), GeneratorError> where
	Gen : JavaGenerator<'a> + GeneratorWithFile
{
	match value_type {
		LangType::Versioned(VersionedTypeKind::Enum, name, version, args, _) => {
			gen.write_qual_name(name)?;
			write!(gen.file(), ".V{}.{}", version, make_type_name(case_name))?;
			if !args.is_empty() {
				write!(gen.file(), "<")?;
				for_sep!(arg, args, { write!(gen.file(), ", ")?; }, {
					gen.write_type(arg, true)?;
				});
				write!(gen.file(), ">")?;
			}
		},
		_ => panic!("Invalid enum type."),
	}

	Ok(())
}

impl <'a, Output: OutputHandler<'a>, TypeDef: model::GeneratableType<'a>> JavaTypeGenerator<'a, Output, TypeDef> {


	fn open(model: &'a model::Verilization, options: &'a JavaOptions, output: &'a mut Output, type_def: Named<'a, TypeDef>) -> Result<Self, GeneratorError> {
		let file = open_java_file(options, output, type_def.name())?;
		Ok(JavaTypeGenerator {
			file: file,
			model: model,
			options: options,
			type_def: type_def,
			scope: type_def.scope(),
			indentation_level: 0,
		})
	}

	fn write_expr_statement(&mut self, stmt: &LangExprStmt<'a>, is_expr: bool) -> Result<(), GeneratorError> {
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
				
				self.write_statement(read, ResultHandling::Return)?;

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

				self.write_statement(write, ResultHandling::Return)?;

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
				writeln!(self.file, " {}) {{", JavaLanguage::convert_prev_param_name())?;
				self.indent_increase();

				self.write_statement(body, ResultHandling::Return)?;

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}}")?;

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}};")?;
			},

			LangExprStmt::CreateRemoteWrapper { t, connection, id, methods } => {
				writeln!(self.file, "switch(0) {{")?;
				self.indent_increase();

				self.write_indent()?;
				writeln!(self.file, "default -> {{")?;
				self.indent_increase();

				self.write_indent()?;
				write!(self.file, "final class RemoteWrapper extends {}.RemoteObject implements ", RUNTIME_PACKAGE)?;
				self.write_type(t, false)?;
				writeln!(self.file, " {{")?;
				self.indent_increase();

				self.write_indent()?;
				writeln!(self.file, "public RemoteWrapper({}.RemoteConnection connection, {}.RemoteObjectId id) {{", RUNTIME_PACKAGE, RUNTIME_PACKAGE)?;
				self.indent_increase();

				self.write_indent()?;
				writeln!(self.file, "super(connection, id);")?;

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}}")?;


				for method in methods {
					self.write_indent()?;
					writeln!(self.file, "@Override")?;

					self.write_indent()?;
					write!(self.file, "public ")?;
					self.write_type_params(&method.type_params)?;
					if !method.type_params.is_empty() {
						write!(self.file, " ")?;
					}

					self.write_type(&method.return_type, false)?;

					write!(self.file, " {}(", method.name)?;
					for_sep!(type_param, method.type_params, { write!(self.file, ", ")? }, {
						write!(self.file, "{}.Codec<{}> {}_codec", RUNTIME_PACKAGE, type_param, type_param)?;
					});
					if !method.type_params.is_empty() && !method.parameters.is_empty() {
						write!(self.file, ", ")?;
					}
					for_sep!(param, &method.parameters, { write!(self.file, ", ")? }, {
						self.write_type(&param.param_type, false)?;
						write!(self.file, " {}", param.name)?;
					});
					writeln!(self.file, ") throws java.io.IOException {{")?;
					self.indent_increase();
					
					self.write_indent()?;
					write!(self.file(), "return this.connection.invokeMethod(this.id, \"{}\", new {}.RemoteConnection.MethodArgument<?>[] {{", method.name, RUNTIME_PACKAGE)?;
					for_sep!(param, &method.parameters, { write!(self.file(), ", ")?; }, {
						write!(self.file(), "new {}.RemoteConnection.MethodArgument<", RUNTIME_PACKAGE)?;
						self.write_type(&param.param_type, true)?;
						write!(self.file(), ">({}, ", param.name)?;
						self.write_expr(&self.build_codec(param.param_type.clone())?)?;
						write!(self.file(), ")")?;
					});
					write!(self.file(), "}}, ")?;
					self.write_expr(&self.build_codec(method.return_type.clone())?)?;
					writeln!(self.file(), ");")?;

					self.indent_decrease();
					self.write_indent()?;
					writeln!(self.file, "}}")?;
				}

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}}")?;

				self.write_indent()?;
				write!(self.file, "yield new RemoteWrapper(")?;
				self.write_expr(&connection)?;
				write!(self.file, ", ")?;
				self.write_expr(&id)?;
				writeln!(self.file, ");")?;

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
	
	fn get_result_handling_kw(result_handling: ResultHandling) -> &'static str {
		match result_handling {
			ResultHandling::Return => "return",
			ResultHandling::Yield => "yield",
		}
	}

	fn write_statement(&mut self, stmt: &LangStmt<'a>, result_handling: ResultHandling) -> Result<(), GeneratorError> {
		match stmt {
			LangStmt::Expr(exprs, result_expr) => {
				for expr in exprs {
					self.write_indent()?;
					self.write_expr(expr)?;
					writeln!(self.file, ";")?;
				}

				if let Some(result_expr) = result_expr {
					self.write_indent()?;
					write!(self.file, "{} ", Self::get_result_handling_kw(result_handling))?;
					self.write_expr(result_expr)?;
					writeln!(self.file, ";")?;
				}
			},

			LangStmt::MatchEnum { value, value_type, cases } => {
				self.write_indent()?;
				if stmt.has_value() {
					write!(self.file, "{} ", Self::get_result_handling_kw(result_handling))?
				}
				
				write!(self.file, "switch(")?;
				self.write_expr(value)?;
				writeln!(self.file, ") {{")?;
				self.indent_increase();

				for MatchCase { binding_name, case_name, body } in cases {
					self.write_indent()?;
					write!(self.file, "case ")?;
					write_enum_case_type(self, value_type, case_name)?;
					write!(self.file, " case_{}", binding_name)?;

					if body.has_value() {
						write!(self.file, " -> ")?;
					}
					else {
						writeln!(self.file, ":")?;
						self.write_indent()?;
					}
					writeln!(self.file, "{{")?;

					self.indent_increase();

					self.write_indent()?;
					writeln!(self.file, "var {} = case_{}.{}();", binding_name, binding_name, make_field_name(case_name))?;
					self.write_statement(body, ResultHandling::Yield)?;

					if !body.has_value() {
						self.write_indent()?;
						writeln!(self.file, "break;")?;
					}

					self.indent_decrease();

					self.write_indent()?;
					writeln!(self.file, "}}")?;
				}

				self.indent_decrease();
				self.write_indent()?;
				if stmt.has_value() {
					writeln!(self.file, "}};")?;
				}
				else {
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

					self.write_statement(body, result_handling)?;


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


pub struct JavaLanguage {}


impl Language for JavaLanguage {
	type Options = JavaOptions;

    fn name() -> &'static str {
        "java"
    }

	fn generate<Output : for<'output> OutputHandler<'output>>(model: &model::Verilization, options: Self::Options, output: &mut Output) -> Result<(), GeneratorError> {
		let mut codegen = JavaCodeGenerator {
			model,
			options: &options,
			output,
		};
		codegen.generate(model)
	}

}

impl LanguageOptions for JavaOptions {
	type Builder = JavaOptionsBuilder;

	fn build(builder: Self::Builder) -> Result<Self, GeneratorError> {
		Ok(JavaOptions {
			output_dir: builder.output_dir.ok_or_else(|| GeneratorError::InvalidOptions(String::from("Output directory not specified")))?,
			package_mapping: builder.package_mapping,
			library_mapping: builder.library_mapping,
			extern_mapping: builder.extern_mapping,
		})
	}
}

impl LanguageOptionsBuilder for JavaOptionsBuilder {
	fn empty() -> JavaOptionsBuilder {
		JavaOptionsBuilder {
			output_dir: None,
			package_mapping: HashMap::new(),
			library_mapping: HashMap::new(),
			extern_mapping: HashMap::new(),
		}
	}

	fn add(&mut self, name: &str, value: OsString) -> Result<(), GeneratorError> {
		if name == "out_dir" {
			if self.output_dir.is_some() {
				return Err(GeneratorError::InvalidOptions(String::from("Output directory already specified")))
			}

			self.output_dir = Some(value);
			Ok(())
		}
		else if let Some(pkg) = name.strip_prefix("pkg:") {
			let package = model::PackageName::from_str(pkg);

            let java_package = model::PackageName::from_str(value.to_str().unwrap());

			if self.library_mapping.contains_key(&package) || self.package_mapping.insert(package, java_package).is_some() {
				return Err(GeneratorError::InvalidOptions(format!("Package already mapped: {}", pkg)))
			}
			Ok(())
		}
		else if let Some(pkg) = name.strip_prefix("lib:") {
			let package = model::PackageName::from_str(pkg);

            let java_package = model::PackageName::from_str(value.to_str().unwrap());

			if self.package_mapping.contains_key(&package) || self.library_mapping.insert(package, java_package).is_some() {
				return Err(GeneratorError::InvalidOptions(format!("Package already mapped: {}", pkg)))
			}
			Ok(())
		}
		else if let Some(extern_name) = name.strip_prefix("extern:") {
			let qual_name = model::QualifiedName::from_str(extern_name).ok_or_else(|| GeneratorError::InvalidOptions(format!("Invalid extern type name: {}", extern_name)))?;

			let java_name = model::QualifiedName::from_str(value.to_str().unwrap()).ok_or_else(|| GeneratorError::InvalidOptions(format!("Invalid Java type name: {}", value.to_str().unwrap())))?;

			if self.extern_mapping.insert(qual_name, java_name).is_some() {
				return Err(GeneratorError::InvalidOptions(format!("Extern type already mapped: {}", extern_name)))
			}

			Ok(())
		}
		else {
			Err(GeneratorError::InvalidOptions(format!("Unknown option: {}", name)))
		}
	}
}

struct JavaCodeGenerator<'a, Output> {
	model: &'a model::Verilization,
	options: &'a JavaOptions,
	output: &'a mut Output,
}

impl <'a, 'b, Output : OutputHandler<'a>> GeneratorFactory<'a> for JavaCodeGenerator<'b, Output> {
	type ConstGen = JavaConstGenerator<'a, Output>;
	type VersionedTypeGen = JavaTypeGenerator<'a, Output, model::VersionedTypeDefinitionData>;
	type InterfaceTypeGen = JavaTypeGenerator<'a, Output, model::InterfaceTypeDefinitionData>;

	fn create_constant_generator(&'a mut self, constant: Named<'a, model::Constant>) -> Result<Self::ConstGen, GeneratorError> {
		JavaConstGenerator::open(self.model, self.options, self.output, constant)
	}

	fn create_versioned_type_generator(&'a mut self, t: Named<'a, model::VersionedTypeDefinitionData>) -> Result<Self::VersionedTypeGen, GeneratorError> {
		JavaTypeGenerator::open(self.model, self.options, self.output, t)
	}

	fn create_interface_type_generator(&'a mut self, t: Named<'a, model::InterfaceTypeDefinitionData>) -> Result<Self::InterfaceTypeGen, GeneratorError> {
		JavaTypeGenerator::open(self.model, self.options, self.output, t)
	}
}

