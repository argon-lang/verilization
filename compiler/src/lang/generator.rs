use crate::lang::GeneratorError;
use crate::model;

use model::Named;

use std::collections::HashMap;
use num_bigint::{BigUint, BigInt};
use num_traits::One;
use std::io::Write;
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug)]
pub enum VersionedTypeKind {
	Struct,
	Enum,
}

#[derive(Clone, Debug)]
pub enum LangType<'model> {
	Versioned(VersionedTypeKind, &'model model::QualifiedName, BigUint, Vec<LangType<'model>>, LangVerTypeFields<'model>),
	Extern(&'model model::QualifiedName, Vec<LangType<'model>>, LangExternTypeLiterals<'model>),
	Interface(&'model model::QualifiedName, BigUint, Vec<LangType<'model>>, LangInterfaceMethods<'model>),
	TypeParameter(String),
	Converter(Box<LangType<'model>>, Box<LangType<'model>>),
	Codec(Box<LangType<'model>>),
	RemoteObjectId,
	RemoteConnection,
}

impl <'model> LangType<'model> {
	fn is_same_type(&self, other: &LangType<'model>) -> bool {
		match (self, other) {
			(LangType::Versioned(_, name1, _, args1, _), LangType::Versioned(_, name2, _, args2, _)) =>
				name1 == name2 &&
					args1.len() == args2.len() &&
					args1.iter().zip(args2.iter()).all(|(arg1, arg2)| arg1.is_same_type(arg2)),

			(LangType::Extern(name1, args1, _), LangType::Extern(name2, args2, _)) =>
				name1 == name2 &&
					args1.len() == args2.len() &&
					args1.iter().zip(args2.iter()).all(|(arg1, arg2)| arg1.is_same_type(arg2)),

			(LangType::TypeParameter(name1), LangType::TypeParameter(name2)) =>
				name1 == name2,

			(LangType::Converter(from1, to1), LangType::Converter(from2, to2)) =>
				from1.is_same_type(from2) && to1.is_same_type(to2),

			(LangType::Codec(elem1), LangType::Codec(elem2)) =>
				elem1.is_same_type(elem2),

			_ => false
		}
	}

	fn is_final_in_version(&self, version: &BigUint, model: &'model model::Verilization) -> bool {
		match self {
			LangType::Versioned(_, name, _, args, _) => {
				let t = match model.get_type(name) {
					Some(model::NamedTypeDefinition::StructType(t)) => t,
					Some(model::NamedTypeDefinition::EnumType(t)) => t,
					_ => panic!("Unexpected resolved type for given language type."),
				};

				t.is_final() &&
					t.last_explicit_version().iter().all(|last_ver| last_ver <= &version) &&
					args.iter().all(|arg| arg.is_final_in_version(version, model))
			},

			LangType::Extern(_, args, _) =>
				args.iter().all(|arg| arg.is_final_in_version(version, model)),

			LangType::Interface(name, _, args, _) => {
				match model.get_type(name) {
					Some(model::NamedTypeDefinition::InterfaceType(t)) => {
						return t.is_final() &&
							t.last_explicit_version().iter().all(|last_ver| last_ver <= &version) &&
							args.iter().all(|arg| arg.is_final_in_version(version, model))
					},
					_ => panic!("Unexpected resolved type for given language type."),
				}
			},

			LangType::TypeParameter(_) => true,

			LangType::RemoteObjectId | LangType::RemoteConnection | LangType::Converter(..) | LangType::Codec(_) => false,
		}
	}
}

pub struct LangField<'model> {
	pub name: &'model String,
	pub field_type: LangType<'model>,
}

impl <'model> Clone for LangField<'model> {
	fn clone(&self) -> Self {
		LangField {
			name: self.name,
			field_type: self.field_type.clone(),
		}
	}
}

pub enum LangLiteral<'model> {
	Integer(model::ExternLiteralIntBound, Option<BigInt>, model::ExternLiteralIntBound, Option<BigInt>),
	String,
	Sequence(LangType<'model>),
	Case(String, Vec<LangType<'model>>),
	Record(Vec<LangField<'model>>),
}

#[derive(Clone)]
pub struct LangInterfaceMethod<'model> {
	pub name: &'model String,
	pub type_params: &'model Vec<String>,
	pub parameters: Vec<LangInterfaceMethodParameter<'model>>,
	pub return_type: LangType<'model>,
}

#[derive(Clone)]
pub struct LangInterfaceMethodParameter<'model> {
	pub name: &'model String,
	pub param_type: LangType<'model>,
}

pub struct LangVerTypeFields<'model> {
	model: &'model model::Verilization,
	type_args: HashMap<String, LangType<'model>>,
	type_def: Named<'model, model::VersionedTypeDefinitionData>,
	ver_type: model::TypeVersionInfo<&'model model::TypeVersionDefinition>,
}

impl <'model> Clone for LangVerTypeFields<'model> {
	fn clone(&self) -> Self {
		LangVerTypeFields {
			model: self.model,
			type_args: self.type_args.clone(),
			type_def: self.type_def,
			ver_type: self.ver_type.clone(),
		}
	}
}

impl <'model> std::fmt::Debug for LangVerTypeFields<'model> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.debug_struct("LangVerTypeFields").finish()
	}
}

impl <'model> LangVerTypeFields<'model> {
	pub fn build(&self) -> Result<Vec<LangField<'model>>, GeneratorError> {
		let scope = self.type_def.scope();
		let mut fields = Vec::new();

		for (name, field) in self.ver_type.ver_type.fields() {
			let t = build_type_impl(self.model, &self.ver_type.version, &field.field_type, &scope, &self.type_args)?;
			
			fields.push(LangField {
				name: &name,
				field_type: t,
			});
		}

		Ok(fields)
	}
}

pub struct LangExternTypeLiterals<'model> {
	model: &'model model::Verilization,
	type_args: HashMap<String, LangType<'model>>,
	type_def: Named<'model, model::ExternTypeDefinitionData>,
}

impl <'model> Clone for LangExternTypeLiterals<'model> {
	fn clone(&self) -> Self {
		LangExternTypeLiterals {
			model: self.model,
			type_args: self.type_args.clone(),
			type_def: self.type_def,
		}
	}
}

impl <'model> std::fmt::Debug for LangExternTypeLiterals<'model> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.debug_struct("LangExternTypeLiterals").finish()
	}
}

