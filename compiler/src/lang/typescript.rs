use crate::model;
use model::Named;
use crate::lang::{GeneratorError, Language, OutputHandler};
use std::ffi::OsString;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use num_bigint::{BigUint, BigInt, Sign};
use super::generator::*;
use crate::util::{capitalize_identifier, uncapitalize_identifier};

pub struct TSOptionsBuilder {
	output_dir: Option<OsString>,
	package_mapping: HashMap<model::PackageName, OsString>,
	library_mapping: HashMap<model::PackageName, OsString>,
}

pub struct TSOptions {
	pub output_dir: OsString,
	pub package_mapping: HashMap<model::PackageName, OsString>,
	pub library_mapping: HashMap<model::PackageName, OsString>,
}


fn open_ts_file<'output, Output: OutputHandler>(options: &TSOptions, output: &'output mut Output, name: &model::QualifiedName) -> Result<Output::FileHandle<'output>, GeneratorError> {
	let pkg_dir = options.package_mapping.get(&name.package).ok_or(format!("Unmapped package: {}", name.package))?;
	let mut path = PathBuf::from(&options.output_dir);
	path.push(pkg_dir);
	path.push(name.name.clone() + ".ts");
	Ok(output.create_file(path)?)
}

fn make_type_name(name: &str) -> String {
	let mut name = String::from(name);
	capitalize_identifier(&mut name);
	name
}

pub fn make_field_name(field_name: &str) -> String {
	let mut name = String::from(field_name);
	uncapitalize_identifier(&mut name);
	name
}

pub trait TSGenerator<'model> : Generator<'model, TypeScriptLanguage> + GeneratorWithFile {
	fn generator_element_name(&self) -> Option<&'model model::QualifiedName>;
	fn options(&self) -> &TSOptions;
	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model>;
	fn current_dir(&self) -> Result<PathBuf, GeneratorError>;

	fn write_import_name(&mut self, name: &model::QualifiedName) -> Result<(), GeneratorError> {
		write!(self.file(), "sym_")?;

		for part in &name.package.package {
			write!(self.file(), "{}_", part)?;
		}

		write!(self.file(), "{}", &name.name)?;

		Ok(())
	}

	fn write_import<P: AsRef<Path>>(&mut self, t: &model::QualifiedName, current_path: &P) -> Result<(), GeneratorError> {
		let is_rel;

		let mut import_path = if let Some(import_pkg_dir) = self.options().package_mapping.get(&t.package) {
			let mut abs_import_path = PathBuf::from(&self.options().output_dir);
			abs_import_path.push(import_pkg_dir);

			is_rel = true;
			pathdiff::diff_paths(abs_import_path, current_path).ok_or("Could not find relative path.")?
		}
		else if let Some(import_lib) = self.options().library_mapping.get(&t.package) {
			is_rel = false;
			PathBuf::from(import_lib)
		}
		else {
			return Err(GeneratorError::from(format!("Unmapped package: {}", t.package)))
		};

		import_path.push(t.name.clone() + ".js");


		write!(self.file(), "import * as ")?;
		self.write_import_name(&t)?;
		write!(self.file(), " from \"")?;
		if is_rel {
			write!(self.file(), "./")?;
		}
		writeln!(self.file(), "{}\";", import_path.to_str().unwrap())?;

		Ok(())
	}

	fn write_imports(&mut self) -> Result<(), GeneratorError> {
		let current_path = self.current_dir()?;


		let mut referenced_types: Vec<_> = self.referenced_types().collect();
		referenced_types.sort();

		for t in referenced_types {
			if self.generator_element_name() == Some(t) {
				continue;
			}

			let t = match self.scope().lookup(t.clone()) {
				model::ScopeLookup::TypeParameter(_) => continue,
				model::ScopeLookup::NamedType(t) => t,
			};

			self.write_import(&t, &current_path)?;
		}

		Ok(())
	}

