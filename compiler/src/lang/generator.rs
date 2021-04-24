use crate::lang::GeneratorError;
use crate::model;

use model::Named;

use num_bigint::{BigUint, BigInt, Sign};
use std::io::Write;
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug)]
pub enum TypeContext {
	Field,
	ParameterOrResult,
	TypeParameter,
}

#[derive(Clone, Debug)]
pub enum LangTypeV<'model> {
	Nat,
	Int,
	U8,
	I8,
	U16,
	I16,
	U32,
	I32,
	U64,
	I64,
	String,
	Unit,
	List(Box<LangType<'model>>),
	Option(Box<LangType<'model>>),
	Versioned(&'model model::QualifiedName, BigUint, Vec<LangType<'model>>),
	TypeParameter(String),
	Converter(Box<LangType<'model>>, Box<LangType<'model>>),
	Codec(Box<LangType<'model>>),
}

#[derive(Clone, Debug)]
pub struct LangType<'model> {
	pub variant: LangTypeV<'model>,
	pub context: TypeContext,
}

impl <'model> LangType<'model> {
	pub fn new(variant: LangTypeV<'model>, context: TypeContext) -> Self {
		LangType {
			variant: variant,
			context: context,
		}
	}
}

#[derive(Clone, Debug)]
pub enum Operation {
	FromPreviousVersion(BigUint),
	FinalTypeConverter,
	VersionedTypeCodec,
}

#[derive(Debug)]
pub enum LangExpr<'model> {
	Identifier(String),
	InvokeConverter {
		converter: Box<LangExpr<'model>>,
		value: Box<LangExpr<'model>>,
	},
	IdentityConverter(LangType<'model>),
	MapListConverter {
		from_type: LangType<'model>,
		to_type: LangType<'model>,
		element_converter: Box<LangExpr<'model>>,
	},
	MapOptionConverter {
		from_type: LangType<'model>,
		to_type: LangType<'model>,
		element_converter: Box<LangExpr<'model>>,
	},
	NatCodec,
	IntCodec,
	U8Codec,
	I8Codec,
	U16Codec,
	I16Codec,
	U32Codec,
	I32Codec,
	U64Codec,
	I64Codec,
	StringCodec,
	ListCodec(Box<LangExpr<'model>>),
	OptionCodec(Box<LangExpr<'model>>),
	ReadDiscriminator,
	WriteDiscriminator(BigUint),
	CodecRead {
		codec: Box<LangExpr<'model>>,
	},
	CodecWrite {
		codec: Box<LangExpr<'model>>,
		value: Box<LangExpr<'model>>,
	},
	InvokeOperation(Operation, &'model model::QualifiedName, BigUint, Vec<LangType<'model>>, Vec<LangExpr<'model>>),
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
	pub implementation: LangStmt<'model>,
}

pub struct MatchCase<'model> {
	pub binding_name: String,
	pub case_name: String,
	pub body: LangStmt<'model>,
}

pub enum LangStmt<'model> {
	Expr(Vec<LangExpr<'model>>, Option<LangExpr<'model>>),
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
			LangStmt::Expr(_, result) => result.is_some(),
			LangStmt::CreateCodec { .. } => true,
			LangStmt::CreateConverter { .. } => true,
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
	match t {
		model::Type::List(inner) => requires_conversion(gen, inner, prev_ver),
		model::Type::Option(inner) => requires_conversion(gen, inner, prev_ver),
		model::Type::Defined(name, args) => match gen.scope().lookup(name.clone()) {
			model::ScopeLookup::NamedType(name) => match gen.model().get_type(&name) {
				Some(model::NamedTypeDefinition::StructType(type_def) | model::NamedTypeDefinition::EnumType(type_def)) => {
					!type_def.is_final() ||
						(match type_def.last_explicit_version() {
							Some(last_ver) => last_ver > prev_ver,
							None => true
						}) ||
						args.iter().any(|arg| requires_conversion(gen, arg, prev_ver))
				},

				None => true, // Error condition, assume conversion required. Should fail when determining the conversion.
			},
			model::ScopeLookup::TypeParameter(_) => true,
		},
		_ => false,
	}
}

pub trait GeneratorNameMapping {
	fn convert_prev_type_param(param: &str) -> String;
	fn convert_current_type_param(param: &str) -> String;
	fn convert_conv_param_name(param: &str) -> String;
	fn convert_prev_param_name() -> &'static str;

	fn codec_write_value_name() -> &'static str;
	fn codec_codec_param_name(param: &str) -> String;
	
	fn constant_version_name(version: &BigUint) -> String;
}

pub trait Generator<'model> : GeneratorNameMapping + Sized {
	fn model(&self) -> &'model model::Verilization;
	fn scope(&self) -> &model::Scope<'model>;


	fn build_type(&self, version: &BigUint, t: &model::Type, context: TypeContext) -> Result<LangType<'model>, GeneratorError> {
		Ok(match t {
			model::Type::Nat => LangType::new(LangTypeV::Nat, context),
			model::Type::Int => LangType::new(LangTypeV::Int, context),
			model::Type::U8 => LangType::new(LangTypeV::U8, context),
			model::Type::I8 => LangType::new(LangTypeV::I8, context),
			model::Type::U16 => LangType::new(LangTypeV::U16, context),
			model::Type::I16 => LangType::new(LangTypeV::I16, context),
			model::Type::U32 => LangType::new(LangTypeV::U32, context),
			model::Type::I32 => LangType::new(LangTypeV::I32, context),
			model::Type::U64 => LangType::new(LangTypeV::U64, context),
			model::Type::I64 => LangType::new(LangTypeV::I64, context),
			model::Type::String => LangType::new(LangTypeV::String, context),
			model::Type::List(inner) => LangType::new(LangTypeV::List(Box::new(self.build_type(version, &*inner, TypeContext::TypeParameter)?)), context),
			model::Type::Option(inner) => LangType::new(LangTypeV::Option(Box::new(self.build_type(version, &*inner, TypeContext::TypeParameter)?)), context),
			model::Type::Defined(name, args) => match self.scope().lookup(name.clone()) {
				model::ScopeLookup::NamedType(name) => match self.model().get_type(&name).ok_or("Could not find type")? {
					model::NamedTypeDefinition::StructType(type_def) | model::NamedTypeDefinition::EnumType(type_def) => {
						let ver_type = type_def.versioned(version).ok_or("Could not find version of type")?;

						let lang_args = args.iter()
							.map(|arg| self.build_type(version, arg, TypeContext::TypeParameter))
							.collect::<Result<Vec<_>, _>>()?;

						LangType::new(
							LangTypeV::Versioned(type_def.name(), ver_type.version.clone(), lang_args),
							context
						)
					},
				},
				model::ScopeLookup::TypeParameter(name) => LangType::new(LangTypeV::TypeParameter(name), context),
			},
		})
	}

	fn build_codec(&self, version: &BigUint, t: &model::Type) -> Result<LangExpr<'model>, GeneratorError> {
		Ok(match t {
			model::Type::Nat => LangExpr::NatCodec,
			model::Type::Int => LangExpr::IntCodec,
			model::Type::U8 => LangExpr::U8Codec,
			model::Type::I8 => LangExpr::I8Codec,
			model::Type::U16 => LangExpr::U16Codec,
			model::Type::I16 => LangExpr::I16Codec,
			model::Type::U32 => LangExpr::U32Codec,
			model::Type::I32 => LangExpr::I32Codec,
			model::Type::U64 => LangExpr::U64Codec,
			model::Type::I64 => LangExpr::I64Codec,
			model::Type::String => LangExpr::StringCodec,
			model::Type::List(inner) => LangExpr::ListCodec(Box::new(self.build_codec(version, &*inner)?)),
			model::Type::Option(inner) => LangExpr::OptionCodec(Box::new(self.build_codec(version, &*inner)?)),
			model::Type::Defined(name, args) => match self.scope().lookup(name.clone()) {
				model::ScopeLookup::NamedType(name) => {
					let named_type = self.model().get_type(&name).ok_or("Could not find type")?;
					match named_type {
						model::NamedTypeDefinition::StructType(type_def) | model::NamedTypeDefinition::EnumType(type_def) => {
							let ver_type = type_def.versioned(version).ok_or("Could not find version of type")?;

							LangExpr::InvokeOperation(
								Operation::VersionedTypeCodec,
								named_type.name(),
								ver_type.version.clone(),
								args.iter().map(|arg| self.build_type(version, arg, TypeContext::TypeParameter)).collect::<Result<Vec<_>, _>>()?,
								args.iter().map(|arg| self.build_codec(version, arg)).collect::<Result<Vec<_>, _>>()?,
							)
						},
					}
				},
				model::ScopeLookup::TypeParameter(name) => LangExpr::Identifier(Self::codec_codec_param_name(&name)),
			},
		})
	}

	fn build_conversion(&self, prev_ver: &BigUint, version: &BigUint, t: &model::Type, param: ConvertParam<'model>) -> Result<LangExpr<'model>, GeneratorError> {
		if !requires_conversion(self, t, prev_ver) {
			return Ok(match param {
				ConvertParam::ConverterObject => LangExpr::IdentityConverter(self.build_type(version, t, TypeContext::TypeParameter)?),
				ConvertParam::Expression(expr) => expr,
			})
		}

		let converter = match t {
			model::Type::Defined(name, args) => {
				match self.scope().lookup(name.clone()) {
					model::ScopeLookup::NamedType(name) => {
						let named_type_def = self.model().get_type(&name).ok_or("Could not find type")?;
						match named_type_def {
							model::NamedTypeDefinition::StructType(type_def) | model::NamedTypeDefinition::EnumType(type_def) => {
								let ver_type = type_def.versioned(version).ok_or("Could not find version of type")?;

								let mut op_type_args = Vec::new();
								let mut op_args = Vec::new();

								for arg in args {
									op_type_args.push(self.build_type(prev_ver, arg, TypeContext::TypeParameter)?);
									op_type_args.push(self.build_type(version, arg, TypeContext::TypeParameter)?);
									op_args.push(self.build_conversion(prev_ver, version, arg, ConvertParam::ConverterObject)?);
								}

								let operation =
									if ver_type.version < *version {
										Operation::FinalTypeConverter
									}
									else {
										Operation::FromPreviousVersion(prev_ver.clone())
									};

								LangExpr::InvokeOperation(
									operation,
									named_type_def.name(),
									ver_type.version.clone(),
									op_type_args,
									op_args
								)
							},
						}
					},
					model::ScopeLookup::TypeParameter(name) => LangExpr::Identifier(Self::convert_conv_param_name(&name)),
				}
			},
	
			model::Type::List(inner) => LangExpr::MapListConverter {
				from_type: self.build_type(prev_ver, inner, TypeContext::TypeParameter)?,
				to_type: self.build_type(version, inner, TypeContext::TypeParameter)?,
				element_converter: Box::new(self.build_conversion(prev_ver, version, &*inner, ConvertParam::ConverterObject)?),
			},
	
			model::Type::Option(inner) => LangExpr::MapOptionConverter {
				from_type: self.build_type(prev_ver, inner, TypeContext::TypeParameter)?,
				to_type: self.build_type(version, inner, TypeContext::TypeParameter)?,
				element_converter: Box::new(self.build_conversion(prev_ver, version, &*inner, ConvertParam::ConverterObject)?),
			},	
	
			_ => LangExpr::IdentityConverter(self.build_type(version, t, TypeContext::TypeParameter)?),
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

pub trait GeneratorImpl<'model, GenType> {
	fn generate(&mut self) -> Result<(), GeneratorError>;
}

pub trait ConstGenerator<'model> : Generator<'model> {
	fn constant(&self) -> Named<'model, model::Constant>;

	fn write_header(&mut self) -> Result<(), GeneratorError>;
	fn write_constant(&mut self, version_name: String, t: LangType<'model>, value: LangExpr<'model>) -> Result<(), GeneratorError>;
	fn write_footer(&mut self) -> Result<(), GeneratorError>;

	fn build_value(&mut self, _version: &BigUint, _t: &model::Type, _value: &model::ConstantValue) -> Result<LangExpr<'model>, GeneratorError> {
		Err(GeneratorError::from("Not implemented"))
	}

	fn build_value_from_prev(&mut self, prev_ver: &BigUint, version: &BigUint, t: &model::Type) -> Result<LangExpr<'model>, GeneratorError> {
		self.build_conversion(prev_ver, version, t, ConvertParam::Expression(LangExpr::ConstantValue(self.constant().name(), prev_ver.clone())))
	}



}