impl <'model> LangExternTypeLiterals<'model> {
	pub fn build(self) -> Result<Vec<LangLiteral<'model>>, GeneratorError> {
		let scope = self.type_def.scope();
		let mut fields = Vec::new();

		for literal in self.type_def.literals() {
			let lang_literal = match literal {
				model::ExternLiteralSpecifier::Integer(lower_type, lower, upper_type, upper) => LangLiteral::Integer(*lower_type, lower.clone(), *upper_type, upper.clone()),
				model::ExternLiteralSpecifier::String => LangLiteral::String,
				model::ExternLiteralSpecifier::Sequence(t) => LangLiteral::Sequence(build_type_impl(self.model, &BigUint::one(), t, &scope, &self.type_args)?),
				model::ExternLiteralSpecifier::Case(name, params) =>
					LangLiteral::Case(name.clone(), params.iter().map(|param| build_type_impl(self.model, &BigUint::one(), param, &scope, &self.type_args)).collect::<Result<Vec<_>, _>>()?),
				model::ExternLiteralSpecifier::Record(fields) => {
					let mut lang_fields = Vec::new();

					for (name, field) in fields {
						let t = build_type_impl(self.model, &BigUint::one(), &field.field_type, &scope, &self.type_args)?;
						
						lang_fields.push(LangField {
							name: &name,
							field_type: t,
						});
					}

					LangLiteral::Record(lang_fields)
				}
			};
			
			fields.push(lang_literal);
		}

		Ok(fields)
	}
}

pub struct LangInterfaceMethods<'model> {
	model: &'model model::Verilization,
	type_args: HashMap<String, LangType<'model>>,
	type_def: Named<'model, model::InterfaceTypeDefinitionData>,
	ver_type: model::TypeVersionInfo<model::OfInterface<'model, model::InterfaceVersionDefinition>>,
}

impl <'model> Clone for LangInterfaceMethods<'model> {
	fn clone(&self) -> Self {
		LangInterfaceMethods {
			model: self.model,
			type_args: self.type_args.clone(),
			type_def: self.type_def,
			ver_type: self.ver_type.clone(),
		}
	}
}

impl <'model> std::fmt::Debug for LangInterfaceMethods<'model> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.debug_struct("LangInterfaceMethods").finish()
	}
}

impl <'model> LangInterfaceMethods<'model> {
	pub fn build(self) -> Result<Vec<LangInterfaceMethod<'model>>, GeneratorError> {
		let mut methods = Vec::new();
		
		for (name, method) in self.ver_type.ver_type.methods() {
			let scope = method.scope();

			let mut type_args = self.type_args.clone();
			for type_param in method.type_params() {
				type_args.insert(type_param.clone(), LangType::TypeParameter(type_param.clone()));
			}


			let mut parameters = Vec::new();
			for param in method.parameters() {
				let param_type = build_type_impl(self.model, &self.ver_type.version, &param.param_type, &scope, &type_args)?;
				parameters.push(LangInterfaceMethodParameter {
					name: &param.name,
					param_type,
				});
			}

			let return_type = build_type_impl(self.model, &self.ver_type.version, method.return_type(), &scope, &type_args)?;

			methods.push(LangInterfaceMethod {
				name,
				type_params: method.type_params(),
				parameters,
				return_type,
			});
		}

		Ok(methods)
	}
}


#[derive(Debug)]
pub enum Operation {
	FromPreviousVersion(BigUint),
	FinalTypeConverter,
	TypeCodec,
	FromInteger,
	FromString,
	FromSequence,
	FromCase(String),
	FromRecord(Vec<String>),
	CreateRemoteWrapper,
}