	fn write_type_args(&mut self, args: &Vec<LangType<'model>>) -> Result<(), GeneratorError> {
		if !args.is_empty() {
			write!(self.file(), "<")?;
			for_sep!(arg, args, { write!(self.file(), ", ")?; }, {
				self.write_type(&arg)?;
			});
			write!(self.file(), ">")?;
		}

		Ok(())
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

	fn write_type(&mut self, t: &LangType<'model>) -> Result<(), GeneratorError> {
		Ok(match t {	
			LangType::Versioned(name, version, args) => {
				// Only use a qualifier if not a value of the current type.
				if self.generator_element_name() != Some(name) {
					self.write_import_name(name)?;
					write!(self.file(), ".")?;
				}
	
				write!(self.file(), "V{}", version)?;
				self.write_type_args(&args)?;
			},

			LangType::Extern(name, args) => {
				self.write_import_name(name)?;

				write!(self.file(), ".{}", make_type_name(&name.name))?;
				self.write_type_args(&args)?;
			},

			LangType::TypeParameter(name) => {
				write!(self.file(), "{}", name)?;
			},

			LangType::Converter(from, to) => {
				write!(self.file(), "Converter<")?;
				self.write_type(&*from)?;
				write!(self.file(), ", ")?;
				self.write_type(&*to)?;
				write!(self.file(), ">")?;
			},

			LangType::Codec(t) => {
				write!(self.file(), "Codec<")?;
				self.write_type(&*t)?;
				write!(self.file(), ">")?;
			},
		})
	}

	fn write_operation_name(&mut self, op: &Operation) -> Result<(), GeneratorError> {
		match op {
			Operation::FromPreviousVersion(prev_ver) => write!(self.file(), "fromV{}", prev_ver)?,
			Operation::FinalTypeConverter => write!(self.file(), "converter")?,
			Operation::TypeCodec => write!(self.file(), "codec")?,
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
				write!(self.file(), "Converter.identity<")?;
				self.write_type(t)?;
				write!(self.file(), ">()")?;
			},
			LangExpr::ReadDiscriminator => write!(self.file(), "await natCodec.read(reader)")?,
			LangExpr::WriteDiscriminator(value) => write!(self.file(), "await natCodec.write(writer, {}n)", value)?,
			LangExpr::CodecRead { codec } => {
				write!(self.file(), "await ")?;
				self.write_expr(&*codec)?;
				write!(self.file(), ".read(reader)")?;
			},
			LangExpr::CodecWrite { codec, value } => {
				write!(self.file(), "await ")?;
				self.write_expr(&*codec)?;
				write!(self.file(), ".write(writer, ")?;
				self.write_expr(value)?;
				write!(self.file(), ")")?;
			},
			LangExpr::InvokeOperation(op, target, type_args, args) => {
				match target {
					OperationTarget::VersionedType(name, version) => {
						// Only use a qualifier if not a value of the current type.
						if self.generator_element_name() != Some(name) {
							self.write_import_name(name)?;
							write!(self.file(), ".")?;
						}
			
						write!(self.file(), "V{}.", version)?;
					},
					OperationTarget::ExternType(name) => {
						self.write_import_name(name)?;
						write!(self.file(), ".")?;
					},
				}
				self.write_operation_name(op)?;

				self.write_type_args(type_args)?;
				self.write_args(args)?;
			},
			LangExpr::InvokeUserConverter { name: _, prev_ver, version, type_args, args } => {
				write!(self.file(), "v{}_to_v{}", prev_ver, version)?;
				self.write_type_args(type_args)?;
				self.write_args(args)?;
			},
			LangExpr::ConstantValue(name, version) => {
				// Only use a qualifier if not a value of the current type.
				if self.generator_element_name() != Some(name) {
					self.write_import_name(name)?;
					write!(self.file(), ".")?;
				}
	
				write!(self.file(), "{}", Self::constant_version_name(version))?;
			},
			LangExpr::CreateStruct(_, _, _, fields) => {
				write!(self.file(), "{{ ")?;
				for (field_name, value) in fields {
					write!(self.file(), "{}: ", make_field_name(field_name))?;
					self.write_expr(value)?;
					write!(self.file(), ", ")?;
				}
				write!(self.file(), "}}")?;
			},
			LangExpr::CreateEnum(_, _, _, field_name, value) => {
				write!(self.file(), "{{ tag: \"{}\", {}: ", field_name, make_field_name(field_name))?;
				self.write_expr(value)?;
				write!(self.file(), "}}")?;
			},
			LangExpr::StructField(_, _, field_name, value) => {
				self.write_expr(value)?;
				write!(self.file(), ".{}", make_field_name(field_name))?;
			},
		}

		Ok(())
	}
}

impl <'model, TImpl> GeneratorNameMapping<TypeScriptLanguage> for TImpl where TImpl : TSGenerator<'model> {
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

fn current_dir_of_name<'model, Gen: TSGenerator<'model>>(gen: &Gen, name: &model::QualifiedName) -> Result<PathBuf, GeneratorError> {
	let current_pkg_dir = gen.options().package_mapping.get(&name.package)
		.or_else(|| gen.options().library_mapping.get(&name.package))
		.ok_or_else(|| format!("Unmapped package: {}", name.package))?;
	let mut current_path = PathBuf::from(&gen.options().output_dir);
	current_path.push(current_pkg_dir);
	Ok(current_path)
}



struct TSConstGenerator<'model, 'opt, 'output, Output: OutputHandler> {
	file: Output::FileHandle<'output>,
	model: &'model model::Verilization,
	options: &'opt TSOptions,
	constant: Named<'model, model::Constant>,
	scope: model::Scope<'model>,
}

