use verilization_compiler::{lang, model, util, for_sep};

use model::Named;
use lang::{GeneratorError, Language, LanguageOptions, LanguageOptionsBuilder, OutputHandler};
use std::ffi::OsString;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;
use num_bigint::BigUint;
use lang::generator::*;
use util::{capitalize_identifier, uncapitalize_identifier};
use num_traits::ToPrimitive;

type PackageMap = HashMap<model::PackageName, model::PackageName>;
const RUNTIME_PACKAGE: &str = "dev.argon.verilization.scala_runtime";


pub struct ScalaOptionsBuilder {
	output_dir: Option<OsString>,
	package_mapping: PackageMap,
	library_mapping: PackageMap,
}

pub struct ScalaOptions {
	pub output_dir: OsString,
	pub package_mapping: PackageMap,
	pub library_mapping: PackageMap,
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


fn scala_package_impl<'a>(options: &'a ScalaOptions, package: &model::PackageName) -> Result<&'a model::PackageName, GeneratorError> {
	options.package_mapping.get(&package)
		.or_else(|| options.library_mapping.get(&package))
		.ok_or_else(|| GeneratorError::UnmappedPackage(package.clone()))
}





fn open_scala_file<'a, Output: OutputHandler<'a>>(options: &ScalaOptions, output: &'a mut Output, name: &model::QualifiedName) -> Result<Output::FileHandle, GeneratorError> {
	let java_pkg = scala_package_impl(options, &name.package)?;
	let mut path = PathBuf::from(&options.output_dir);
    for part in &java_pkg.package {
        path.push(part);
    }
	
	path.push(name.name.clone() + ".scala");
	Ok(output.create_file(path)?)
}