#[derive(Debug)]
pub enum OperationTarget<'model> {
	VersionedType(&'model model::QualifiedName, BigUint),
	ExternType(&'model model::QualifiedName),
	InterfaceType(&'model model::QualifiedName, BigUint),
}

#[derive(Debug)]
pub enum LangExpr<'model> {
	Identifier(String),
	IntegerLiteral(BigInt),
	StringLiteral(String),
	InvokeConverter {
		converter: Box<LangExpr<'model>>,
		value: Box<LangExpr<'model>>,
	},
	IdentityConverter(LangType<'model>),
	ReadDiscriminator,
	WriteDiscriminator(BigUint),
	CodecRead {
		codec: Box<LangExpr<'model>>,
	},
	CodecWrite {
		codec: Box<LangExpr<'model>>,
		value: Box<LangExpr<'model>>,
	},
	InvokeOperation(Operation, OperationTarget<'model>, Vec<LangType<'model>>, Vec<LangExpr<'model>>),
	InvokeUserConverter {
		name: &'model model::QualifiedName,
		prev_ver: BigUint,
		version: BigUint,
		type_args: Vec<LangType<'model>>,
		args: Vec<LangExpr<'model>>,
	},
	ConstantValue(&'model model::QualifiedName, BigUint),
	CreateStruct(&'model model::QualifiedName, BigUint, Vec<LangType<'model>>, Vec<(&'model String, LangExpr<'model>)>),
	CreateEnum(&'model model::QualifiedName, BigUint, Vec<LangType<'model>>, &'model String, Box<LangExpr<'model>>),
	StructField(&'model model::QualifiedName, BigUint, &'model String, Box<LangExpr<'model>>),
	ReadRemoteObject {
		object_type_target: OperationTarget<'model>,
		connection: Box<LangExpr<'model>>,
	},
	WriteRemoteObject {
		object: Box<LangExpr<'model>>,
		connection: Box<LangExpr<'model>>,
	},
}

pub struct OperationInfo<'model> {
	pub operation: Operation,
	pub version: BigUint,
	pub type_params: Vec<String>,
	pub params: Vec<(String, LangType<'model>)>,
	pub result: LangType<'model>,
	pub implementation: LangExprStmt<'model>,
}

pub struct MatchCase<'model> {
	pub binding_name: String,
	pub case_name: String,
	pub body: LangStmt<'model>,
}

pub enum LangExprStmt<'model> {
	Expr(LangExpr<'model>),
	CreateCodec {
		t: LangType<'model>,
		read: Box<LangStmt<'model>>,
		write: Box<LangStmt<'model>>,
	},
	CreateConverter {
		from_type: LangType<'model>,
		to_type: LangType<'model>,
		body: Box<LangStmt<'model>>,
	},
	CreateRemoteWrapper {
		t: LangType<'model>,
		connection: LangExpr<'model>,
		id: LangExpr<'model>,
		methods: Vec<LangInterfaceMethod<'model>>,
	},
}

pub enum LangStmt<'model> {
	Expr(Vec<LangExpr<'model>>, Option<LangExpr<'model>>),
	MatchEnum {
		value: LangExpr<'model>,
		value_type: LangType<'model>,
		cases: Vec<MatchCase<'model>>,
	},
	MatchDiscriminator {
		value: LangExpr<'model>,
		cases: Vec<(BigUint, LangStmt<'model>)>,
	},
}

impl <'model> LangStmt<'model> {
	pub fn has_value(&self) -> bool {
		match self {
			LangStmt::Expr(_, None) => false,
			LangStmt::Expr(_, Some(_)) => true,
			LangStmt::MatchEnum { cases, .. } => cases.iter().any(|MatchCase { body, .. }| body.has_value()),
			LangStmt::MatchDiscriminator { cases, .. } => cases.iter().any(|(_, body)| body.has_value()),
		}
	}
}

pub enum ConvertParam<'model> {
	ConverterObject,
	Expression(LangExpr<'model>),
}

fn requires_conversion<'model, G: Generator<'model>>(gen: &G, t: &model::Type, prev_ver: &BigUint) -> bool {
	match gen.scope().lookup(t.name.clone()) {
		model::ScopeLookup::NamedType(name) => match gen.model().get_type(&name) {
			Some(model::NamedTypeDefinition::StructType(type_def)) | Some(model::NamedTypeDefinition::EnumType(type_def)) => {
				!type_def.is_final() ||
					(match type_def.last_explicit_version() {
						Some(last_ver) => last_ver > prev_ver,
						None => true
					}) ||
					t.args.iter().any(|arg| requires_conversion(gen, arg, prev_ver))
			},

			Some(model::NamedTypeDefinition::ExternType(_)) => false,
			Some(model::NamedTypeDefinition::InterfaceType(_)) => false,

			None => true, // Error condition, assume conversion required. Should fail when determining the conversion.
		},
		model::ScopeLookup::TypeParameter(_) => true,
	}
}

pub trait GeneratorNameMapping {
	fn convert_prev_type_param(param: &str) -> String;
	fn convert_current_type_param(param: &str) -> String;
	fn convert_conv_param_name(param: &str) -> String;
	fn convert_prev_param_name() -> &'static str;

	fn codec_write_value_name() -> &'static str;
	fn codec_codec_param_name(param: &str) -> String;

	fn format_writer_name() -> &'static str;
	fn format_reader_name() -> &'static str;
	fn connection_name() -> &'static str;
	fn object_id_name() -> &'static str;
	
	fn constant_version_name(version: &BigUint) -> String;
}


fn build_type_impl<'model>(model: &'model model::Verilization, version: &BigUint, t: &model::Type, scope: &model::Scope<'model>, type_args: &HashMap<String, LangType<'model>>) -> Result<LangType<'model>, GeneratorError> {
	let lang_args = t.args.iter()
		.map(|arg| build_type_impl(model, version, arg, scope, type_args))
		.collect::<Result<Vec<_>, _>>()?;

	Ok(match scope.lookup(t.name.clone()) {
		model::ScopeLookup::NamedType(name) => match model.get_type(&name).ok_or_else(|| GeneratorError::CouldNotFind(name.clone()))? {
			model::NamedTypeDefinition::StructType(type_def) => {
				let ver_type = type_def.versioned(version).ok_or_else(|| GeneratorError::CouldNotFindVersion(name, version.clone()))?;
				let type_ver = ver_type.version.clone();

				if type_def.type_params().len() != lang_args.len() {
					return Err(GeneratorError::ArityMismatch(type_def.type_params().len(), lang_args.len()));
				}

				let lang_args_map = type_def.type_params().clone().into_iter()
					.zip(lang_args.clone().into_iter())
					.collect::<HashMap<_, _>>();

				let fields = LangVerTypeFields {
					model: model,
					type_args: lang_args_map,
					type_def: type_def,
					ver_type: ver_type,
				};

				LangType::Versioned(VersionedTypeKind::Struct, type_def.name(), type_ver, lang_args, fields)
			},

			model::NamedTypeDefinition::EnumType(type_def) => {
				let lang_args_map = type_def.type_params().clone().into_iter()
					.zip(lang_args.clone().into_iter())
					.collect::<HashMap<_, _>>();

				let ver_type = type_def.versioned(version).ok_or_else(|| GeneratorError::CouldNotFindVersion(name, version.clone()))?;
				let type_ver = ver_type.version.clone();

				let fields = LangVerTypeFields {
					model: model,
					type_args: lang_args_map,
					type_def: type_def,
					ver_type: ver_type,
				};

				LangType::Versioned(VersionedTypeKind::Enum, type_def.name(), type_ver, lang_args, fields)
			},

			model::NamedTypeDefinition::ExternType(type_def) => {
				let lang_args_map = type_def.type_params().clone().into_iter()
					.zip(lang_args.clone().into_iter())
					.collect::<HashMap<_, _>>();

				let literals = LangExternTypeLiterals {
					model,
					type_args: lang_args_map,
					type_def,
				};

				LangType::Extern(type_def.name(), lang_args, literals)
			},

			model::NamedTypeDefinition::InterfaceType(type_def) => {
				let lang_args_map = type_def.type_params().clone().into_iter()
					.zip(lang_args.clone().into_iter())
					.collect::<HashMap<_, _>>();

				let ver_type = type_def.versioned(version).ok_or_else(|| GeneratorError::CouldNotFindVersion(name, version.clone()))?;
				let type_ver = ver_type.version.clone();

				let methods = LangInterfaceMethods {
					model,
					type_args: lang_args_map,
					type_def,
					ver_type,
				};

				LangType::Interface(type_def.name(), type_ver, lang_args, methods)
			},
		},
		model::ScopeLookup::TypeParameter(name) => {
			if !lang_args.is_empty() {
				return Err(GeneratorError::ArityMismatch(0, lang_args.len()));
			}

			type_args.get(&name).ok_or_else(|| GeneratorError::CouldNotResolveTypeParameter(name))?.clone()
		},
	})
}

fn is_valid_integer_for_type(n: &BigInt, literals: LangExternTypeLiterals) -> Result<bool, GeneratorError> {
	Ok(literals.build()?.iter().any(|literal| match literal {
		LangLiteral::Integer(lower_bound, lower_value, upper_bound, upper_value) =>
			(
				match (lower_bound, lower_value) {
					(_, None) => true,
					(model::ExternLiteralIntBound::Inclusive, Some(x)) => n >= x,
					(model::ExternLiteralIntBound::Exclusive, Some(x)) => n > x,
				}
			) && (
				match (upper_bound, upper_value) {
					(_, None) => true,
					(model::ExternLiteralIntBound::Inclusive, Some(x)) => n <= x,
					(model::ExternLiteralIntBound::Exclusive, Some(x)) => n < x,
				}
			),

		_ => false,
	}))
}

pub trait Generator<'model> : Sized {
	type Lang: GeneratorNameMapping;

	fn model(&self) -> &'model model::Verilization;
	fn scope(&self) -> &model::Scope<'model>;


	fn build_type<'gen>(&'gen self, version: &BigUint, t: &model::Type) -> Result<LangType<'model>, GeneratorError> {
		let type_args = self.scope().type_params().into_iter().map(|param| (param.clone(), LangType::TypeParameter(param.clone()))).collect::<HashMap<_, _>>();

		build_type_impl(self.model(), version, t, self.scope(), &type_args)
	}

	fn build_codec(&self, t: LangType<'model>) -> Result<LangExpr<'model>, GeneratorError> {
		Ok(match t {
			LangType::Versioned(_, name, version, args, _) => {
				let codec_args = args.iter().map(|arg| self.build_codec(arg.clone())).collect::<Result<Vec<_>, _>>()?;

				LangExpr::InvokeOperation(
					Operation::TypeCodec,
					OperationTarget::VersionedType(name, version),
					args,
					codec_args,
				)
			},

			LangType::Extern(name, args, _) => {
				let codec_args = args.iter().map(|arg| self.build_codec(arg.clone())).collect::<Result<Vec<_>, _>>()?;

				LangExpr::InvokeOperation(
					Operation::TypeCodec,
					OperationTarget::ExternType(name),
					args,
					codec_args,
				)
			},

			LangType::Interface(name, version, args, _) => {
				let mut codec_args = vec!(
					LangExpr::Identifier(Self::Lang::object_id_name().to_string())
				);

				for arg in &args {
					codec_args.push(self.build_codec(arg.clone())?);
				}


				LangExpr::InvokeOperation(
					Operation::TypeCodec,
					OperationTarget::InterfaceType(name, version),
					args,
					codec_args,
				)
			},

			LangType::TypeParameter(name) => LangExpr::Identifier(Self::Lang::codec_codec_param_name(&name)),

			LangType::RemoteObjectId | LangType::RemoteConnection | LangType::Codec(_) | LangType::Converter(_, _) => return Err(GeneratorError::InvalidTypeForCodec),
		})
	}

	fn build_conversion(&self, prev_ver: &BigUint, version: &BigUint, t: &model::Type, param: ConvertParam<'model>) -> Result<LangExpr<'model>, GeneratorError> {
		if !requires_conversion(self, t, prev_ver) {
			return Ok(match param {
				ConvertParam::ConverterObject => LangExpr::IdentityConverter(self.build_type(version, t)?),
				ConvertParam::Expression(expr) => expr,
			})
		}

		let converter = match self.scope().lookup(t.name.clone()) {
			model::ScopeLookup::NamedType(name) => {

				let mut op_type_args = Vec::new();
				let mut op_args = Vec::new();

				for arg in &t.args {
					op_type_args.push(self.build_type(prev_ver, arg)?);
					op_type_args.push(self.build_type(version, arg)?);
					op_args.push(self.build_conversion(prev_ver, version, arg, ConvertParam::ConverterObject)?);
				}


				let named_type_def = self.model().get_type(&name).ok_or_else(|| GeneratorError::CouldNotFind(name.clone()))?;
				let operation;
				let target;
				match named_type_def {
					model::NamedTypeDefinition::StructType(type_def) | model::NamedTypeDefinition::EnumType(type_def) => {
						let ver_type = type_def.versioned(version).ok_or_else(|| GeneratorError::CouldNotFindVersion(name.clone(), version.clone()))?;

						operation =
							if ver_type.version < *version {
								Operation::FinalTypeConverter
							}
							else {
								Operation::FromPreviousVersion(prev_ver.clone())
							};

						target = OperationTarget::VersionedType(named_type_def.name(), ver_type.version.clone());
					},

					model::NamedTypeDefinition::ExternType(_) => {
						operation = Operation::FinalTypeConverter;
						target = OperationTarget::ExternType(named_type_def.name());
					},

					model::NamedTypeDefinition::InterfaceType(_) => panic!("Cannot convert interface type to newer version."),
				};

				LangExpr::InvokeOperation(
					operation,
					target,
					op_type_args,
					op_args
				)
			},
			model::ScopeLookup::TypeParameter(name) => LangExpr::Identifier(Self::Lang::convert_conv_param_name(&name)),
		};

					
		Ok(match param {
			ConvertParam::ConverterObject => converter,
			ConvertParam::Expression(expr) => LangExpr::InvokeConverter {
				converter: Box::new(converter),
				value: Box::new(expr),
			},
		})
	}

	fn build_value(&self, version: &BigUint, t: LangType<'model>, value: model::ConstantValue) -> Result<LangExpr<'model>, GeneratorError> {
		Ok(match value {
			model::ConstantValue::Integer(n) =>
				match t {
					LangType::Extern(name, args, literals) =>
						if !is_valid_integer_for_type(&n, literals)? {
							return Err(GeneratorError::InvalidTypeForIntValue)
						}
						else {
							LangExpr::InvokeOperation(Operation::FromInteger, OperationTarget::ExternType(name), args, vec!(LangExpr::IntegerLiteral(n)))
						},

					_ => return Err(GeneratorError::InvalidTypeForIntValue),
				},

			model::ConstantValue::String(s) =>
				match t {
					LangType::Extern(name, args, literals) =>
						if !literals.build()?.iter().any(|literal| match literal { LangLiteral::String => true, _ => false, }) {
							return Err(GeneratorError::InvalidTypeForString)
						}
						else {
							LangExpr::InvokeOperation(Operation::FromString, OperationTarget::ExternType(name), args, vec!(LangExpr::StringLiteral(s)))
						},

					_ => return Err(GeneratorError::InvalidTypeForString),
				},

			model::ConstantValue::Sequence(seq) => match t {
				LangType::Extern(type_name, type_args, literals) => {
					let literals = literals.build()?;

					let element_type = literals
						.into_iter()
						.find_map(|literal| match literal {
							LangLiteral::Sequence(element_type) => Some(element_type),
							_ => None,
						})
						.ok_or_else(|| GeneratorError::TypeCannotBeSequence(type_name.clone()))?;

					let args = seq.into_iter()
						.map(|elem| self.build_value(version, element_type.clone(), elem))
						.collect::<Result<Vec<_>, _>>()?;

					LangExpr::InvokeOperation(
						Operation::FromSequence,
						OperationTarget::ExternType(type_name),
						type_args,
						args,
					)
				},
				LangType::Versioned(_, type_name, ..) => return Err(GeneratorError::TypeCannotBeSequence(type_name.clone())),
				LangType::RemoteObjectId | LangType::RemoteConnection |
				LangType::Interface(..) | LangType::TypeParameter(_) |
				LangType::Codec(_) | LangType::Converter(_, _) =>
					return Err(GeneratorError::InvalidTypeForConstant),
			},
			
			model::ConstantValue::Case(case_name, mut args) => match t {
				LangType::Versioned(VersionedTypeKind::Enum, type_name, type_version, type_args, fields) if args.len() == 1 => {
					let field = fields.build()?.into_iter().find(|field| *field.name == case_name).ok_or_else(|| GeneratorError::TypeDoesNotHaveCase(type_name.clone(), Some(type_version.clone()), case_name.clone()))?;
					
					let arg = self.build_value(version, field.field_type, args.remove(0))?;
					LangExpr::CreateEnum(type_name, type_version, type_args, field.name, Box::new(arg))
				},
				LangType::Versioned(VersionedTypeKind::Enum, type_name, ..) => return Err(GeneratorError::IncorrectCaseArity(type_name.clone(), case_name.clone())),

				LangType::Extern(type_name, type_args, literals) => {
					let case_params = literals.build()?
						.into_iter()
						.find_map(|literal| match literal {
							LangLiteral::Case(name, case_params) if name == *case_name => Some(case_params),
							_ => None,
						})
						.ok_or_else(|| GeneratorError::TypeDoesNotHaveCase(type_name.clone(), None, case_name.clone()))?;

					let args = args.into_iter().zip(case_params.into_iter())
						.map(|(arg, param)| self.build_value(version, param, arg))
						.collect::<Result<Vec<_>, _>>()?;

					LangExpr::InvokeOperation(
						Operation::FromCase(case_name.clone()),
						OperationTarget::ExternType(type_name),
						type_args,
						args,
					)
				},

				LangType::Versioned(VersionedTypeKind::Struct, ..) => return Err(GeneratorError::RecordLiteralNotForStruct),
				LangType::RemoteObjectId | LangType::RemoteConnection |
				LangType::Interface(..) | LangType::TypeParameter(_) |
				LangType::Codec(_) | LangType::Converter(_, _) =>
					return Err(GeneratorError::InvalidTypeForConstant),
			},
			
			model::ConstantValue::Record(record) => match t {
				LangType::Versioned(VersionedTypeKind::Struct, type_name, type_version, type_args, fields) => {
					let mut lang_args = Vec::new();
					let mut field_values = record.into_field_values();

					for field in fields.build()? {
						let value = field_values.remove(field.name).ok_or_else(|| GeneratorError::CouldNotFindRecordField(type_name.clone(), Some(type_version.clone()), field.name.clone()))?;
						let value = self.build_value(version, field.field_type, value)?;
						lang_args.push((field.name, value));
					}

					LangExpr::CreateStruct(type_name, type_version, type_args, lang_args)
				},

				LangType::Extern(type_name, type_args, literals) => {
					let record_fields = literals.build()?
						.into_iter()
						.find_map(|literal| match literal {
							LangLiteral::Record(fields) => Some(fields),
							_ => None,
						})
						.ok_or_else(|| GeneratorError::ExternTypeDoesNotHaveRecordLiteral(type_name.clone()))?;

					let mut field_values = record.into_field_values();
					let mut field_names = Vec::new();
					let mut args = Vec::new();

					for field in record_fields {
						let value = field_values.remove(field.name).ok_or_else(|| GeneratorError::CouldNotFindRecordField(type_name.clone(), None, field.name.clone()))?;
						let value = self.build_value(version, field.field_type, value)?;
						field_names.push(field.name.clone());
						args.push(value);
					}

					LangExpr::InvokeOperation(
						Operation::FromRecord(field_names),
						OperationTarget::ExternType(type_name),
						type_args,
						args,
					)
				},

				LangType::Versioned(VersionedTypeKind::Enum, ..) => return Err(GeneratorError::InvalidTypeForConstant),
				LangType::RemoteObjectId | LangType::RemoteConnection |
				LangType::Interface(..) | LangType::TypeParameter(_) |
				LangType::Codec(_) | LangType::Converter(_, _) => return Err(GeneratorError::InvalidTypeForConstant),
			},
			model::ConstantValue::Constant(name) => {
				match self.scope().lookup(name) {
					model::ScopeLookup::NamedType(name) => {
						let constant = self.model().get_constant(&name).ok_or_else(|| GeneratorError::CouldNotFind(name.clone()))?;

						if !constant.has_version(version) {
							return Err(GeneratorError::CouldNotFindVersion(name, version.clone()))
						}
						
						let const_type = self.build_type(version, constant.value_type())?;

						if !t.is_same_type(&const_type) {
							return Err(GeneratorError::TypeMismatch)
						}

						LangExpr::ConstantValue(constant.name(), version.clone())
					},
					model::ScopeLookup::TypeParameter(name) => return Err(GeneratorError::CouldNotFind(model::QualifiedName { package: model::PackageName::new(), name: name, }))
				}
				
			},
		})
	}
	
}

#[derive(Default)]
pub struct GenConstant {}

#[derive(Default)]
pub struct GenType<GenTypeKind> {
	type_gen: PhantomData<GenTypeKind>,
}

pub trait ConstGenerator<'model> : Generator<'model> {
	fn constant(&self) -> Named<'model, model::Constant>;

	fn write_header(&mut self) -> Result<(), GeneratorError>;
	fn write_constant(&mut self, version_name: String, t: LangType<'model>, value: LangExpr<'model>) -> Result<(), GeneratorError>;
	fn write_footer(&mut self) -> Result<(), GeneratorError>;


	fn generate(&mut self) -> Result<(), GeneratorError> {
		self.write_header()?;

		for ver in self.constant().versions() {
			let version_name = Self::Lang::constant_version_name(&ver.version);
			let t = self.build_type(&ver.version, self.constant().value_type())?;
			let value = self.build_value(&ver.version, t.clone(), ver.value.clone())?;

			self.write_constant(version_name, t, value)?;
		}

		self.write_footer()
	}

	fn build_value_from_prev(&self, prev_ver: &BigUint, version: &BigUint, t: &model::Type) -> Result<LangExpr<'model>, GeneratorError> {
		self.build_conversion(prev_ver, version, t, ConvertParam::Expression(LangExpr::ConstantValue(self.constant().name(), prev_ver.clone())))
	}



}

pub trait TypeGenerator<'model> : Generator<'model> {
	type TypeDefinition : 'model + model::GeneratableType<'model>;
	fn type_def(&self) -> Named<'model, Self::TypeDefinition>;

	fn write_header(&mut self) -> Result<(), GeneratorError>;
	fn write_version_header(&mut self, t: LangType<'model>) -> Result<(), GeneratorError>;
	fn write_operation(&mut self, operation: OperationInfo<'model>) -> Result<(), GeneratorError>;
	fn write_version_footer(&mut self) -> Result<(), GeneratorError>;
	fn write_footer(&mut self) -> Result<(), GeneratorError>;
}

pub trait TypeGeneratorOperations<'model, TypeDef : 'model + model::GeneratableType<'model>> : TypeGenerator<'model, TypeDefinition = TypeDef> {
	type OperationsState;

	fn prepare_operaitons_state(&self, t: &LangType<'model>, ver_type: &model::TypeVersionInfo<TypeDef::TypeVersionRef>, prev_ver: &Option<BigUint>) -> Result<Self::OperationsState, GeneratorError>;
	fn generate_operations(&mut self, state: Self::OperationsState, ver_type: &model::TypeVersionInfo<TypeDef::TypeVersionRef>, prev_ver: &Option<BigUint>) -> Result<(), GeneratorError>;
}

pub trait TypeGeneratorGenerate<'model> {
	fn generate(&mut self) -> Result<(), GeneratorError>;
}

impl <'model, TypeDef : 'model + model::GeneratableType<'model>, TImpl : TypeGenerator<'model, TypeDefinition = TypeDef> + TypeGeneratorOperations<'model, TypeDef>> TypeGeneratorGenerate<'model> for TImpl {
	fn generate(&mut self) -> Result<(), GeneratorError> {
		self.write_header()?;

		let mut prev_ver = None;
		
		for ver_type in self.type_def().versions() {
			let version = &ver_type.version;

			let type_params_as_args = build_type_params_as_args(self.type_def());

			let t = self.build_type(version, &model::Type { name: self.type_def().name().clone(), args: type_params_as_args })?;

			let op_state = self.prepare_operaitons_state(&t, &ver_type, &prev_ver)?;

			self.write_version_header(t)?;

			self.generate_operations(op_state, &ver_type, &prev_ver)?;


			self.write_version_footer()?;
			prev_ver = Some(version.clone());
		}

		self.write_footer()
	}
}

fn build_type_params_as_args<'model, TypeDef : model::GeneratableType<'model>>(type_def: Named<'model, TypeDef>) -> Vec<model::Type> {
	type_def.type_params().iter()
		.map(|param| model::Type { name: model::QualifiedName::from_parts(&[], &param), args: vec!() })
		.collect::<Vec<_>>()
}

pub struct VersionedTypeGeneratorOperationsState {
	is_final_last_version: bool,
	type_kind: VersionedTypeKind,
}

impl <'model, TImpl : TypeGenerator<'model, TypeDefinition = model::VersionedTypeDefinitionData>> TypeGeneratorOperations<'model, model::VersionedTypeDefinitionData> for TImpl {
	type OperationsState = VersionedTypeGeneratorOperationsState;

	fn prepare_operaitons_state(&self, t: &LangType<'model>, ver_type: &model::TypeVersionInfo<&'model model::TypeVersionDefinition>, _prev_ver: &Option<BigUint>) -> Result<Self::OperationsState, GeneratorError> {
		let version = &ver_type.version;

		Ok(VersionedTypeGeneratorOperationsState {
			is_final_last_version: t.is_final_in_version(version, self.model()),
			type_kind: match &t {
				LangType::Versioned(kind, ..) => *kind,
				_ => return Err(GeneratorError::CouldNotGenerateType),
			}
		})
	}
	fn generate_operations(&mut self, state: Self::OperationsState, ver_type: &model::TypeVersionInfo<&'model model::TypeVersionDefinition>, prev_ver: &Option<BigUint>) -> Result<(), GeneratorError> {
		let version = &ver_type.version;
		

		// Converter for latest version of final type with type parameters
		if state.is_final_last_version && !self.type_def().type_params().is_empty() {
			self.write_operation(build_converter_operation_common(self, Operation::FinalTypeConverter, state.type_kind, &ver_type, version)?)?;
		}
		
		// Conversion from previous version
		if let Some(prev_ver) = prev_ver { // Skip when there is no prevous version.
			self.write_operation(build_converter_operation_common(self, Operation::FromPreviousVersion(prev_ver.clone()), state.type_kind, &ver_type, &prev_ver)?)?;
		}

		// Codec
		{
			let type_params_as_args = build_type_params_as_args(self.type_def());
			let mut codec_params = Vec::new();

			for param in self.type_def().type_params() {
				let param_type = LangType::TypeParameter(param.clone());

				codec_params.push((Self::Lang::codec_codec_param_name(param), LangType::Codec(Box::new(param_type.clone()))));
			}

			let obj_type = self.build_type(version, &model::Type { name: self.type_def().name().clone(), args: type_params_as_args })?;

			let codec_type = LangType::Codec(Box::new(obj_type.clone()));

			let op = OperationInfo {
				operation: Operation::TypeCodec,
				version: version.clone(),
				type_params: self.type_def().type_params().clone(),
				params: codec_params,
				result: codec_type,
				implementation: LangExprStmt::CreateCodec {
					t: obj_type.clone(),
					read: Box::new(codec_read_implementation(self, obj_type.clone())?),
					write: Box::new(codec_write_implementation(self, obj_type)?),
				},
			};

			self.write_operation(op)?;
		}

		Ok(())
	}
}


pub struct InterfaceTypeGeneratorOperationsState<'model> {
	t: LangType<'model>,
}

impl <'model, TImpl : TypeGenerator<'model, TypeDefinition = model::InterfaceTypeDefinitionData>> TypeGeneratorOperations<'model, model::InterfaceTypeDefinitionData> for TImpl {
	type OperationsState = InterfaceTypeGeneratorOperationsState<'model>;

	fn prepare_operaitons_state(&self, t: &LangType<'model>, _ver_type: &model::TypeVersionInfo<model::OfInterface<'model, model::InterfaceVersionDefinition>>, _prev_ver: &Option<BigUint>) -> Result<Self::OperationsState, GeneratorError> {
		Ok(InterfaceTypeGeneratorOperationsState {
			t: t.clone(),
		})
	}
	fn generate_operations(&mut self, state: Self::OperationsState, ver_type: &model::TypeVersionInfo<model::OfInterface<'model, model::InterfaceVersionDefinition>>, _prev_ver: &Option<BigUint>) -> Result<(), GeneratorError> {
		let version = &ver_type.version;

		// Create Remote Wrapper
		{
			let mut params = Vec::new();

			params.push((TImpl::Lang::connection_name().to_string(), LangType::RemoteConnection));
			params.push((TImpl::Lang::object_id_name().to_string(), LangType::RemoteObjectId));

			for param in self.type_def().type_params() {
				let param_type = LangType::TypeParameter(param.clone());

				params.push((Self::Lang::codec_codec_param_name(param), LangType::Codec(Box::new(param_type.clone()))));
			}

			let methods = match &state.t {
				LangType::Interface(_, _, _, methods) => methods.clone().build()?,
				_ => return Err(GeneratorError::CouldNotGenerateType),
			};

			let op = OperationInfo {
				operation: Operation::CreateRemoteWrapper,
				version: version.clone(),
				type_params: self.type_def().type_params().clone(),
				params,
				result: state.t.clone(),
				implementation: LangExprStmt::CreateRemoteWrapper {
					t: state.t,
					connection: LangExpr::Identifier(TImpl::Lang::connection_name().to_string()),
					id: LangExpr::Identifier(TImpl::Lang::object_id_name().to_string()),
					methods,
				},
			};

			self.write_operation(op)?;
		}

		Ok(())
	}
}


