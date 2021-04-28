use crate::lang::GeneratorError;
use crate::model;

use model::Named;

use std::collections::HashMap;
use num_bigint::{BigUint, BigInt, Sign};
use std::io::Write;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub enum LangType<'model> {
	Versioned(&'model model::QualifiedName, BigUint, Vec<LangType<'model>>),
	Extern(&'model model::QualifiedName, Vec<LangType<'model>>),
	TypeParameter(String),
	Converter(Box<LangType<'model>>, Box<LangType<'model>>),
	Codec(Box<LangType<'model>>),
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
}

#[derive(Debug)]
pub enum OperationTarget<'model> {
	VersionedType(&'model model::QualifiedName, BigUint),
	ExternType(&'model model::QualifiedName)
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
	CreateStruct(&'model model::QualifiedName, BigUint, Vec<LangType<'model>>, Vec<(String, LangExpr<'model>)>),
	CreateEnum(&'model model::QualifiedName, BigUint, Vec<LangType<'model>>, String, Box<LangExpr<'model>>),
	StructField(&'model model::QualifiedName, BigUint, String, Box<LangExpr<'model>>),
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

fn requires_conversion<'model, Lang, G: Generator<'model, Lang>>(gen: &G, t: &model::Type, prev_ver: &BigUint) -> bool {
	match t {
		model::Type::Defined(name, args) => match gen.scope().lookup(name.clone()) {
			model::ScopeLookup::NamedType(name) => match gen.model().get_type(&name) {
				Some(model::NamedTypeDefinition::StructType(type_def)) | Some(model::NamedTypeDefinition::EnumType(type_def)) => {
					!type_def.is_final() ||
						(match type_def.last_explicit_version() {
							Some(last_ver) => last_ver > prev_ver,
							None => true
						}) ||
						args.iter().any(|arg| requires_conversion(gen, arg, prev_ver))
				},

				Some(model::NamedTypeDefinition::ExternType(_)) => false,

				None => true, // Error condition, assume conversion required. Should fail when determining the conversion.
			},
			model::ScopeLookup::TypeParameter(_) => true,
		},
	}
}

pub trait GeneratorNameMapping<Lang> {
	fn convert_prev_type_param(param: &str) -> String;
	fn convert_current_type_param(param: &str) -> String;
	fn convert_conv_param_name(param: &str) -> String;
	fn convert_prev_param_name() -> &'static str;

	fn codec_write_value_name() -> &'static str;
	fn codec_codec_param_name(param: &str) -> String;
	
	fn constant_version_name(version: &BigUint) -> String;
}

pub trait Generator<'model, Lang> : GeneratorNameMapping<Lang> + Sized {
	fn model(&self) -> &'model model::Verilization;
	fn scope(&self) -> &model::Scope<'model>;


	fn build_type(&self, version: &BigUint, t: &model::Type) -> Result<LangType<'model>, GeneratorError> {
		Ok(match t {
			model::Type::Defined(name, args) => {
				let lang_args = args.iter()
					.map(|arg| self.build_type(version, arg))
					.collect::<Result<Vec<_>, _>>()?;

				match self.scope().lookup(name.clone()) {
					model::ScopeLookup::NamedType(name) => match self.model().get_type(&name).ok_or("Could not find type")? {
						model::NamedTypeDefinition::StructType(type_def) | model::NamedTypeDefinition::EnumType(type_def) => {
							let ver_type = type_def.versioned(version).ok_or_else(|| format!("Could not find version {} of type: {:?}", version, t))?;
							LangType::Versioned(type_def.name(), ver_type.version.clone(), lang_args)
						},

						model::NamedTypeDefinition::ExternType(type_def) =>
							LangType::Extern(type_def.name(), lang_args),
					},
					model::ScopeLookup::TypeParameter(name) => LangType::TypeParameter(name),
				}
			},
		})
	}

	fn build_codec(&self, version: &BigUint, t: &model::Type) -> Result<LangExpr<'model>, GeneratorError> {
		Ok(match t {
			model::Type::Defined(name, args) => match self.scope().lookup(name.clone()) {
				model::ScopeLookup::NamedType(name) => {

					let target;

					let named_type = self.model().get_type(&name).ok_or("Could not find type")?;
					match named_type {
						model::NamedTypeDefinition::StructType(type_def) | model::NamedTypeDefinition::EnumType(type_def) => {
							let ver_type = type_def.versioned(version).ok_or_else(|| format!("Could not find version {} of type: {:?}", version, t))?;
							target = OperationTarget::VersionedType(named_type.name(), ver_type.version.clone());
						},

						model::NamedTypeDefinition::ExternType(_) =>
							target = OperationTarget::ExternType(named_type.name()),
					}

					LangExpr::InvokeOperation(
						Operation::TypeCodec,
						target,
						args.iter().map(|arg| self.build_type(version, arg)).collect::<Result<Vec<_>, _>>()?,
						args.iter().map(|arg| self.build_codec(version, arg)).collect::<Result<Vec<_>, _>>()?,
					)
				},
				model::ScopeLookup::TypeParameter(name) => LangExpr::Identifier(Self::codec_codec_param_name(&name)),
			},
		})
	}

	fn build_conversion(&self, prev_ver: &BigUint, version: &BigUint, t: &model::Type, param: ConvertParam<'model>) -> Result<LangExpr<'model>, GeneratorError> {
		if !requires_conversion(self, t, prev_ver) {
			return Ok(match param {
				ConvertParam::ConverterObject => LangExpr::IdentityConverter(self.build_type(version, t)?),
				ConvertParam::Expression(expr) => expr,
			})
		}

		let converter = match t {
			model::Type::Defined(name, args) => {
				match self.scope().lookup(name.clone()) {
					model::ScopeLookup::NamedType(name) => {

						let mut op_type_args = Vec::new();
						let mut op_args = Vec::new();

						for arg in args {
							op_type_args.push(self.build_type(prev_ver, arg)?);
							op_type_args.push(self.build_type(version, arg)?);
							op_args.push(self.build_conversion(prev_ver, version, arg, ConvertParam::ConverterObject)?);
						}


						let named_type_def = self.model().get_type(&name).ok_or("Could not find type")?;
						let operation;
						let target;
						match named_type_def {
							model::NamedTypeDefinition::StructType(type_def) | model::NamedTypeDefinition::EnumType(type_def) => {
								let ver_type = type_def.versioned(version).ok_or_else(|| format!("Could not find version {} of type: {:?}", version, t))?;

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
						};

						LangExpr::InvokeOperation(
							operation,
							target,
							op_type_args,
							op_args
						)
					},
					model::ScopeLookup::TypeParameter(name) => LangExpr::Identifier(Self::convert_conv_param_name(&name)),
				}
			},
		};

					
		Ok(match param {
			ConvertParam::ConverterObject => converter,
			ConvertParam::Expression(expr) => LangExpr::InvokeConverter {
				converter: Box::new(converter),
				value: Box::new(expr),
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

#[derive(Default)]
pub struct GenStructType {}

#[derive(Default)]
pub struct GenEnumType {}

pub trait GeneratorImpl<'model, Lang, GenType> {
	fn generate(&mut self) -> Result<(), GeneratorError>;
}

pub enum TypeArgMap<'model, 'a> {
	HasArgs(HashMap<String, &'model model::Type>, &'a TypeArgMap<'model, 'a>),
	Empty,
}

pub trait ConstGenerator<'model, Lang> : Generator<'model, Lang> {
	fn constant(&self) -> Named<'model, model::Constant>;

	fn write_header(&mut self) -> Result<(), GeneratorError>;
	fn write_constant(&mut self, version_name: String, t: LangType<'model>, value: LangExpr<'model>) -> Result<(), GeneratorError>;
	fn write_footer(&mut self) -> Result<(), GeneratorError>;

	fn constant_invoke_operation(&self, op: Operation, values: Vec<LangExpr<'model>>, version: &BigUint, t: &model::Type) -> Result<LangExpr<'model>, GeneratorError> {
		match t {
			model::Type::Defined(name, type_args) => match self.scope().lookup(name.clone()) {
				model::ScopeLookup::NamedType(name) => {
					let named_type_def = self.model().get_type(&name).ok_or("Could not find type")?;
					let lang_type_args = type_args.iter().map(|arg| self.build_type(version, arg)).collect::<Result<Vec<_>, _>>()?;

					let target = match named_type_def {
						model::NamedTypeDefinition::StructType(type_def) | model::NamedTypeDefinition::EnumType(type_def) => {
							let ver_type = type_def.versioned(version).ok_or_else(|| format!("Could not find version {} of type: {:?}", version, t))?;
							OperationTarget::VersionedType(named_type_def.name(), ver_type.version)
						},

						model::NamedTypeDefinition::ExternType(_) =>
							OperationTarget::ExternType(named_type_def.name()),
					};

					Ok(LangExpr::InvokeOperation(op, target, lang_type_args, values))
				},

				model::ScopeLookup::TypeParameter(_) => Err(GeneratorError::from("Cannot create constant for type parameter")),
			},
		}
	}

	fn build_value<'a>(&self, version: &BigUint, t: &'model model::Type, value: &'model model::ConstantValue, type_arg_map: &'a TypeArgMap<'model, 'a>) -> Result<LangExpr<'model>, GeneratorError> {
		Ok(match value {
			model::ConstantValue::Integer(n) => self.constant_invoke_operation(Operation::FromInteger, vec!(LangExpr::IntegerLiteral(n.clone())), version, t)?,
			model::ConstantValue::String(s) => self.constant_invoke_operation(Operation::FromString, vec!(LangExpr::StringLiteral(s.clone())), version, t)?,
			model::ConstantValue::Sequence(seq) => {
				match t {
					model::Type::Defined(type_name, type_args) => {
						let lang_type_args = type_args.iter().map(|arg| self.build_type(version, arg)).collect::<Result<Vec<_>, _>>()?;
						match self.scope().lookup(type_name.clone()) {
							model::ScopeLookup::NamedType(name) => {
								let named_type_def = self.model().get_type(&name).ok_or("Could not find type")?;
								let type_arg_map = TypeArgMap::HasArgs(
									named_type_def.type_params().iter().map(String::clone).zip(type_args).collect::<HashMap<_, _>>(),
									type_arg_map
								);
			
								match named_type_def {
									model::NamedTypeDefinition::StructType(_) | model::NamedTypeDefinition::EnumType(_) => return Err(GeneratorError::from("Cannot use sequence syntax for non-extern type")),
			
									model::NamedTypeDefinition::ExternType(type_def) => {
										let element_type = type_def.literals()
											.iter()
											.find_map(|literal| match literal {
												model::ExternLiteralSpecifier::Sequence(element_type) => Some(element_type),
												_ => None,
											})
											.ok_or("Type does not have a sequence literal")?;

										let args = seq.iter()
											.map(|elem| self.build_value(version, element_type, elem, &type_arg_map))
											.collect::<Result<Vec<_>, _>>()?;

										LangExpr::InvokeOperation(
											Operation::FromSequence,
											OperationTarget::ExternType(named_type_def.name()),
											lang_type_args,
											args,
										)
									},
								}
							},
			
							model::ScopeLookup::TypeParameter(type_param_name) => {
								match type_arg_map {
									TypeArgMap::HasArgs(map, prev) => {
										let t = map.get(&type_param_name).ok_or("Unknown type parameter")?;
										self.build_value(version, t, value, prev)?
									},
									TypeArgMap::Empty => return Err(GeneratorError::from("Unknown type parameter")),
								}
							},
						}
					},
				}
			}
			model::ConstantValue::Case(case_name, args) => {
				match t {
					model::Type::Defined(type_name, type_args) => {
						let lang_type_args = type_args.iter().map(|arg| self.build_type(version, arg)).collect::<Result<Vec<_>, _>>()?;
						match self.scope().lookup(type_name.clone()) {
							model::ScopeLookup::NamedType(name) => {
								let named_type_def = self.model().get_type(&name).ok_or("Could not find type")?;
								let type_arg_map = TypeArgMap::HasArgs(
									named_type_def.type_params().iter().map(String::clone).zip(type_args).collect::<HashMap<_, _>>(),
									type_arg_map
								);
			
								match named_type_def {
									model::NamedTypeDefinition::StructType(_) => return Err(GeneratorError::from("Cannot use case syntax for struct literal")),
									model::NamedTypeDefinition::EnumType(type_def) => {
										match &args[..] {
											[arg] => {
												let ver_type = type_def.versioned(version).ok_or_else(|| format!("Could not find version {} of type: {:?}", version, t))?;

												let (_, field) = ver_type.ver_type.fields.iter().find(|(field_name, _)| field_name == case_name).ok_or("Could not find field")?;

												let arg = self.build_value(version, &field.field_type, &arg, &type_arg_map)?;
												LangExpr::CreateEnum(named_type_def.name(), ver_type.version.clone(), lang_type_args, case_name.clone(), Box::new(arg))
											},
											_ => return Err(GeneratorError::from("Incorrect number of arguments")),
										}
									},
			
									model::NamedTypeDefinition::ExternType(type_def) => {
										let case_params = type_def.literals()
											.iter()
											.find_map(|literal| match literal {
												model::ExternLiteralSpecifier::Case(name, case_params) if name == case_name => Some(case_params),
												_ => None,
											})
											.ok_or("Type does not have a matching case")?;

										let args = args.iter().zip(case_params.into_iter())
											.map(|(arg, param)| self.build_value(version, param, arg, &type_arg_map))
											.collect::<Result<Vec<_>, _>>()?;

										LangExpr::InvokeOperation(
											Operation::FromCase(case_name.clone()),
											OperationTarget::ExternType(named_type_def.name()),
											lang_type_args,
											args,
										)
									},
								}
							},
			
							model::ScopeLookup::TypeParameter(type_param_name) => {
								match type_arg_map {
									TypeArgMap::HasArgs(map, prev) => {
										let t = map.get(&type_param_name).ok_or("Unknown type parameter")?;
										self.build_value(version, t, value, prev)?
									},
									TypeArgMap::Empty => return Err(GeneratorError::from("Unknown type parameter")),
								}
							},
						}
					},
				}
			},
			model::ConstantValue::Record(field_values) => {
				match t {
					model::Type::Defined(type_name, type_args) => {
						let lang_type_args = type_args.iter().map(|arg| self.build_type(version, arg)).collect::<Result<Vec<_>, _>>()?;
						match self.scope().lookup(type_name.clone()) {
							model::ScopeLookup::NamedType(name) => {
								let named_type_def = self.model().get_type(&name).ok_or("Could not find type")?;
								let type_arg_map = TypeArgMap::HasArgs(
									named_type_def.type_params().iter().map(String::clone).zip(type_args).collect::<HashMap<_, _>>(),
									type_arg_map
								);
			
								match named_type_def {
									model::NamedTypeDefinition::StructType(type_def) => {
										let ver_type = type_def.versioned(version).ok_or_else(|| format!("Could not find version {} of type: {:?}", version, t))?;
										
										let mut args = Vec::new();

										for (field_name, field) in &ver_type.ver_type.fields {
											let value = field_values.get(field_name).ok_or("Could not find record field in literal")?;
											let value = self.build_value(version, &field.field_type, value, &type_arg_map)?;
											args.push((field_name.clone(), value));
										}

										LangExpr::CreateStruct(named_type_def.name(), ver_type.version.clone(), lang_type_args, args)
									},
									model::NamedTypeDefinition::EnumType(_) => return Err(GeneratorError::from("Cannot use record syntax for enum literal")),
			
									model::NamedTypeDefinition::ExternType(type_def) => {
										let record_fields = type_def.literals()
											.iter()
											.find_map(|literal| match literal {
												model::ExternLiteralSpecifier::Record(fields) => Some(fields),
												_ => None,
											})
											.ok_or("Type does not have a record literal")?;

										let mut field_names = Vec::new();
										let mut args = Vec::new();

										for (field_name, field) in record_fields {
											let value = field_values.get(field_name).ok_or("Could not find record field in literal")?;
											let value = self.build_value(version, &field.field_type, value, &type_arg_map)?;
											field_names.push(field_name.clone());
											args.push(value);
										}

										LangExpr::InvokeOperation(
											Operation::FromRecord(field_names),
											OperationTarget::ExternType(named_type_def.name()),
											lang_type_args,
											args,
										)
									},
								}
							},
			
							model::ScopeLookup::TypeParameter(_) => return Err(GeneratorError::from("Cannot create constant for type parameter")),
						}
					},
				}
			},
			model::ConstantValue::Constant(name) => LangExpr::ConstantValue(name, version.clone())
		})
	}

	fn build_value_from_prev(&self, prev_ver: &BigUint, version: &BigUint, t: &model::Type) -> Result<LangExpr<'model>, GeneratorError> {
		self.build_conversion(prev_ver, version, t, ConvertParam::Expression(LangExpr::ConstantValue(self.constant().name(), prev_ver.clone())))
	}



}

impl <'model, TImpl, Lang> GeneratorImpl<'model, Lang, GenConstant> for TImpl where TImpl : ConstGenerator<'model, Lang> {
	fn generate(&mut self) -> Result<(), GeneratorError> {
		self.write_header()?;

		for ver in self.constant().versions() {
			let version_name = Self::constant_version_name(&ver.version);
			let t = self.build_type(&ver.version, self.constant().value_type())?;
			let value =
				if let Some(value) = &ver.value {
					self.build_value(&ver.version, self.constant().value_type(), value, &TypeArgMap::Empty)?
				}
				else {
					let prev_ver: BigInt = BigInt::from_biguint(Sign::Plus, ver.version.clone()) - 1;
					let prev_ver = prev_ver.to_biguint().unwrap();
					self.build_value_from_prev(&prev_ver, &ver.version, self.constant().value_type())?
				};

			self.write_constant(version_name, t, value)?;
		}

		self.write_footer()
	}
}

pub trait VersionedTypeGenerator<'model, Lang, GenTypeKind> : Generator<'model, Lang> {
	fn type_def(&self) -> Named<'model, model::VersionedTypeDefinitionData>;

	fn write_header(&mut self) -> Result<(), GeneratorError>;
	fn write_version_header(&mut self, ver_type: &model::TypeVersionInfo<'model>) -> Result<(), GeneratorError>;
	fn write_operation(&mut self, operation: OperationInfo<'model>) -> Result<(), GeneratorError>;
	fn write_version_footer(&mut self, ver_type: &model::TypeVersionInfo<'model>) -> Result<(), GeneratorError>;
	fn write_footer(&mut self) -> Result<(), GeneratorError>;
}

pub trait VersionedTypeGeneratorOps<'model, Lang, GenTypeKind> {
	fn convert_implementation(&self, ver_type: &model::TypeVersionInfo<'model>, prev_ver: &BigUint) -> Result<LangStmt<'model>, GeneratorError>;
	fn codec_read_implementation(&self, ver_type: &model::TypeVersionInfo<'model>) -> Result<LangStmt<'model>, GeneratorError>;
	fn codec_write_implementation(&self, ver_type: &model::TypeVersionInfo<'model>) -> Result<LangStmt<'model>, GeneratorError>;
}


fn build_converter_operation_common<'model, Lang, GenTypeKind, Gen>(gen: &Gen, op: Operation, ver_type: &model::TypeVersionInfo<'model>, prev_ver: &BigUint) -> Result<OperationInfo<'model>, GeneratorError> where
	Gen : VersionedTypeGenerator<'model, Lang, GenTypeKind> + VersionedTypeGeneratorOps<'model, Lang, GenTypeKind>
{
	let version = &ver_type.version;


	let mut type_params = Vec::new();
	let mut type_args = Vec::new();
	let mut params = Vec::new();
	let mut prev_type_params = Vec::new();
	let mut result_type_params = Vec::new();
	let mut impl_call_args = Vec::new();

	for param in gen.type_def().type_params() {
		let t1 = Gen::convert_prev_type_param(&param);
		let t2 = Gen::convert_current_type_param(&param);
		type_params.push(t1.clone());
		type_params.push(t2.clone());
		let t1_arg = LangType::TypeParameter(t1.clone());
		let t2_arg = LangType::TypeParameter(t2.clone());
		type_args.push(t1_arg.clone());
		type_args.push(t2_arg.clone());
		prev_type_params.push(t1_arg);
		result_type_params.push(t2_arg);

		let conv_type = LangType::Converter(
			Box::new(LangType::TypeParameter(t1)),
			Box::new(LangType::TypeParameter(t2)),
		);

		let conv_param =  Gen::convert_conv_param_name(param);
		params.push((conv_param.clone(), conv_type));
		impl_call_args.push(LangExpr::Identifier(conv_param));
	}

	let prev_type = LangType::Versioned(gen.type_def().name(), prev_ver.clone(), prev_type_params);

	let result_type = LangType::Versioned(gen.type_def().name(), ver_type.version.clone(), result_type_params);

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
		LangExprStmt::CreateConverter {
			from_type: prev_type,
			to_type: result_type,
			body: Box::new(gen.convert_implementation(&ver_type, prev_ver)?),
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

impl <'model, TImpl, Lang, GenTypeKind> GeneratorImpl<'model, Lang, GenType<GenTypeKind>> for TImpl where
	TImpl : VersionedTypeGenerator<'model, Lang, GenTypeKind> + VersionedTypeGeneratorOps<'model, Lang, GenTypeKind>
{
	fn generate(&mut self) -> Result<(), GeneratorError> {
		self.write_header()?;

		let mut first_version = true;
		
		for ver_type in self.type_def().versions() {
			let version = &ver_type.version;
	
			let prev_ver: BigInt = BigInt::from_biguint(Sign::Plus, version.clone()) - 1;
			let prev_ver = prev_ver.to_biguint().unwrap();


			self.write_version_header(&ver_type)?;

			// Converter for latest version of final type with type parameters
			if self.type_def().is_final() && !self.type_def().type_params().is_empty() && self.type_def().last_explicit_version() == Some(&ver_type.version) {
				self.write_operation(build_converter_operation_common(self, Operation::FinalTypeConverter, &ver_type, version)?)?;
			}
			
			// Conversion from previous version
			if !first_version { // Skip when there is no prevous version.
				self.write_operation(build_converter_operation_common(self, Operation::FromPreviousVersion(prev_ver.clone()), &ver_type, &prev_ver)?)?;
			}

			// Codec
			{
				let mut codec_params = Vec::new();

				let mut obj_type_args = Vec::new();

				for param in self.type_def().type_params() {
					let param_type = LangType::TypeParameter(param.clone());

					codec_params.push((Self::codec_codec_param_name(param), LangType::Codec(Box::new(param_type.clone()))));

					obj_type_args.push(param_type);
				}

				let obj_type = LangType::Versioned(self.type_def().name(), ver_type.version.clone(), obj_type_args);

				let codec_type = LangType::Codec(Box::new(obj_type.clone()));

				let op = OperationInfo {
					operation: Operation::TypeCodec,
					version: version.clone(),
					type_params: self.type_def().type_params().clone(),
					params: codec_params,
					result: codec_type,
					implementation: LangExprStmt::CreateCodec {
						t: obj_type,
						read: Box::new(self.codec_read_implementation(&ver_type)?),
						write: Box::new(self.codec_write_implementation(&ver_type)?),
					},
				};

				self.write_operation(op)?;
			}


			self.write_version_footer(&ver_type)?;
			first_version = false;
		}

		self.write_footer()
	}
}

impl <'model, TImpl, Lang> VersionedTypeGeneratorOps<'model, Lang, GenStructType> for TImpl where
	TImpl : VersionedTypeGenerator<'model, Lang, GenStructType>
{
	fn convert_implementation(&self, ver_type: &model::TypeVersionInfo<'model>, prev_ver: &BigUint) -> Result<LangStmt<'model>, GeneratorError> {
		let mut fields = Vec::new();

		let result_type_args = self.type_def().type_params()
			.iter()
			.map(|param| LangType::TypeParameter(Self::convert_current_type_param(param)))
			.collect::<Vec<_>>();

		for (field_name, field) in &ver_type.ver_type.fields {
			let obj_value = LangExpr::Identifier(Self::convert_prev_param_name().to_string());

			let value_expr = LangExpr::StructField(self.type_def().name(), ver_type.version.clone(), field_name.clone(), Box::new(obj_value));
			let conv_value = self.build_conversion(prev_ver, &ver_type.version, &field.field_type, ConvertParam::Expression(value_expr))?;

			fields.push((field_name.clone(), conv_value));
		}

		Ok(LangStmt::Expr(vec!(),
			Some(LangExpr::CreateStruct(self.type_def().name(), ver_type.version.clone(), result_type_args, fields))
		))
	}

	fn codec_read_implementation(&self, ver_type: &model::TypeVersionInfo<'model>) -> Result<LangStmt<'model>, GeneratorError> {

		let mut fields = Vec::new();

		let type_args = self.type_def().type_params()
			.iter()
			.map(|param| LangType::TypeParameter(param.clone()))
			.collect::<Vec<_>>();

		for (field_name, field) in &ver_type.ver_type.fields {
			let field_codec = self.build_codec(&ver_type.version, &field.field_type)?;
			fields.push((field_name.clone(), LangExpr::CodecRead { codec: Box::new(field_codec) }));
		}

		Ok(LangStmt::Expr(vec!(),
			Some(LangExpr::CreateStruct(self.type_def().name(), ver_type.version.clone(), type_args, fields))
		))
	}

	fn codec_write_implementation(&self, ver_type: &model::TypeVersionInfo<'model>) -> Result<LangStmt<'model>, GeneratorError> {
		let mut fields = Vec::new();

		for (field_name, field) in &ver_type.ver_type.fields {
			let obj_value = LangExpr::Identifier(Self::codec_write_value_name().to_string());
			let field_codec = self.build_codec(&ver_type.version, &field.field_type)?;
			let value_expr = LangExpr::StructField(self.type_def().name(), ver_type.version.clone(), field_name.clone(), Box::new(obj_value));

			fields.push(LangExpr::CodecWrite {
				codec: Box::new(field_codec),
				value: Box::new(value_expr),
			});
		}

		Ok(LangStmt::Expr(fields, None))
	}
}

impl <'model, TImpl, Lang> VersionedTypeGeneratorOps<'model, Lang, GenEnumType> for TImpl where
	TImpl : VersionedTypeGenerator<'model, Lang, GenEnumType>
{
	fn convert_implementation(&self, ver_type: &model::TypeVersionInfo<'model>, prev_ver: &BigUint) -> Result<LangStmt<'model>, GeneratorError> {
		let mut cases = Vec::new();

		let prev_type_args = self.type_def().type_params()
			.iter()
			.map(|param| LangType::TypeParameter(Self::convert_prev_type_param(param)))
			.collect::<Vec<_>>();

		let result_type_args = self.type_def().type_params()
			.iter()
			.map(|param| LangType::TypeParameter(Self::convert_current_type_param(param)))
			.collect::<Vec<_>>();

		for (field_name, field) in &ver_type.ver_type.fields {

			let value_expr = LangExpr::Identifier(field_name.clone());
			let conv_value = self.build_conversion(prev_ver, &ver_type.version, &field.field_type, ConvertParam::Expression(value_expr))?;
			let enum_value = LangExpr::CreateEnum(self.type_def().name(), ver_type.version.clone(), result_type_args.clone(), field_name.clone(), Box::new(conv_value));

			cases.push(MatchCase {
				binding_name: field_name.clone(),
				case_name: field_name.clone(),
				body: LangStmt::Expr(vec!(), Some(enum_value)),
			});
		}

		Ok(LangStmt::MatchEnum {
			value: LangExpr::Identifier(Self::convert_prev_param_name().to_string()),
			value_type: LangType::Versioned(self.type_def().name(), prev_ver.clone(), prev_type_args),
			cases: cases,
		})
	}

	fn codec_read_implementation(&self, ver_type: &model::TypeVersionInfo<'model>) -> Result<LangStmt<'model>, GeneratorError> {
		let mut cases = Vec::new();

		for (index, (field_name, field)) in ver_type.ver_type.fields.iter().enumerate() {
			let type_args = self.type_def().type_params()
				.iter()
				.map(|param| LangType::TypeParameter(param.clone()))
				.collect::<Vec<_>>();


			let codec = self.build_codec(&ver_type.version, &field.field_type)?;

			let body = LangStmt::Expr(vec!(),
				Some(LangExpr::CreateEnum(
					self.type_def().name(),
					ver_type.version.clone(),
					type_args,
					field_name.clone(),
					Box::new(LangExpr::CodecRead {
						codec: Box::new(codec),
					})
				))
			);

			cases.push((BigUint::from(index), body));
		}

		Ok(LangStmt::MatchDiscriminator {
			value: LangExpr::ReadDiscriminator,
			cases: cases,
		})
	}

	fn codec_write_implementation(&self, ver_type: &model::TypeVersionInfo<'model>) -> Result<LangStmt<'model>, GeneratorError> {
		let mut cases = Vec::new();

		let type_args = self.type_def().type_params()
			.iter()
			.map(|param| LangType::TypeParameter(param.clone()))
			.collect::<Vec<_>>();

		for (index, (field_name, field)) in ver_type.ver_type.fields.iter().enumerate() {
			let value_expr = LangExpr::Identifier(field_name.clone());
			let codec = self.build_codec(&ver_type.version, &field.field_type)?;


			cases.push(MatchCase {
				binding_name: field_name.clone(),
				case_name: field_name.clone(),
				body: LangStmt::Expr(vec!(
					LangExpr::WriteDiscriminator(BigUint::from(index)),
					LangExpr::CodecWrite {
						codec: Box::new(codec),
						value: Box::new(value_expr),
					},
				), None),
			});
		}

		Ok(LangStmt::MatchEnum {
			value: LangExpr::Identifier(Self::codec_write_value_name().to_string()),
			value_type: LangType::Versioned(self.type_def().name(), ver_type.version.clone(), type_args),
			cases: cases,
		})
	}
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