fn write_operation_target<'a, Gen : ScalaGenerator<'a>>(gen: &mut Gen, target: &OperationTarget) -> Result<(), GeneratorError> {
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


pub trait ScalaGenerator<'a> : Generator<'a> + GeneratorWithFile {
	fn options(&self) -> &'a ScalaOptions;


	fn scala_package(&self, package: &model::PackageName) -> Result<&'a model::PackageName, GeneratorError> {
		scala_package_impl(&self.options(), package)
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
	
		write!(self.file(), "{}", make_type_name(&name.name))?;
	
		Ok(())
	}

	fn write_type_args(&mut self, args: &Vec<LangType<'a>>) -> Result<(), GeneratorError> {
		if !args.is_empty() {
			write!(self.file(), "[")?;
			for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
				self.write_type(arg)?;
			});
			write!(self.file(), "]")?;
		}
	
		Ok(())
	}
	
	
	fn write_type(&mut self, t: &LangType<'a>) -> Result<(), GeneratorError> {
		Ok(match t {
			LangType::Versioned(_, name, version, args, _) => {
				self.write_qual_name(&name)?;
				write!(self.file(), ".V{}", version)?;
				self.write_type_args(args)?;
			},
			
			LangType::Interface(name, version, args, _) => {
				self.write_qual_name(&name)?;
				write!(self.file(), ".V{}[R, E", version)?;
				for t in args {
					write!(self.file(), ", ")?;
					self.write_type(t)?;
				}
				write!(self.file(), "]")?;
			},

			LangType::Extern(name, args, _) => {
				self.write_qual_name(name)?;
				self.write_type_args(args)?;
			},

			LangType::TypeParameter(name) => {
				write!(self.file(), "{}", name)?;
			},

			LangType::Converter(from, to) => {
				write!(self.file(), "{}.Converter[", RUNTIME_PACKAGE)?;
				self.write_type(&*from)?;
				write!(self.file(), ", ")?;
				self.write_type(&*to)?;
				write!(self.file(), "]")?;
			},

			LangType::Codec(t) => {
				write!(self.file(), "{}.Codec[", RUNTIME_PACKAGE)?;
				self.write_type(&*t)?;
				write!(self.file(), "]")?;
			},

			LangType::RemoteObjectId => write!(self.file(), "{}.RemoteObjectId", RUNTIME_PACKAGE)?,
			LangType::RemoteConnection => write!(self.file(), "{}.RemoteConnection[R, E]", RUNTIME_PACKAGE)?,
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
					write!(self.file(), "scala.math.BigInt(\"{}\")", n)?;
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
				write!(self.file(), "{}.Converter.identity[", RUNTIME_PACKAGE)?;
				self.write_type(t)?;
				write!(self.file(), "]")?;
			},
			LangExpr::ReadDiscriminator => write!(self.file(), "{}.Nat.codec.read({})", RUNTIME_PACKAGE, ScalaLanguage::format_reader_name())?,
			LangExpr::WriteDiscriminator(value) => write!(self.file(), "{}.Nat.codec.write({}, {})", RUNTIME_PACKAGE, ScalaLanguage::format_writer_name(), value)?,
			LangExpr::CodecRead { codec } => {
				self.write_expr(&*codec)?;
				write!(self.file(), ".read({})", ScalaLanguage::format_reader_name())?;
			},
			LangExpr::CodecWrite { codec, value } => {
				self.write_expr(&*codec)?;
				write!(self.file(), ".write({}, ", ScalaLanguage::format_writer_name())?;
				self.write_expr(value)?;
				write!(self.file(), ")")?;
			},
			LangExpr::InvokeOperation(op, target, type_args, args) => {
				write_operation_target(self, target)?;
				write!(self.file(), ".")?;
				self.write_operation_name(op)?;
				self.write_type_args(type_args)?;
				match op {
					Operation::FromRecord(field_names) => {
						write!(self.file(), "(")?;
						for_sep!((field_name, arg), field_names.iter().zip(args.iter()), { write!(self.file(), ", ")?; }, {
							write!(self.file(), "{} = ", make_field_name(field_name))?;
							self.write_expr(arg)?;
						});
						write!(self.file(), ")")?;
					},
					_ => self.write_args(args)?,
				}
			},
			LangExpr::InvokeUserConverter { name, prev_ver, version, type_args, args } => {
				self.write_qual_name(name)?;
				write!(self.file(), "_Conversions.v{}ToV{}", prev_ver, version)?;
				self.write_type_args(type_args)?;
				self.write_args(args)?;
			},
			LangExpr::ConstantValue(name, version) => {
				self.write_qual_name(name)?;
				write!(self.file(), ".{}", ScalaLanguage::constant_version_name(version))?;
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

impl GeneratorNameMapping for ScalaLanguage {
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

struct ScalaConstGenerator<'a, Output: OutputHandler<'a>> {
	file: Output::FileHandle,
	model: &'a model::Verilization,
	options: &'a ScalaOptions,
	constant: Named<'a, model::Constant>,
	scope: model::Scope<'a>,
}

impl <'a, Output: OutputHandler<'a>> Generator<'a> for ScalaConstGenerator<'a, Output> {
	type Lang = ScalaLanguage;
	
	fn model(&self) -> &'a model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'a> {
		&self.scope
	}
}

impl <'a, Output: OutputHandler<'a>> GeneratorWithFile for ScalaConstGenerator<'a, Output> {
	type GeneratorFile = Output::FileHandle;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'a, Output: OutputHandler<'a>> ScalaGenerator<'a> for ScalaConstGenerator<'a, Output> {
	fn options(&self) -> &'a ScalaOptions {
		self.options
	}
}

impl <'a, Output: OutputHandler<'a>> ConstGenerator<'a> for ScalaConstGenerator<'a, Output> {
	fn constant(&self) -> Named<'a, model::Constant> {
		self.constant
	}

	fn write_header(&mut self) -> Result<(), GeneratorError> {
        self.write_package(&self.constant.name().package)?;

		writeln!(self.file, "object {} {{", make_type_name(&self.constant.name().name))?;

		Ok(())
	}

	fn write_constant(&mut self, version_name: String, t: LangType<'a>, value: LangExpr<'a>) -> Result<(), GeneratorError> {
		write!(self.file, "\tval {}: ", version_name)?;
		self.write_type(&t)?;
		write!(self.file, " = ")?;
		self.write_expr(&value)?;
		writeln!(self.file)?;

		Ok(())
	}

	fn write_footer(&mut self) -> Result<(), GeneratorError> {
		writeln!(self.file, "}}")?;
		Ok(())
	}
}


impl <'a, Output: OutputHandler<'a>> ScalaConstGenerator<'a, Output> {

	fn open(model: &'a model::Verilization, options: &'a ScalaOptions, output: &'a mut Output, constant: Named<'a, model::Constant>) -> Result<Self, GeneratorError> {
		let file = open_scala_file(options, output, constant.name())?;
		Ok(ScalaConstGenerator {
			file: file,
			model: model,
			options: options,
			constant: constant,
			scope: constant.scope(),
		})
	}

}

struct ScalaTypeGenerator<'a, Output: OutputHandler<'a>, TypeDef> {
	options: &'a ScalaOptions,
	model: &'a model::Verilization,
	file: Output::FileHandle,
	type_def: Named<'a, TypeDef>,
	scope: model::Scope<'a>,
	indentation_level: u32,
}

impl <'a, Output: OutputHandler<'a>, TypeDef> Generator<'a> for ScalaTypeGenerator<'a, Output, TypeDef> {
	type Lang = ScalaLanguage;

	fn model(&self) -> &'a model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'a> {
		&self.scope
	}
}

impl <'a, Output: OutputHandler<'a>, TypeDef> GeneratorWithFile for ScalaTypeGenerator<'a, Output, TypeDef> {
	type GeneratorFile = Output::FileHandle;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'a, Output: OutputHandler<'a>, TypeDef> Indentation for ScalaTypeGenerator<'a, Output, TypeDef> {
	fn indentation_size(&mut self) -> &mut u32 {
		&mut self.indentation_level
	}
}

