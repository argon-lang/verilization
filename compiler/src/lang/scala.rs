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


pub trait ScalaGenerator<'model, 'opt> : Generator<'model, ScalaLanguage> + GeneratorWithFile {
	fn options(&self) -> &'opt ScalaOptions;
	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model>;


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

	fn write_type_args(&mut self, args: &Vec<LangType<'model>>) -> Result<(), GeneratorError> {
		if !args.is_empty() {
			write!(self.file(), "[")?;
			for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
				self.write_type(arg)?;
			});
			write!(self.file(), "]")?;
		}
	
		Ok(())
	}
	
	
	fn write_type(&mut self, t: &LangType<'model>) -> Result<(), GeneratorError> {
		Ok(match t {
			// Map built-in types to the equivalent Scala type.
			LangType::Nat | LangType::Int => write!(self.file(), "scala.math.BigInt")?,
			
			LangType::U8 | LangType::I8 => write!(self.file(), "scala.Byte")?,
			
			LangType::U16 | LangType::I16 => write!(self.file(), "scala.Short")?,
	
			LangType::U32 | LangType::I32 => write!(self.file(), "scala.Int")?,
	
			LangType::U64 | LangType::I64 => write!(self.file(), "scala.Long")?,
	
			LangType::String => write!(self.file(), "scala.Predef.String")?,
	
	
			LangType::List(inner) => {
				write!(self.file(), "zio.Chunk[")?;
				self.write_type(inner)?;
				write!(self.file(), "]")?;
			},
			LangType::Option(inner) => {
				write!(self.file(), "scala.Option[")?;
				self.write_type(inner)?;
				write!(self.file(), "]")?;
			},
	
			LangType::Versioned(name, version, args) => {
				self.write_qual_name(&name)?;
				write!(self.file(), ".V{}", version)?;
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
				write!(self.file(), "{}.Converter.identity[", RUNTIME_PACKAGE)?;
				self.write_type(t)?;
				write!(self.file(), "]")?;
			},
			LangExpr::MapListConverter { from_type, to_type, element_converter } => {
				write!(self.file(), "{}.List.converter[", RUNTIME_PACKAGE)?;
				self.write_type(from_type)?;
				write!(self.file(), ", ")?;
				self.write_type(to_type)?;
				write!(self.file(), "](")?;
				self.write_expr(element_converter)?;
				write!(self.file(), ")")?;
				
			},
			LangExpr::MapOptionConverter { from_type, to_type, element_converter } => {
				write!(self.file(), "{}.Option.converter[", RUNTIME_PACKAGE)?;
				self.write_type(from_type)?;
				write!(self.file(), ", ")?;
				self.write_type(to_type)?;
				write!(self.file(), "](")?;
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
				write!(self.file(), "{}.Option.codec(", RUNTIME_PACKAGE)?;
				self.write_expr(&*inner)?;
				write!(self.file(), ")")?
			},
			LangExpr::ReadDiscriminator => write!(self.file(), "{}.StandardCodecs.natCodec.read(reader)", RUNTIME_PACKAGE)?,
			LangExpr::WriteDiscriminator(value) => write!(self.file(), "{}.StandardCodecs.natCodec.write(writer, {})", RUNTIME_PACKAGE, value)?,
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
				self.write_operation_name(op)?;
				self.write_type_args(type_args)?;
				self.write_args(args)?;
			},
			LangExpr::InvokeUserConverter { name, prev_ver, version, type_args, args } => {
				self.write_qual_name(name)?;
				write!(self.file(), "_Conversions.v{}ToV{}", prev_ver, version)?;
				self.write_type_args(type_args)?;
				self.write_args(args)?;
			},
			LangExpr::ConstantValue(name, version) => {
				self.write_qual_name(name)?;
				write!(self.file(), ".{}", Self::constant_version_name(version))?;
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

impl <'model, 'opt, TImpl> GeneratorNameMapping<ScalaLanguage> for TImpl where TImpl : ScalaGenerator<'model, 'opt> {
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

struct ScalaConstGenerator<'model, 'opt, 'output, Output: OutputHandler> {
	file: Output::FileHandle<'output>,
	model: &'model model::Verilization,
	options: &'opt ScalaOptions,
	constant: Named<'model, model::Constant>,
	scope: model::Scope<'model>,
}

impl <'model, 'opt, 'output, Output: OutputHandler> Generator<'model, ScalaLanguage> for ScalaConstGenerator<'model, 'opt, 'output, Output> {
	fn model(&self) -> &'model model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> GeneratorWithFile for ScalaConstGenerator<'model, 'opt, 'output, Output> {
	type GeneratorFile = Output::FileHandle<'output>;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> ScalaGenerator<'model, 'opt> for ScalaConstGenerator<'model, 'opt, 'output, Output> {
	fn options(&self) -> &'opt ScalaOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.constant.referenced_types()
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> ConstGenerator<'model, ScalaLanguage> for ScalaConstGenerator<'model, 'opt, 'output, Output> {
	fn constant(&self) -> Named<'model, model::Constant> {
		self.constant
	}

	fn write_header(&mut self) -> Result<(), GeneratorError> {
        self.write_package(&self.constant.name().package)?;

		writeln!(self.file, "object {} {{", self.constant.name().name)?;

		Ok(())
	}

	fn write_constant(&mut self, version_name: String, t: LangType<'model>, value: LangExpr<'model>) -> Result<(), GeneratorError> {
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


impl <'model, 'opt, 'output, Output: OutputHandler> ScalaConstGenerator<'model, 'opt, 'output, Output> {

	fn open(model: &'model model::Verilization, options: &'opt ScalaOptions, output: &'output mut Output, constant: Named<'model, model::Constant>) -> Result<Self, GeneratorError> {
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

struct ScalaTypeGenerator<'model, 'opt, 'output, Output: OutputHandler, Extra> {
	options: &'opt ScalaOptions,
	model: &'model model::Verilization,
	file: Output::FileHandle<'output>,
	type_def: Named<'model, model::TypeDefinitionData>,
	scope: model::Scope<'model>,
	indentation_level: u32,
	_extra: Extra,
}

trait ScalaExtraGeneratorOps {
	fn write_versioned_type(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
	fn write_versioned_type_object_data(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> Generator<'model, ScalaLanguage> for ScalaTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	fn model(&self) -> &'model model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> GeneratorWithFile for ScalaTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	type GeneratorFile = Output::FileHandle<'output>;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> Indentation for ScalaTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	fn indentation_size(&mut self) -> &mut u32 {
		&mut self.indentation_level
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> ScalaGenerator<'model, 'opt> for ScalaTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	fn options(&self) -> &'opt ScalaOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.type_def.referenced_types()
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler, GenTypeKind> VersionedTypeGenerator<'model, ScalaLanguage, GenTypeKind> for ScalaTypeGenerator<'model, 'opt, 'output, Output, GenTypeKind>
	where ScalaTypeGenerator<'model, 'opt, 'output, Output, GenTypeKind> : ScalaExtraGeneratorOps
{
	fn type_def(&self) -> Named<'model, model::TypeDefinitionData> {
		self.type_def
	}

	fn write_header(&mut self) -> Result<(), GeneratorError> {
		self.write_package(&self.type_def.name().package)?;
		writeln!(self.file, "sealed abstract class {}", self.type_def.name().name)?;
		writeln!(self.file, "object {} {{", self.type_def.name().name)?;
		self.indent_increase();
		
		Ok(())
	}

	fn write_version_header(&mut self, ver_type: &model::TypeVersionInfo<'model>) -> Result<(), GeneratorError> {

		let version = &ver_type.version;

		self.write_versioned_type(ver_type)?;

		self.write_indent()?;
		writeln!(self.file, "object V{} {{", version)?;
		self.indent_increase();
		self.write_versioned_type_object_data(ver_type)?;

		Ok(())
	}

	fn write_operation(&mut self, operation: OperationInfo<'model>) -> Result<(), GeneratorError> {
		let is_func = !operation.type_params.is_empty() || !operation.params.is_empty();

		self.write_indent()?;
		if is_func {
			write!(self.file, "def ")?;
		}
		else {
			write!(self.file, "val ")?;
		}

		self.write_operation_name(&operation.operation)?;
		self.write_type_params(&operation.type_params)?;
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

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> ScalaTypeGenerator<'model, 'opt, 'output, Output, Extra> where ScalaTypeGenerator<'model, 'opt, 'output, Output, Extra> : ScalaExtraGeneratorOps {


	fn open(model: &'model model::Verilization, options: &'opt ScalaOptions, output: &'output mut Output, type_def: Named<'model, model::TypeDefinitionData>) -> Result<Self, GeneratorError> where Extra : Default {
		let file = open_scala_file(options, output, type_def.name())?;
		Ok(ScalaTypeGenerator {
			file: file,
			model: model,
			options: options,
			type_def: type_def,
			scope: type_def.scope(),
			indentation_level: 0,
			_extra: Extra::default(),
		})
	}

	fn write_expr_statement(&mut self, stmt: LangExprStmt<'model>) -> Result<(), GeneratorError> {
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
				write!(self.file, "override def read[R, E](reader: {}.FormatReader[R, E]): zio.ZIO[R, E, ", RUNTIME_PACKAGE)?;
				self.write_type(&t)?;
				write!(self.file, "] = ")?;
				
				self.write_statement(*read, true)?;

				self.write_indent()?;
				write!(self.file, "override def write[R, E](writer: {}.FormatWriter[R, E], value: ", RUNTIME_PACKAGE)?;
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
				write!(self.file, "override def convert({}: ", Self::convert_prev_param_name())?;
				self.write_type(&from_type)?;
				write!(self.file, "): ")?;
				self.write_type(&to_type)?;
				write!(self.file, " = ")?;

				self.write_statement(*body, false)?;

				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}};")?;
			},
		}

		Ok(())
	}

	fn record_io_expr(&self, expr: &mut LangExpr<'model>, ops: &mut Vec<(String, LangExpr<'model>)>) {
		let name = format!("value{}", ops.len());
		let old_expr = std::mem::replace(expr, LangExpr::Identifier(name.clone()));
		ops.push((name, old_expr));
	}

	fn gather_io_exprs(&self, expr: &mut LangExpr<'model>, ops: &mut Vec<(String, LangExpr<'model>)>) {
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
			LangExpr::MapListConverter { element_converter, .. } => self.gather_io_exprs(element_converter, ops),
			LangExpr::MapOptionConverter { element_converter, .. } => self.gather_io_exprs(element_converter, ops),
			LangExpr::ListCodec(inner) => self.gather_io_exprs(inner, ops),
			LangExpr::OptionCodec(inner) => self.gather_io_exprs(inner, ops),

			LangExpr::InvokeOperation(_, _, _, _, args) => {
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
	
	fn write_statement(&mut self, stmt: LangStmt<'model>, use_io: bool) -> Result<(), GeneratorError> {
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
						LangType::Versioned(name, version, args) => {
							self.write_qual_name(name)?;
							write!(self.file, ".V{}.{}({})", version, case_name, binding_name)?;
						},
						_ => Err("Invalid enum type.")?,
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

	fn write_type_params(&mut self, params: &Vec<String>) -> Result<(), GeneratorError> {
		if !params.is_empty() {
			write!(self.file, "[")?;
			for_sep!(param, params, { write!(self.file, ", ")?; }, {
				write!(self.file, "{}", param)?;
			});
			write!(self.file, "]")?;
		}
	
		Ok(())
	}
}

impl <'model, 'opt, 'output, 'state, Output: OutputHandler> ScalaExtraGeneratorOps for ScalaTypeGenerator<'model, 'opt, 'state, Output, GenStructType> {

	fn write_versioned_type(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		self.write_indent()?;
		write!(self.file, "final case class V{}", ver_type.version)?;
		self.write_type_params(&self.type_def().type_params())?;
		writeln!(self.file, "(")?;
		self.indent_increase();

		for (field_name, field) in &ver_type.ver_type.fields {
			self.write_indent()?;
			write!(self.file, "{}: ", field_name)?;
			self.write_type(&self.build_type(&ver_type.version, &field.field_type)?)?;
			writeln!(self.file, ",")?;
		}

		self.indent_decrease();
		self.write_indent()?;
		writeln!(self.file, ") extends {}", self.type_def.name().name)?;

		Ok(())
	}

	fn write_versioned_type_object_data(&mut self, _ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		Ok(())
	}
}

impl <'model, 'opt, 'output, 'state, Output: OutputHandler> ScalaExtraGeneratorOps for ScalaTypeGenerator<'model, 'opt, 'state, Output, GenEnumType> {
	fn write_versioned_type(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		self.write_indent()?;
		write!(self.file, "sealed abstract class V{}", ver_type.version)?;
		self.write_type_params(&self.type_def().type_params())?;
		writeln!(self.file, " extends {}", self.type_def.name().name)?;
		Ok(())
	}

	fn write_versioned_type_object_data(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		for (field_name, field) in &ver_type.ver_type.fields {
			self.write_indent()?;
			write!(self.file, "final case class {}", field_name)?;
			self.write_type_params(&self.type_def().type_params())?;
			write!(self.file, "({}: ", field_name)?;
			self.write_type(&self.build_type(&ver_type.version, &field.field_type)?)?;
			write!(self.file, ") extends V{}", ver_type.version)?;
			self.write_type_params(&self.type_def().type_params())?;
			writeln!(self.file)?;
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
			let mut const_gen = ScalaConstGenerator::open(model, &options, output, constant)?;
			const_gen.generate()?;
		}

		for t in model.types() {
			match t {
				model::NamedTypeDefinition::StructType(t) => {
					let mut type_gen: ScalaTypeGenerator<_, GenStructType> = ScalaTypeGenerator::open(model, &options, output, t)?;
					type_gen.generate()?;		
				},
				model::NamedTypeDefinition::EnumType(t) => {
					let mut type_gen: ScalaTypeGenerator<_, GenEnumType> = ScalaTypeGenerator::open(model, &options, output, t)?;
					type_gen.generate()?;		
				},
			}
		}

		Ok(())
	}

}