fn build_converter_operation_common<'model, Gen>(gen: &Gen, op: Operation, type_kind: VersionedTypeKind, ver_type: &model::TypeVersionInfo<&'model model::TypeVersionDefinition>, prev_ver: &BigUint) -> Result<OperationInfo<'model>, GeneratorError> where
	Gen : TypeGenerator<'model>
{
	let version = &ver_type.version;

	let mut type_params_as_args = Vec::new();
	let mut type_params = Vec::new();
	let mut type_args = Vec::new();
	let mut params = Vec::new();
	let mut result_type_args = Vec::new();
	let mut prev_type_args = HashMap::new();
	let mut result_type_args_map = HashMap::new();
	let mut impl_call_args = Vec::new();

	for param in gen.type_def().type_params() {
		type_params_as_args.push(model::Type { name: model::QualifiedName::from_parts(&[], &param), args: vec!() });
		let t1 = Gen::Lang::convert_prev_type_param(&param);
		let t2 = Gen::Lang::convert_current_type_param(&param);
		type_params.push(t1.clone());
		type_params.push(t2.clone());
		let t1_arg = LangType::TypeParameter(t1.clone());
		let t2_arg = LangType::TypeParameter(t2.clone());
		type_args.push(t1_arg.clone());
		type_args.push(t2_arg.clone());
		result_type_args.push(t2_arg.clone());
		prev_type_args.insert(param.clone(), t1_arg);
		result_type_args_map.insert(param.clone(), t2_arg);

		let conv_type = LangType::Converter(
			Box::new(LangType::TypeParameter(t1)),
			Box::new(LangType::TypeParameter(t2)),
		);

		let conv_param = Gen::Lang::convert_conv_param_name(param);
		params.push((conv_param.clone(), conv_type));
		impl_call_args.push(LangExpr::Identifier(conv_param));
	}

	let prev_type = build_type_impl(gen.model(), prev_ver, &model::Type { name: gen.type_def().name().clone(), args: type_params_as_args.clone() }, gen.scope(), &prev_type_args)?;
	let result_type = build_type_impl(gen.model(), version, &model::Type { name: gen.type_def().name().clone(), args: type_params_as_args.clone() }, gen.scope(), &result_type_args_map)?;

	let converter_type = LangType::Converter(Box::new(prev_type.clone()), Box::new(result_type.clone()));

	let implementation = if ver_type.explicit_version && ver_type.version != *prev_ver {
		LangExprStmt::Expr(LangExpr::InvokeUserConverter {
			name: gen.type_def().name(),
			prev_ver: prev_ver.clone(),
			version: version.clone(),
			type_args: type_args,
			args: impl_call_args,
		})
	}
	else {
		let body = match type_kind {
			VersionedTypeKind::Struct => {
				let mut fields = Vec::new();
		
				for (field_name, field) in ver_type.ver_type.fields() {
					let obj_value = LangExpr::Identifier(Gen::Lang::convert_prev_param_name().to_string());
		
					let value_expr = LangExpr::StructField(gen.type_def().name(), ver_type.version.clone(), field_name, Box::new(obj_value));
					let conv_value = gen.build_conversion(prev_ver, &ver_type.version, &field.field_type, ConvertParam::Expression(value_expr))?;
		
					fields.push((field_name, conv_value));
				}
		
				LangStmt::Expr(vec!(),
					Some(LangExpr::CreateStruct(gen.type_def().name(), ver_type.version.clone(), result_type_args, fields))
				)
			},
			VersionedTypeKind::Enum => {
				let mut cases = Vec::new();
		
		
				for (field_name, field) in ver_type.ver_type.fields() {
		
					let value_expr = LangExpr::Identifier(field_name.clone());
					let conv_value = gen.build_conversion(prev_ver, &ver_type.version, &field.field_type, ConvertParam::Expression(value_expr))?;
					let enum_value = LangExpr::CreateEnum(gen.type_def().name(), ver_type.version.clone(), result_type_args.clone(), field_name, Box::new(conv_value));
		
					cases.push(MatchCase {
						binding_name: field_name.clone(),
						case_name: field_name.clone(),
						body: LangStmt::Expr(vec!(), Some(enum_value)),
					});
				}
		
				LangStmt::MatchEnum {
					value: LangExpr::Identifier(Gen::Lang::convert_prev_param_name().to_string()),
					value_type: build_type_impl(gen.model(), prev_ver, &model::Type { name: gen.type_def().name().clone(), args: type_params_as_args.clone() }, gen.scope(), &prev_type_args)?,
					cases: cases,
				}
			},
		};

		LangExprStmt::CreateConverter {
			from_type: prev_type,
			to_type: result_type,
			body: Box::new(body),
		}
	};

	Ok(OperationInfo {
		operation: op,
		version: version.clone(),
		type_params: type_params,
		params: params,
		result: converter_type,
		implementation: implementation,
	})
}