impl <'a, Output: OutputHandler<'a>, TypeDef> ScalaGenerator<'a> for ScalaTypeGenerator<'a, Output, TypeDef> {
	fn options(&self) -> &'a ScalaOptions {
		self.options
	}
}

impl <'a, Output: OutputHandler<'a>, TypeDef: 'a + model::GeneratableType<'a>> TypeGenerator<'a> for ScalaTypeGenerator<'a, Output, TypeDef> {
	type TypeDefinition = TypeDef;

	fn type_def(&self) -> Named<'a, TypeDef> {
		self.type_def
	}

	fn write_header(&mut self) -> Result<(), GeneratorError> {
		self.write_package(&self.type_def.name().package)?;
		writeln!(self.file, "sealed abstract class {}", make_type_name(&self.type_def.name().name))?;
		writeln!(self.file, "object {} {{", make_type_name(&self.type_def.name().name))?;
		self.indent_increase();
		
		Ok(())
	}

	fn write_version_header(&mut self, t: LangType<'a>) -> Result<(), GeneratorError> {
		match t {
			LangType::Versioned(VersionedTypeKind::Struct, _, version, _, fields) => {
				self.write_indent()?;
				write!(self.file, "final case class V{}", version)?;
				self.write_type_params(&self.type_def().type_params())?;
				writeln!(self.file, "(")?;
				self.indent_increase();

				for field in fields.build()? {
					self.write_indent()?;
					write!(self.file, "{}: ", make_field_name(field.name))?;
					self.write_type(&field.field_type)?;
					writeln!(self.file, ",")?;
				}

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, ") extends {}", self.type_def.name().name)?;

				self.write_indent()?;
				writeln!(self.file, "object V{} {{", version)?;
				self.indent_increase();
			},
			LangType::Versioned(VersionedTypeKind::Enum, _, version, _, fields) => {
				self.write_indent()?;
				write!(self.file, "sealed abstract class V{}", version)?;
				self.write_type_params(&self.type_def().type_params())?;
				writeln!(self.file, " extends {}", self.type_def.name().name)?;

				self.write_indent()?;
				writeln!(self.file, "object V{} {{", version)?;
				self.indent_increase();

				for field in fields.build()? {
					self.write_indent()?;
					write!(self.file, "final case class {}", make_type_name(field.name))?;
					self.write_type_params(&self.type_def().type_params())?;
					write!(self.file, "({}: ", make_field_name(field.name))?;
					self.write_type(&field.field_type)?;
					write!(self.file, ") extends V{}", version)?;
					self.write_type_params(&self.type_def().type_params())?;
					writeln!(self.file)?;
				}
			},
			LangType::Interface(_, version, _, methods) => {
				self.write_indent()?;
				write!(self.file, "trait V{}[R, E", version)?;
				for t in self.type_def().type_params() {
					write!(self.file, ", {}", t)?;
				}
				writeln!(self.file, "] {{")?;

				self.indent_increase();

				let methods = methods.build()?;

				for method in methods {
					self.write_indent()?;
					write!(self.file, "def {}", make_field_name(method.name))?;
					self.write_type_params(&method.type_params)?;
					write!(self.file, "(")?;

					for_sep!(type_param, method.type_params, { write!(self.file, ", ")? }, {
						write!(self.file, "{}_codec: {}.Codec[{}]", type_param, RUNTIME_PACKAGE, type_param)?;
					});
					if !method.type_params.is_empty() && !method.parameters.is_empty() {
						write!(self.file, ", ")?;
					}
					for_sep!(param, method.parameters, { write!(self.file, ", ")? }, {
						write!(self.file, "{}: ", param.name)?;
						self.write_type(&param.param_type)?;
					});
					write!(self.file, "): zio.ZIO[R, E, ")?;
					self.write_type(&method.return_type)?;
					writeln!(self.file, "]")?;
				}

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}}")?;

				self.write_indent()?;
				writeln!(self.file, "object V{} {{", version)?;
				self.indent_increase();
			},
			_ => return Err(GeneratorError::CouldNotGenerateType)
		}

		Ok(())
	}

	fn write_operation(&mut self, operation: OperationInfo<'a>) -> Result<(), GeneratorError> {
		let mut type_params = Vec::new();
		if operation.params.iter().any(|(_, param)| match param { LangType::RemoteConnection => true, _ => false, }) {
			type_params.push("R");
			type_params.push("E");
		}
		for type_param in &operation.type_params {
			type_params.push(type_param);
		}


		let is_func = !type_params.is_empty() || !operation.params.is_empty();

		self.write_indent()?;
		if is_func {
			write!(self.file, "def ")?;
		}
		else {
			write!(self.file, "val ")?;
		}

		self.write_operation_name(&operation.operation)?;
		self.write_type_params(&type_params)?;
		if is_func {
			write!(self.file, "(")?;
			for_sep!((param_name, param), operation.params, { write!(self.file, ", ")?; }, {
				write!(self.file, "{}: ", param_name)?;
				self.write_type(&param)?;
			});
			write!(self.file, ")")?;
		}
		write!(self.file, ": ")?;
		self.write_type(&operation.result)?;
		write!(self.file, " = ")?;

		self.write_expr_statement(operation.implementation)?;


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

impl <'a, Output: OutputHandler<'a>, TypeDef: model::GeneratableType<'a>> ScalaTypeGenerator<'a, Output, TypeDef> {


	fn open(model: &'a model::Verilization, options: &'a ScalaOptions, output: &'a mut Output, type_def: Named<'a, TypeDef>) -> Result<Self, GeneratorError> {
		let file = open_scala_file(options, output, type_def.name())?;
		Ok(ScalaTypeGenerator {
			file: file,
			model: model,
			options: options,
			type_def: type_def,
			scope: type_def.scope(),
			indentation_level: 0,
		})
	}

	fn write_expr_statement(&mut self, stmt: LangExprStmt<'a>) -> Result<(), GeneratorError> {
		match stmt {
			LangExprStmt::Expr(expr) => {
				self.write_expr(&expr)?;
				writeln!(self.file, ";")?;
			},

			LangExprStmt::CreateCodec { t, read, write } => {
				write!(self.file, "new {}.Codec[", RUNTIME_PACKAGE)?;
				self.write_type(&t)?;
				writeln!(self.file, "] {{")?;
				self.indent_increase();


				self.write_indent()?;
				write!(self.file, "override def read[R, E]({}: {}.FormatReader[R, E]): zio.ZIO[R, E, ", ScalaLanguage::format_reader_name(), RUNTIME_PACKAGE)?;
				self.write_type(&t)?;
				write!(self.file, "] = ")?;
				
				self.write_statement(*read, true)?;

				self.write_indent()?;
				write!(self.file, "override def write[R, E]({}: {}.FormatWriter[R, E], value: ", ScalaLanguage::format_writer_name(), RUNTIME_PACKAGE)?;
				self.write_type(&t)?;
				write!(self.file, "): zio.ZIO[R, E, Unit] = ")?;

				self.write_statement(*write, true)?;

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}}")?;
			},

			LangExprStmt::CreateConverter { from_type, to_type, body } => {
				write!(self.file, "new {}.Converter[", RUNTIME_PACKAGE)?;
				self.write_type(&from_type)?;
				write!(self.file, ", ")?;
				self.write_type(&to_type)?;
				writeln!(self.file, "] {{")?;
				self.indent_increase();


				self.write_indent()?;
				write!(self.file, "override def convert({}: ", ScalaLanguage::convert_prev_param_name())?;
				self.write_type(&from_type)?;
				write!(self.file, "): ")?;
				self.write_type(&to_type)?;
				write!(self.file, " = ")?;

				self.write_statement(*body, false)?;

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}};")?;
			},

			LangExprStmt::CreateRemoteWrapper { t, connection, id, methods } => {
				write!(self.file, "new {}.RemoteObject[R, E](", RUNTIME_PACKAGE)?;
				self.write_expr(&connection)?;
				write!(self.file, ", ")?;
				self.write_expr(&id)?;
				write!(self.file, ") with ")?;
				self.write_type(&t)?;
				writeln!(self.file, " {{")?;
				self.indent_increase();


				for method in methods {
					self.write_indent()?;
					write!(self.file, "override def {}", method.name)?;

					self.write_type_params(&method.type_params)?;

					write!(self.file, "(")?;
					for_sep!(type_param, method.type_params, { write!(self.file, ", ")? }, {
						write!(self.file, "{}_codec: {}.Codec[{}]", type_param, RUNTIME_PACKAGE, type_param)?;
					});
					if !method.type_params.is_empty() && !method.parameters.is_empty() {
						write!(self.file, ", ")?;
					}
					for_sep!(param, &method.parameters, { write!(self.file, ", ")? }, {
						write!(self.file, "{}: ", param.name)?;
						self.write_type(&param.param_type)?;
					});
					write!(self.file, "): zio.ZIO[R, E, ")?;
					self.write_type(&method.return_type)?;
					writeln!(self.file, "] =")?;
					self.indent_increase();

					
					self.write_indent()?;
					write!(self.file(), "this.remote_connection.invokeMethod(this.object_id, \"{}\", _root_.scala.collection.immutable.Seq[{}.RemoteConnection.MethodArgument[_]](", method.name, RUNTIME_PACKAGE)?;
					for_sep!(param, &method.parameters, { write!(self.file(), ", ")?; }, {
						write!(self.file(), "new {}.RemoteConnection.MethodArgument[", RUNTIME_PACKAGE)?;
						self.write_type(&param.param_type)?;
						write!(self.file(), "]({}, ", param.name)?;
						self.write_expr(&self.build_codec(param.param_type.clone())?)?;
						write!(self.file(), ")")?;
					});
					write!(self.file(), "), ")?;
					self.write_expr(&self.build_codec(method.return_type.clone())?)?;
					writeln!(self.file(), ")")?;

					self.indent_decrease();
				}

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}}")?;
			},
		}

		Ok(())
	}

	fn record_io_expr(&self, expr: &mut LangExpr<'a>, ops: &mut Vec<(String, LangExpr<'a>)>) {
		let name = format!("value{}", ops.len());
		let old_expr = std::mem::replace(expr, LangExpr::Identifier(name.clone()));
		ops.push((name, old_expr));
	}

	fn gather_io_exprs(&self, expr: &mut LangExpr<'a>, ops: &mut Vec<(String, LangExpr<'a>)>) {
		match expr {
			LangExpr::ReadDiscriminator | LangExpr::WriteDiscriminator(_) => self.record_io_expr(expr, ops),

			LangExpr::CodecRead { codec } => {
				self.gather_io_exprs(codec, ops);
				self.record_io_expr(expr, ops);
			},
			LangExpr::CodecWrite { codec, value } => {
				self.gather_io_exprs(codec, ops);
				self.gather_io_exprs(value, ops);
				self.record_io_expr(expr, ops);
			},



			LangExpr::InvokeConverter { converter, value } => {
				self.gather_io_exprs(converter, ops);
				self.gather_io_exprs(value, ops);
			},

			LangExpr::InvokeOperation(_, _, _, args) => {
				for arg in args {
					self.gather_io_exprs(arg, ops);
				}
			},
			LangExpr::InvokeUserConverter { args, .. } => {
				for arg in args {
					self.gather_io_exprs(arg, ops);
				}
			},
			LangExpr::CreateStruct(_, _, _, fields) => {
				for (_, value) in fields {
					self.gather_io_exprs(value, ops);
				}
			},
			LangExpr::CreateEnum(_, _, _, _, value) => {
				self.gather_io_exprs(value, ops);
			},
			LangExpr::StructField(_, _, _, value) => {
				self.gather_io_exprs(value, ops);
			},
			_ => (),
		}
	}
	
	fn write_statement(&mut self, stmt: LangStmt<'a>, use_io: bool) -> Result<(), GeneratorError> {
		match stmt {
			LangStmt::Expr(mut exprs, mut result_expr) if use_io => {

				let mut io_ops = Vec::new();
				let mut ignored_values = HashSet::new();

				for mut expr in &mut exprs {
					self.gather_io_exprs(&mut expr, &mut io_ops);
					match expr {
						LangExpr::Identifier(name) => { ignored_values.insert(name.clone()); },
						_ => (),
					};
				}
				
				if let Some(result_expr) = &mut result_expr {
					self.gather_io_exprs(result_expr, &mut io_ops);
				}

				if io_ops.is_empty() {
					write!(self.file, "zio.IO.succeed(")?;
					if let Some(result_expr) = result_expr {
						self.write_expr(&result_expr)?;
					}
					else {
						write!(self.file, "()")?;
					}
					writeln!(self.file, ")")?;
				}
				else {
					writeln!(self.file, "for {{")?;
					self.indent_increase();
	
					for (name, expr) in io_ops {
						self.write_indent()?;
						if ignored_values.contains(&name) {
							write!(self.file, "_")?;
						}
						else {
							write!(self.file, "{}", name)?;
						}
						write!(self.file, " <- ")?;
						self.write_expr(&expr)?;
						writeln!(self.file)?;
					}
	
					self.indent_decrease();
					self.write_indent()?;
					write!(self.file, "}} yield ")?;
	
					if let Some(result_expr) = result_expr {
						self.write_expr(&result_expr)?;
					}
					else {
						write!(self.file, "()")?;
					}
	
					writeln!(self.file)?;
				}
			},

			LangStmt::Expr(exprs, result_expr) => {
				writeln!(self.file, "{{")?;
				self.indent_increase();

				for expr in exprs {
					self.write_indent()?;
					self.write_expr(&expr)?;
					writeln!(self.file)?;
				}

				if let Some(result_expr) = result_expr {
					self.write_indent()?;
					self.write_expr(&result_expr)?;
					writeln!(self.file)?;
				}

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}}")?;
			},

			LangStmt::MatchEnum { mut value, value_type, cases } => {
				if cases.is_empty() {
					self.write_expr(&value)?;
					writeln!(self.file)?;
					return Ok(())
				}

				let mut io_ops = Vec::new();
				if use_io {
					self.gather_io_exprs(&mut value, &mut io_ops);
				}

				for (name, op) in &io_ops {
					self.write_expr(op)?;
					writeln!(self.file, ".flatMap {{ {} =>", name)?;
					self.indent_increase();
					self.write_indent()?;
				}


				self.write_expr(&value)?;
				writeln!(self.file, " match {{")?;
				self.indent_increase();

				for MatchCase { binding_name, case_name, body } in cases {
					self.write_indent()?;
					write!(self.file, "case ")?;
					match &value_type {
						LangType::Versioned(_, name, version, _, _) => {
							self.write_qual_name(name)?;
							write!(self.file, ".V{}.{}({})", version, make_type_name(&case_name), binding_name)?;
						},
						_ => panic!("Invalid enum type."),
					}
					
					write!(self.file, " => ")?;

					self.write_statement(body, use_io)?;
				}
				
				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}}")?;



				for _ in &io_ops {
					self.indent_decrease();
					self.write_indent()?;
					writeln!(self.file, "}}")?;
				}
			},

			LangStmt::MatchDiscriminator { mut value, cases } => {
				let mut io_ops = Vec::new();
				if use_io {
					self.gather_io_exprs(&mut value, &mut io_ops);
				}

				for (name, op) in &io_ops {
					self.write_expr(op)?;
					writeln!(self.file, ".flatMap {{ {} =>", name)?;
					self.indent_increase();
					self.write_indent()?;
				}

				self.write_expr(&value)?;
				writeln!(self.file, " match {{")?;
				self.indent_increase();

				for (n, body) in cases {
					self.write_indent()?;
					write!(self.file, "case {}.Util.BigIntValue({}) => ", RUNTIME_PACKAGE, n)?;

					self.write_statement(body, use_io)?;
				}
				
				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}}")?;

				for _ in &io_ops {
					self.indent_decrease();
					self.write_indent()?;
					writeln!(self.file, "}}")?;
				}
			},
			

		}

		Ok(())
	}

	fn write_type_params<S: AsRef<str>>(&mut self, params: &Vec<S>) -> Result<(), GeneratorError> {
		if !params.is_empty() {
			write!(self.file, "[")?;
			for_sep!(param, params, { write!(self.file, ", ")?; }, {
				write!(self.file, "{}", param.as_ref())?;
			});
			write!(self.file, "]")?;
		}
	
		Ok(())
	}
}