impl <'model, 'opt, 'output, Output: OutputHandler> Generator<'model, TypeScriptLanguage> for TSConstGenerator<'model, 'opt, 'output, Output> {
	fn model(&self) -> &'model model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> GeneratorWithFile for TSConstGenerator<'model, 'opt, 'output, Output> {
	type GeneratorFile = Output::FileHandle<'output>;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> TSGenerator<'model> for TSConstGenerator<'model, 'opt, 'output, Output> {
	fn generator_element_name(&self) -> Option<&'model model::QualifiedName> {
		Some(self.constant.name())
	}

	fn options(&self) -> &TSOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.constant.referenced_types()
	}

	fn current_dir(&self) -> Result<PathBuf, GeneratorError> {
		current_dir_of_name(self, self.constant.name())
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> ConstGenerator<'model, TypeScriptLanguage> for TSConstGenerator<'model, 'opt, 'output, Output> {
	fn constant(&self) -> Named<'model, model::Constant> {
		self.constant
	}

	fn write_header(&mut self) -> Result<(), GeneratorError> {
		self.write_imports()
	}

	fn write_constant(&mut self, version_name: String, t: LangType<'model>, value: LangExpr<'model>) -> Result<(), GeneratorError> {
		write!(self.file(), "export const {}: ", version_name)?;
		self.write_type(&t)?;
		write!(self.file, " = ")?;
		self.write_expr(&value)?;
		writeln!(self.file, ";")?;

		Ok(())
	}

	fn write_footer(&mut self) -> Result<(), GeneratorError> {
		Ok(())
	}
}



impl <'model, 'opt, 'output, Output: OutputHandler> TSConstGenerator<'model, 'opt, 'output, Output> {

	fn open(model: &'model model::Verilization, options: &'opt TSOptions, output: &'output mut Output, constant: Named<'model, model::Constant>) -> Result<Self, GeneratorError> {
		let file = open_ts_file(options, output, constant.name())?;
		Ok(TSConstGenerator {
			file: file,
			model: model,
			options: options,
			constant: constant,
			scope: constant.scope(),
		})
	}

}

struct TSTypeGenerator<'model, 'opt, 'output, Output: OutputHandler, Extra> {
	file: Output::FileHandle<'output>,
	model: &'model model::Verilization,
	options: &'opt TSOptions,
	type_def: Named<'model, model::VersionedTypeDefinitionData>,
	scope: model::Scope<'model>,
	versions: HashSet<BigUint>,
	indentation_level: u32,
	_extra: Extra,
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> Generator<'model, TypeScriptLanguage> for TSTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	fn model(&self) -> &'model model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> GeneratorWithFile for TSTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	type GeneratorFile = Output::FileHandle<'output>;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> Indentation for TSTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	fn indentation_size(&mut self) -> &mut u32 {
		&mut self.indentation_level
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler, Extra> TSGenerator<'model> for TSTypeGenerator<'model, 'opt, 'output, Output, Extra> {
	fn generator_element_name(&self) -> Option<&'model model::QualifiedName> {
		Some(self.type_def.name())
	}

	fn options(&self) -> &TSOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.type_def.referenced_types()
	}

	fn current_dir(&self) -> Result<PathBuf, GeneratorError> {
		current_dir_of_name(self, self.type_def.name())
	}
}

trait TSExtraGeneratorOps {
	fn write_versioned_type(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError>;
}

impl <'model, 'opt, 'output, Output: OutputHandler, GenTypeKind> VersionedTypeGenerator<'model, TypeScriptLanguage, GenTypeKind> for TSTypeGenerator<'model, 'opt, 'output, Output, GenTypeKind>
	where TSTypeGenerator<'model, 'opt, 'output, Output, GenTypeKind> : TSExtraGeneratorOps
{
	fn type_def(&self) -> Named<'model, model::VersionedTypeDefinitionData> {
		self.type_def
	}

	fn write_header(&mut self) -> Result<(), GeneratorError> {
		writeln!(self.file, "import {{Codec, FormatWriter, FormatReader, Converter, natCodec}} from \"@verilization/runtime\";")?;
		self.write_imports()?;
		
		Ok(())
	}