fn codec_read_implementation<'model, Gen>(gen: &Gen, t: LangType<'model>) -> Result<LangStmt<'model>, GeneratorError> where
	Gen : TypeGenerator<'model>
{
	Ok(match t {
		LangType::Versioned(VersionedTypeKind::Struct, _, version, type_args, fields) => {
			let mut field_values = Vec::new();
		
			for field in fields.build()? {
				let field_codec = gen.build_codec(field.field_type)?;
				field_values.push((field.name, LangExpr::CodecRead { codec: Box::new(field_codec) }));
			}
		
			LangStmt::Expr(vec!(),
				Some(LangExpr::CreateStruct(gen.type_def().name(), version, type_args, field_values))
			)
		},

		LangType::Versioned(VersionedTypeKind::Enum, _, version, type_args, fields) => {
			let mut cases = Vec::new();
	
			for (index, field) in fields.build()?.into_iter().enumerate() {	
				let codec = gen.build_codec(field.field_type)?;
	
				let body = LangStmt::Expr(vec!(),
					Some(LangExpr::CreateEnum(
						gen.type_def().name(),
						version.clone(),
						type_args.clone(),
						field.name,
						Box::new(LangExpr::CodecRead {
							codec: Box::new(codec),
						})
					))
				);
	
				cases.push((BigUint::from(index), body));
			}
	
			LangStmt::MatchDiscriminator {
				value: LangExpr::ReadDiscriminator,
				cases: cases,
			}
		},

		_ => return Err(GeneratorError::InvalidTypeForCodec),
	})
}