pub struct ScalaLanguage {}

impl Language for ScalaLanguage {
	type Options = ScalaOptions;

    fn name() -> &'static str {
        "scala"
    }

	fn generate<Output: for<'output> OutputHandler<'output>>(model: &model::Verilization, options: Self::Options, output: &mut Output) -> Result<(), GeneratorError> {
		let mut codegen = ScalaCodeGenerator {
			model,
			options: &options,
			output,
		};
		codegen.generate(model)
	}

}

impl LanguageOptions for ScalaOptions {
	type Builder = ScalaOptionsBuilder;


	fn build(builder: Self::Builder) -> Result<Self, GeneratorError> {
		Ok(ScalaOptions {
			output_dir: builder.output_dir.ok_or_else(|| GeneratorError::InvalidOptions(String::from("Output directory not specified")))?,
			package_mapping: builder.package_mapping,
			library_mapping: builder.library_mapping,
		})
	}
}

impl LanguageOptionsBuilder for ScalaOptionsBuilder {
	fn empty() -> ScalaOptionsBuilder {
		ScalaOptionsBuilder {
			output_dir: None,
			package_mapping: HashMap::new(),
			library_mapping: HashMap::new(),
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

            let scala_package = model::PackageName::from_str(value.to_str().unwrap());

			if self.library_mapping.contains_key(&package) || self.package_mapping.insert(package, scala_package).is_some() {
				return Err(GeneratorError::InvalidOptions(format!("Package already mapped: {}", pkg)))
			}
			Ok(())
		}
		else if let Some(pkg) = name.strip_prefix("lib:") {
			let package = model::PackageName::from_str(pkg);

            let scala_package = model::PackageName::from_str(value.to_str().unwrap());

			if self.package_mapping.contains_key(&package) || self.library_mapping.insert(package, scala_package).is_some() {
				return Err(GeneratorError::InvalidOptions(format!("Package already mapped: {}", pkg)))
			}
			Ok(())
		}
		else {
			Err(GeneratorError::InvalidOptions(format!("Unknown option: {}", name)))
		}
	}
}

