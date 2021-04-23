use crate::lang::GeneratorError;
use crate::model;

use model::Named;

use num_bigint::BigUint;
use std::io::Write;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub enum TypeContext {
	Field,
	ParameterOrResult,
	TypeParameter,
}


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
	List(Box<LangType<'model>>),
	Option(Box<LangType<'model>>),
	Versioned(Named<'model, model::TypeDefinitionData>, model::TypeVersionInfo<'model>, Vec<LangType<'model>>),
	TypeParameter(String)
}

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

#[derive(Clone)]
pub enum Operation<'model> {
	FromPreviousVersion {
		name: &'model model::QualifiedName,
		prev_ver: BigUint,
		version: BigUint,
	},
	FinalTypeConverter(&'model model::QualifiedName, BigUint),
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
	ListCodec,
	OptionCodec,
	VersionedTypeCodec(&'model model::QualifiedName, BigUint),
}

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
	InvokeOperation(Operation<'model>, Vec<LangType<'model>>, Vec<LangExpr<'model>>),
}

pub trait ExprWriter<'model> {
	fn write<F: Write>(self, f: &mut F) -> Result<(), GeneratorError>;
}

pub enum ConvertParam<'model> {
	ConverterObject,
	Expression(LangExpr<'model>),
}

pub struct ConversionInfo<'model> {
	pub from_type: LangType<'model>,
	pub to_type: LangType<'model>,
	pub converter: LangExpr<'model>,

	dummy: PhantomData<()>,
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

pub trait Generator<'model> : Sized {
	fn model(&self) -> &'model model::Verilization;
	fn scope(&self) -> &model::Scope<'model>;

	fn write_type(&mut self, t: LangType<'model>) -> Result<(), GeneratorError>;
	fn write_expr(&mut self, t: LangType<'model>) -> Result<(), GeneratorError>;

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
							LangTypeV::Versioned(type_def, ver_type, lang_args),
							context
						)
					},
				},
				model::ScopeLookup::TypeParameter(name) => LangType::new(LangTypeV::TypeParameter(name), context),
			},
		})
	}

	fn build_conversion(&mut self, prev_ver: &BigUint, version: &BigUint, t: &model::Type, param: ConvertParam<'model>) -> Result<LangExpr<'model>, GeneratorError> {
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
										Operation::FinalTypeConverter(named_type_def.name(), version.clone())
									}
									else {
										Operation::FromPreviousVersion {
											name: named_type_def.name(),
											prev_ver: prev_ver.clone(),
											version: version.clone(),
										}
									};


								LangExpr::InvokeOperation(
									operation,
									op_type_args,
									op_args
								)
							},
						}
					},
					model::ScopeLookup::TypeParameter(name) => LangExpr::Identifier(name),
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

pub trait ConstGenerator<'model> : Generator<'model> {
	fn constant(&self) -> Named<'model, model::Constant>;

	fn version_name(version: &BigUint) -> String;

	fn write_header(&mut self) -> Result<(), GeneratorError>;
	fn write_constant(&mut self, name: &str, t: LangType<'model>, value: LangExpr<'model>) -> Result<(), GeneratorError>;
	fn write_footer(&mut self) -> Result<(), GeneratorError>;
}