fn codec_write_implementation<'model, Gen>(gen: &Gen, t: LangType<'model>) -> Result<LangStmt<'model>, GeneratorError> where
	Gen : TypeGenerator<'model>
{
	Ok(match t.clone() {
		LangType::Versioned(VersionedTypeKind::Struct, _, version, _, fields) => {
			let mut field_values = Vec::new();
	
			for field in fields.build()? {
				let obj_value = LangExpr::Identifier(Gen::Lang::codec_write_value_name().to_string());
				let field_codec = gen.build_codec(field.field_type)?;
				let value_expr = LangExpr::StructField(gen.type_def().name(), version.clone(), field.name, Box::new(obj_value));
	
				field_values.push(LangExpr::CodecWrite {
					codec: Box::new(field_codec),
					value: Box::new(value_expr),
				});
			}
	
			LangStmt::Expr(field_values, None)
		},

		LangType::Versioned(VersionedTypeKind::Enum, _, _, _, fields) => {
			let mut cases = Vec::new();
	
			for (index, field) in fields.build()?.into_iter().enumerate() {
				let value_expr = LangExpr::Identifier(field.name.clone());
				let codec = gen.build_codec(field.field_type)?;
	
	
				cases.push(MatchCase {
					binding_name: field.name.clone(),
					case_name: field.name.clone(),
					body: LangStmt::Expr(vec!(
						LangExpr::WriteDiscriminator(BigUint::from(index)),
						LangExpr::CodecWrite {
							codec: Box::new(codec),
							value: Box::new(value_expr),
						},
					), None),
				});
			}
	
			LangStmt::MatchEnum {
				value: LangExpr::Identifier(Gen::Lang::codec_write_value_name().to_string()),
				value_type: t,
				cases: cases,
			}
		},

		_ => return Err(GeneratorError::InvalidTypeForCodec),
	})
}