	fn write_version_header(&mut self, ver_type: &model::TypeVersionInfo<'model>) -> Result<(), GeneratorError> {
		self.write_versioned_type(ver_type)?;

		let version = &ver_type.version;

		let prev_ver: BigInt = BigInt::from_biguint(Sign::Plus, version.clone()) - 1;
		let prev_ver = prev_ver.to_biguint().unwrap();

		if ver_type.explicit_version && !self.versions.is_empty() {
			writeln!(self.file, "import {{v{}_to_v{}}} from \"./{}.conv.js\";", prev_ver, version, self.type_def.name().name)?;
		}
		self.versions.insert(ver_type.version.clone());

		writeln!(self.file, "export namespace V{} {{", version)?;
		self.indent_increase();

		Ok(())
	}

	fn write_operation(&mut self, operation: OperationInfo<'model>) -> Result<(), GeneratorError> {
		let is_func = !operation.type_params.is_empty() || !operation.params.is_empty();

		self.write_indent()?;
		write!(self.file, "export ")?;

		if is_func {
			write!(self.file, "function ")?;
		}
		else {
			write!(self.file, "const ")?;
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

		if is_func {
			writeln!(self.file, " {{")?;
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
		
		Ok(())
	}

}



impl <'model, 'opt, 'output, Output: OutputHandler, GenTypeKind> TSTypeGenerator<'model, 'opt, 'output, Output, GenTypeKind> where TSTypeGenerator<'model, 'opt, 'output, Output, GenTypeKind> : TSExtraGeneratorOps {

	fn open(model: &'model model::Verilization, options: &'opt TSOptions, output: &'output mut Output, type_def: Named<'model, model::VersionedTypeDefinitionData>) -> Result<Self, GeneratorError> where GenTypeKind : Default {
		let file = open_ts_file(options, output, type_def.name())?;
		Ok(TSTypeGenerator {
			file: file,
			model: model,
			options: options,
			type_def: type_def,
			scope: type_def.scope(),
			versions: HashSet::new(),
			indentation_level: 0,
			_extra: GenTypeKind::default(),
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
				writeln!(self.file, "{{")?;
				self.indent_increase();

		
				self.write_indent()?;
				write!(self.file, "async read(reader: FormatReader): Promise<")?;
				self.write_type(t)?;
				writeln!(self.file, "> {{")?;
				self.indent_increase();
				self.write_statement(read)?;
				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}},")?;
		
				self.write_indent()?;
				write!(self.file, "async write(writer: FormatWriter, {}: ", Self::codec_write_value_name())?;
				self.write_type(t)?;
				writeln!(self.file, "): Promise<void> {{")?;
				self.indent_increase();
				self.write_statement(write)?;
				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}},")?;


				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}};")?;
			}

			LangExprStmt::CreateConverter { from_type, to_type, body } => {
				writeln!(self.file, "{{")?;
				self.indent_increase();

		
				self.write_indent()?;
				write!(self.file, "convert({}: ", Self::convert_prev_param_name())?;
				self.write_type(from_type)?;
				write!(self.file, "): ")?;
				self.write_type(to_type)?;
				writeln!(self.file, " {{")?;
				self.indent_increase();
				self.write_statement(body)?;
				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}},")?;
		
				self.indent_decrease();
				self.write_indent()?;
				writeln!(self.file, "}};")?;
			}
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

			LangStmt::MatchEnum { value, value_type: _, cases } => {
				self.write_indent()?;
				write!(self.file, "switch(")?;
				self.write_expr(value)?;
				writeln!(self.file, ".tag) {{")?;

				self.indent_increase();

				for MatchCase { binding_name, case_name, body } in cases {
					self.write_indent()?;
					writeln!(self.file, "case \"{}\":", case_name)?;
					self.write_indent()?;
					writeln!(self.file, "{{")?;

					self.indent_increase();

					self.write_indent()?;
					write!(self.file, "const {} = ", binding_name)?;
					self.write_expr(value)?;
					writeln!(self.file, ".{};", make_field_name(case_name))?;

					self.write_statement(body)?;
					if !body.has_value() {
						self.write_indent()?;
						writeln!(self.file, "break;")?;
					}
					self.indent_decrease();

					self.write_indent()?;
					writeln!(self.file, "}}")?;
				}

				if stmt.has_value() {
					self.write_indent()?;
					write!(self.file, "default: return ")?;
					self.write_expr(value)?;
					writeln!(self.file, ";")?;
				}
					
				self.indent_decrease();

				self.write_indent()?;
				writeln!(self.file, "}}")?;
			},

			LangStmt::MatchDiscriminator { value, cases } => {
				self.write_indent()?;
				write!(self.file, "switch(")?;
				self.write_expr(value)?;
				writeln!(self.file, ") {{")?;

				self.indent_increase();

				for (n, body) in cases {
					self.write_indent()?;
					writeln!(self.file, "case {}n:", n)?;
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
				write!(self.file, "default: throw new Error(\"Unknown tag\");")?;
					
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

impl <'model, 'opt, 'output, Output: OutputHandler> TSExtraGeneratorOps for TSTypeGenerator<'model, 'opt, 'output, Output, GenStructType> {
	fn write_versioned_type(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		write!(self.file, "export interface V{}", ver_type.version)?;
		self.write_type_params(self.type_def().type_params())?;
		writeln!(self.file, " {{")?;
		self.indent_increase();
		for (field_name, field) in &ver_type.ver_type.fields {
			self.write_indent()?;
			write!(self.file, "readonly {}: ", make_field_name(field_name))?;
			self.write_type(&self.build_type(&ver_type.version, &field.field_type)?)?;
			writeln!(self.file, ";")?;
		}
		self.indent_decrease();
		writeln!(self.file, "}}")?;
		Ok(())
	}
}

impl <'model, 'opt, 'output, Output: OutputHandler> TSExtraGeneratorOps for TSTypeGenerator<'model, 'opt, 'output, Output, GenEnumType> {
	fn write_versioned_type(&mut self, ver_type: &model::TypeVersionInfo) -> Result<(), GeneratorError> {
		write!(self.file, "export type V{}", ver_type.version)?;
		self.write_type_params(self.type_def().type_params())?;
		write!(self.file, " = ")?;
		self.indent_increase();
		let mut is_first = true;
		for (field_name, field) in &ver_type.ver_type.fields {
			if !is_first {
				writeln!(self.file)?;
				self.write_indent()?;
				write!(self.file, "| ")?;
			}
			else {
				is_first = false;
			}
			write!(self.file, "{{ readonly tag: \"{}\", readonly {}: ", field_name, make_field_name(field_name))?;
			self.write_type(&self.build_type(&ver_type.version, &field.field_type)?)?;
			write!(self.file, ", }}")?;
		}
		if is_first {
			write!(self.file, "never")?;
		}
		self.indent_decrease();

		writeln!(self.file, ";")?;
		

		Ok(())
	}
}


pub struct TypeScriptLanguage {}

impl Language for TypeScriptLanguage {
	type OptionsBuilder = TSOptionsBuilder;
	type Options = TSOptions;

	fn empty_options() -> TSOptionsBuilder {
		TSOptionsBuilder {
			output_dir: None,
			package_mapping: HashMap::new(),
			library_mapping: HashMap::new(),
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

			if builder.library_mapping.contains_key(&package) || builder.package_mapping.insert(package, value).is_some() {
				return Err(GeneratorError::from(format!("Package already mapped: {}", pkg)))
			}
			Ok(())
		}
		else if let Some(pkg) = name.strip_prefix("lib:") {
			let package = model::PackageName::from_str(pkg);

			if builder.package_mapping.contains_key(&package) || builder.library_mapping.insert(package, value).is_some() {
				return Err(GeneratorError::from(format!("Package already mapped: {}", pkg)))
			}
			Ok(())
		}
		else {
			Err(GeneratorError::from(format!("Unknown option: {}", name)))
		}
	}

	fn finalize_options(builder: Self::OptionsBuilder) -> Result<Self::Options, GeneratorError> {
		Ok(TSOptions {
			output_dir: builder.output_dir.ok_or("Output directory not specified")?,
			package_mapping: builder.package_mapping,
			library_mapping: builder.library_mapping,
		})
	}

	fn generate<Output: OutputHandler>(model: &model::Verilization, options: Self::Options, output: &mut Output) -> Result<(), GeneratorError> {
		for constant in model.constants() {
			if options.library_mapping.contains_key(&constant.name().package) {
				continue;
			}

			let mut const_gen = TSConstGenerator::open(model, &options, output, constant)?;
			const_gen.generate()?;
		}

		for t in model.types() {
			if options.library_mapping.contains_key(&t.name().package) {
				continue;
			}

			match t {
				model::NamedTypeDefinition::StructType(t) => {
					let mut type_gen: TSTypeGenerator<_, GenStructType> = TSTypeGenerator::open(model, &options, output, t)?;
					type_gen.generate()?;		
				},
				model::NamedTypeDefinition::EnumType(t) => {
					let mut type_gen: TSTypeGenerator<_, GenEnumType> = TSTypeGenerator::open(model, &options, output, t)?;
					type_gen.generate()?;		
				},
				model::NamedTypeDefinition::ExternType(_) => (),
			}
		}

		Ok(())
	}

}