impl <'model, TImpl> GeneratorImpl<'model, GenConstant> for TImpl where TImpl : ConstGenerator<'model> {
	fn generate(&mut self) -> Result<(), GeneratorError> {
		self.write_header()?;

		for ver in self.constant().versions() {
			let version_name = Self::constant_version_name(&ver.version);
			let t = self.build_type(&ver.version, self.constant().value_type(), TypeContext::Field)?;
			let value =
				if let Some(value) = &ver.value {
					self.build_value(&ver.version, self.constant().value_type(), value)?
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

pub trait VersionedTypeGenerator<'model, GenTypeKind> : Generator<'model> {
	fn type_def(&self) -> Named<'model, model::TypeDefinitionData>;

	fn write_header(&mut self) -> Result<(), GeneratorError>;
	fn write_version_header(&mut self, ver_type: &model::TypeVersionInfo<'model>) -> Result<(), GeneratorError>;
	fn write_operation(&mut self, operation: OperationInfo<'model>) -> Result<(), GeneratorError>;
	fn write_version_footer(&mut self, ver_type: &model::TypeVersionInfo<'model>) -> Result<(), GeneratorError>;
	fn write_footer(&mut self) -> Result<(), GeneratorError>;
}

pub trait VersionedTypeGeneratorOps<'model, GenTypeKind> {
	fn convert_implementation(&self, ver_type: &model::TypeVersionInfo<'model>, prev_ver: &BigUint) -> Result<LangStmt<'model>, GeneratorError>;
	fn codec_read_implementation(&self, ver_type: &model::TypeVersionInfo<'model>) -> Result<LangStmt<'model>, GeneratorError>;
	fn codec_write_implementation(&self, ver_type: &model::TypeVersionInfo<'model>) -> Result<LangStmt<'model>, GeneratorError>;
}


fn build_converter_operation_common<'model, GenTypeKind, Gen>(gen: &Gen, op: Operation, ver_type: &model::TypeVersionInfo<'model>, prev_ver: &BigUint) -> Result<OperationInfo<'model>, GeneratorError> where
	Gen : VersionedTypeGenerator<'model, GenTypeKind> + VersionedTypeGeneratorOps<'model, GenTypeKind>
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
		let t1_arg = LangType::new(LangTypeV::TypeParameter(t1.clone()), TypeContext::TypeParameter);
		let t2_arg = LangType::new(LangTypeV::TypeParameter(t2.clone()), TypeContext::TypeParameter);
		type_args.push(t1_arg.clone());
		type_args.push(t2_arg.clone());
		prev_type_params.push(t1_arg);
		result_type_params.push(t2_arg);

		let conv_type = LangType {
			variant: LangTypeV::Converter(
				Box::new(LangType::new(LangTypeV::TypeParameter(t1), TypeContext::TypeParameter)),
				Box::new(LangType::new(LangTypeV::TypeParameter(t2), TypeContext::TypeParameter)),
			),
			context: TypeContext::ParameterOrResult,
		};

		let conv_param =  Gen::convert_conv_param_name(param);
		params.push((conv_param.clone(), conv_type));
		impl_call_args.push(LangExpr::Identifier(conv_param));
	}

	let prev_type = LangType::new(
		LangTypeV::Versioned(gen.type_def().name(), prev_ver.clone(), prev_type_params),
		TypeContext::TypeParameter,
	);

	let result_type = LangType::new(
		LangTypeV::Versioned(gen.type_def().name(), ver_type.version.clone(), result_type_params),
		TypeContext::TypeParameter,
	);

	let converter_type = LangType::new(
		LangTypeV::Converter(Box::new(prev_type.clone()), Box::new(result_type.clone())),
		TypeContext::ParameterOrResult,
	);

	let implementation = if ver_type.explicit_version && ver_type.version != *prev_ver {
		LangStmt::Expr(vec!(), Some(LangExpr::InvokeUserConverter {
			name: gen.type_def().name(),
			prev_ver: prev_ver.clone(),
			version: version.clone(),
			type_args: type_args,
			args: impl_call_args,
		}))
	}
	else {
		LangStmt::CreateConverter {
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

impl <'model, TImpl, GenTypeKind> GeneratorImpl<'model, GenType<GenTypeKind>> for TImpl where
	TImpl : VersionedTypeGenerator<'model, GenTypeKind> + VersionedTypeGeneratorOps<'model, GenTypeKind>
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
					let param_type = LangType::new(
						LangTypeV::TypeParameter(param.clone()),
						TypeContext::TypeParameter,
					);

					codec_params.push((format!("{}_codec", param), LangType::new(
						LangTypeV::Codec(Box::new(param_type.clone())),
						TypeContext::ParameterOrResult,
					)));

					obj_type_args.push(param_type);
				}

				let obj_type = LangType::new(
					LangTypeV::Versioned(self.type_def().name(), ver_type.version.clone(), obj_type_args),
					TypeContext::TypeParameter,
				);

				let codec_type = LangType::new(
					LangTypeV::Codec(Box::new(obj_type.clone())),
					TypeContext::ParameterOrResult
				);

				let op = OperationInfo {
					operation: Operation::VersionedTypeCodec,
					version: version.clone(),
					type_params: self.type_def().type_params().clone(),
					params: codec_params,
					result: codec_type,
					implementation: LangStmt::CreateCodec {
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

impl <'model, TImpl> VersionedTypeGeneratorOps<'model, GenStructType> for TImpl where
	TImpl : VersionedTypeGenerator<'model, GenStructType>
{
	fn convert_implementation(&self, ver_type: &model::TypeVersionInfo<'model>, prev_ver: &BigUint) -> Result<LangStmt<'model>, GeneratorError> {
		let mut fields = Vec::new();

		let result_type_args = self.type_def().type_params()
			.iter()
			.map(|param| LangType::new(LangTypeV::TypeParameter(Self::convert_current_type_param(param)), TypeContext::TypeParameter))
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
			.map(|param| LangType::new(LangTypeV::TypeParameter(param.clone()), TypeContext::TypeParameter))
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

impl <'model, TImpl> VersionedTypeGeneratorOps<'model, GenEnumType> for TImpl where
	TImpl : VersionedTypeGenerator<'model, GenEnumType>
{
	fn convert_implementation(&self, ver_type: &model::TypeVersionInfo<'model>, prev_ver: &BigUint) -> Result<LangStmt<'model>, GeneratorError> {
		let mut cases = Vec::new();

		let prev_type_args = self.type_def().type_params()
			.iter()
			.map(|param| LangType::new(LangTypeV::TypeParameter(Self::convert_prev_type_param(param)), TypeContext::TypeParameter))
			.collect::<Vec<_>>();

		let result_type_args = self.type_def().type_params()
			.iter()
			.map(|param| LangType::new(LangTypeV::TypeParameter(Self::convert_current_type_param(param)), TypeContext::TypeParameter))
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
			value_type: LangType::new(LangTypeV::Versioned(self.type_def().name(), ver_type.version.clone(), prev_type_args), TypeContext::ParameterOrResult),
			cases: cases,
		})
	}

	fn codec_read_implementation(&self, ver_type: &model::TypeVersionInfo<'model>) -> Result<LangStmt<'model>, GeneratorError> {
		let mut cases = Vec::new();

		for (index, (field_name, field)) in ver_type.ver_type.fields.iter().enumerate() {
			let type_args = self.type_def().type_params()
				.iter()
				.map(|param| LangType::new(LangTypeV::TypeParameter(param.clone()), TypeContext::TypeParameter))
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
			.map(|param| LangType::new(LangTypeV::TypeParameter(param.clone()), TypeContext::TypeParameter))
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
			value_type: LangType::new(LangTypeV::Versioned(self.type_def().name(), ver_type.version.clone(), type_args), TypeContext::ParameterOrResult),
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