pub trait GeneratorWithFile {
	type GeneratorFile : Write;
	fn file(&mut self) -> &mut Self::GeneratorFile;
}

pub trait Indentation : GeneratorWithFile {
    fn indentation_size(&mut self) -> &mut u32;

    fn write_indent(&mut self) -> Result<(), GeneratorError> {
        for _ in 0..*self.indentation_size() {
            write!(self.file(), "\t")?;
        }
		Ok(())
    }

	fn indent_increase(&mut self) {
		*self.indentation_size() += 1;
	}

	fn indent_decrease(&mut self) {
		*self.indentation_size() -= 1;
	}
}


struct ExternTypeChecker<'model, Lang> {
	model: &'model model::Verilization,
	scope: model::Scope<'model>,
	dummy_lang: PhantomData<Lang>,
}

impl <'model, Lang: GeneratorNameMapping> Generator<'model> for ExternTypeChecker<'model, Lang> {
	type Lang = Lang;

	fn model(&self) -> &'model model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}
}



pub trait GeneratorFactory<'a> {
	type ConstGen: ConstGenerator<'a>;
	type VersionedTypeGen: TypeGeneratorGenerate<'a>;
	type InterfaceTypeGen: TypeGeneratorGenerate<'a>;

	fn create_constant_generator(&'a mut self, constant: Named<'a, model::Constant>) -> Result<Self::ConstGen, GeneratorError>;
	fn create_versioned_type_generator(&'a mut self, t: Named<'a, model::VersionedTypeDefinitionData>) -> Result<Self::VersionedTypeGen, GeneratorError>;
	fn create_interface_type_generator(&'a mut self, t: Named<'a, model::InterfaceTypeDefinitionData>) -> Result<Self::InterfaceTypeGen, GeneratorError>;
}

pub trait CodeGenerator<'a> {
	fn generate(&'a mut self, model: &'a model::Verilization) -> Result<(), GeneratorError>;
}

impl <'a, TImpl: for<'b> GeneratorFactory<'b>> CodeGenerator<'a> for TImpl {
	fn generate(&'a mut self, model: &'a model::Verilization) -> Result<(), GeneratorError> {
		for constant in model.constants() {
			let mut gen = self.create_constant_generator(constant)?;
			gen.generate()?;
		}

		for t in model.types() {
			match t {
				model::NamedTypeDefinition::StructType(t) | model::NamedTypeDefinition::EnumType(t) => {
					let mut gen = self.create_versioned_type_generator(t)?;
					gen.generate()?;
				},
				model::NamedTypeDefinition::ExternType(_) => (),
				model::NamedTypeDefinition::InterfaceType(t) => {
					let mut gen = self.create_interface_type_generator(t)?;
					gen.generate()?;
				}
			}
		}

		Ok(())
	}
}