struct ScalaCodeGenerator<'a, Output> {
	model: &'a model::Verilization,
	options: &'a ScalaOptions,
	output: &'a mut Output,
}

impl <'a, 'b, Output : OutputHandler<'a>> GeneratorFactory<'a> for ScalaCodeGenerator<'b, Output> {
	type ConstGen = ScalaConstGenerator<'a, Output>;
	type VersionedTypeGen = ScalaTypeGenerator<'a, Output, model::VersionedTypeDefinitionData>;
	type InterfaceTypeGen = ScalaTypeGenerator<'a, Output, model::InterfaceTypeDefinitionData>;

	fn create_constant_generator(&'a mut self, constant: Named<'a, model::Constant>) -> Result<Self::ConstGen, GeneratorError> {
		ScalaConstGenerator::open(self.model, self.options, self.output, constant)
	}

	fn create_versioned_type_generator(&'a mut self, t: Named<'a, model::VersionedTypeDefinitionData>) -> Result<Self::VersionedTypeGen, GeneratorError> {
		ScalaTypeGenerator::open(self.model, self.options, self.output, t)
	}

	fn create_interface_type_generator(&'a mut self, t: Named<'a, model::InterfaceTypeDefinitionData>) -> Result<Self::InterfaceTypeGen, GeneratorError> {
		ScalaTypeGenerator::open(self.model, self.options, self.output, t)
	}
}

