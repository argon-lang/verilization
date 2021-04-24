use crate::model;
use num_bigint::{ BigUint, BigInt, Sign };
use num_traits::{Zero, One};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use nom::{
	IResult,
	branch::{alt},
	multi::{many0, separated_list1},
	character::complete::{multispace0, multispace1, alphanumeric1, one_of, char},
	combinator::{map, opt, eof},
	bytes::complete::tag,
	sequence::preceded,
	error::{ParseError, ErrorKind},
};

#[derive(Debug)]
pub enum PErrorType<I> {
	ParseError(I, ErrorKind),
	DuplicateVersion(I, String, BigUint),
	DuplicateField(I, BigUint, String),
	DuplicateConstant(model::QualifiedName),
	DuplicateType(model::QualifiedName),
}

impl<I> ParseError<I> for PErrorType<I> {
	fn from_error_kind(input: I, kind: ErrorKind) -> Self {
		PErrorType::ParseError(input, kind)
	}

	fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
		other
	}
}

type PResult<I, A> = IResult<I, A, PErrorType<I>>;

// Keywords
fn kw_version(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("version")(input)?;
	let (input, _) = multispace1(input)?;
	Ok((input, ()))
}

fn kw_package(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("package")(input)?;
	let (input, _) = multispace1(input)?;
	Ok((input, ()))
}

fn kw_const(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("const")(input)?;
	let (input, _) = multispace1(input)?;
	Ok((input, ()))
}

fn kw_enum(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("enum")(input)?;
	let (input, _) = multispace1(input)?;
	Ok((input, ()))
}

fn kw_struct(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("struct")(input)?;
	let (input, _) = multispace1(input)?;
	Ok((input, ()))
}

fn kw_final(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("final")(input)?;
	let (input, _) = multispace1(input)?;
	Ok((input, ()))
}

// Symbols
fn sym_semicolon(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char(';')(input)?;
	Ok((input, ()))
}

fn sym_colon(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char(':')(input)?;
	Ok((input, ()))
}

fn sym_dot(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char('.')(input)?;
	Ok((input, ()))
}

fn sym_comma(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char(',')(input)?;
	Ok((input, ()))
}

fn sym_eq(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char('=')(input)?;
	Ok((input, ()))
}

fn sym_open_curly(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char('{')(input)?;
	Ok((input, ()))
}

fn sym_close_curly(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char('}')(input)?;
	Ok((input, ()))
}

fn sym_open_paren(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char('(')(input)?;
	Ok((input, ()))
}

fn sym_close_paren(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char(')')(input)?;
	Ok((input, ()))
}


fn dec_digit(input: &str) -> PResult<&str, u8> {
	let (input, ch) = one_of("0123456789")(input)?;
	let digit = ch.to_digit(10).unwrap() as u8;
	Ok((input, digit))
}


// Integer literal (no signs)
fn biguint(input: &str) -> PResult<&str, BigUint> {
	let (input, _) = multispace0(input)?;
	let (input, first_dig) = dec_digit(input)?;
	
	if first_dig == 0 {
		Ok((input, Zero::zero()))
	}
	else {
		let (input, mut digits) = many0(dec_digit)(input)?;
		digits.insert(0, first_dig);
		Ok((input, BigUint::from_radix_be(&digits, 10).unwrap()))
	}
}

// Allows an optional sign before the integer literal
fn bigint(input: &str) -> PResult<&str, BigInt> {
	let (input, _) = multispace0(input)?;
	let (input, sign) = opt(one_of("+-"))(input)?;
	let (input, n) = biguint(input)?;

	let sign =
		if n == Zero::zero() {
			Sign::NoSign
		} else {
			match sign {
				Some('-') => Sign::Minus,
				_ => Sign::Plus,
			}
		};

	Ok((input, BigInt::from_biguint(sign, n)))
}

fn identifier(input: &str) -> PResult<&str, String> {
	let (input, _) = multispace0(input)?;
	let (input, str) = alphanumeric1(input)?;
	Ok((input, str.to_string()))
}

// Ex: version 5;
fn version_directive(input: &str) -> PResult<&str, BigUint> {
	let (input, _) = kw_version(input)?;
	let (input, ver) = biguint(input)?;
	let (input, _) = sym_semicolon(input)?;

	Ok((input, ver))
}

// Ex: package hello.world;
fn package_directive(input: &str) -> PResult<&str, model::PackageName> {
	let (input, _) = kw_package(input)?;
	let (input, pkg) = separated_list1(sym_dot, identifier)(input)?;
	let (input, _) = sym_semicolon(input)?;

	Ok((input, model::PackageName { package: pkg }))
}

fn type_expr_args(input: &str) -> PResult<&str, Vec<model::Type>> {
	let (input, _) = sym_open_paren(input)?;
	let (input, args) = separated_list1(sym_comma, type_expr)(input)?;
	let (input, _) = sym_close_paren(input)?;
	Ok((input, args))
}

fn type_expr(input: &str) -> PResult<&str, model::Type> {
	let (input, qual_name, args) = {
		let (input, mut name) = identifier(input)?;
		let (input, parts) = many0(preceded(sym_dot, identifier))(input)?;

		let mut package: Vec<String> = Vec::new();
		for part in parts.into_iter() {
			package.push(name);
			name = part;
		}

		let qual_name = model::QualifiedName {
			package: model::PackageName {
				package: package,
			},
			name: name,
		};

		let (input, args) = opt(type_expr_args)(input)?;

		(input, qual_name, args.unwrap_or(Vec::new()))
	};
	
	if qual_name.package.package.is_empty() {
		Ok((input, match (qual_name.name.as_str(), &args[..]) {
			("nat", []) => model::Type::Nat,
			("int", []) => model::Type::Int,
			("u8", []) => model::Type::U8,
			("i8", []) => model::Type::I8,
			("u16", []) => model::Type::U16,
			("i16", []) => model::Type::I16,
			("u32", []) => model::Type::U32,
			("i32", []) => model::Type::I32,
			("u64", []) => model::Type::U64,
			("i64", []) => model::Type::I64,
			("string", []) => model::Type::String,
			("list", [elem_type]) => {
				model::Type::List(Box::new(elem_type.clone()))
			},
			("option", [elem_type]) => {
				model::Type::Option(Box::new(elem_type.clone()))
			},
			(_, _) => model::Type::Defined(qual_name, args),
		}))
	}
	else {
		Ok((input, model::Type::Defined(qual_name, args)))
	}
}

fn constant_value(input: &str) -> PResult<&str, model::ConstantValue> {
	map(bigint, model::ConstantValue::Integer)(input)
}


enum TopLevelDefinition {
	Constant(model::Constant),
	Type(model::TypeDefinition),
}

fn versioned_constant(input: &str) -> PResult<&str, (BigUint, model::ConstantValue)> {
	let (input, _) = kw_version(input)?;
	let (input, ver) = biguint(input)?;
	let (input, _) = sym_eq(input)?;
	let (input, value) = constant_value(input)?;
	let (input, _) = sym_semicolon(input)?;
	Ok((input, (ver, value)))
}

// Ex:
// const name: Type {
//     version 1 = ...;
//}
fn constant_defn(input: &str) -> PResult<&str, (String, TopLevelDefinition)> {
	let (input, _) = kw_const(input)?;
	let (input, name) = identifier(input)?;
	let (input, _) = sym_colon(input)?;
	let (input, t) = type_expr(input)?;
	let (input, _) = multispace0(input)?;
	let (input, _) = sym_open_curly(input)?;
	let (input, versions) = many0(versioned_constant)(input)?;
	let (input, _) = sym_close_curly(input)?;

	let mut version_map = HashMap::new();
	for (ver, value) in versions.into_iter() {
		if version_map.contains_key(&ver) {
			return Err(nom::Err::Failure(PErrorType::DuplicateVersion(input, name, ver)))
		}

		version_map.insert(ver, value);
	}

	Ok((input, (name, TopLevelDefinition::Constant(model::Constant {
		imports: HashMap::new(),
		value_type: t,
		versions: version_map,
	}))))
}

// Ex: name: Type;
fn field_definition(input: &str) -> PResult<&str, (String, model::FieldInfo)> {
	let (input, name) = identifier(input)?;
	let (input, _) = sym_colon(input)?;
	let (input, t) = type_expr(input)?;
	let (input, _) = sym_semicolon(input)?;

	Ok((input, (name, model::FieldInfo {
		field_type: t,
		dummy: PhantomData {},
	})))
}

// Ex:
// version 5 {
//   ...	
// }
fn versioned_type(input: &str) -> PResult<&str, (BigUint, model::VersionedTypeDefinition)> {
	let (input, _) = kw_version(input)?;
	let (input, ver) = biguint(input)?;
	let (input, _) = sym_open_curly(input)?;
	let (input, fields_orig) = many0(field_definition)(input)?;
	let (input, _) = sym_close_curly(input)?;

	let mut field_names: HashSet<String> = HashSet::new();
	let mut fields: Vec<(String, model::FieldInfo)> = Vec::new();
	for (name, field) in fields_orig {
		if !field_names.insert(name.clone()) {
			return Err(nom::Err::Failure(PErrorType::DuplicateField(input, ver, name)))
		}

		fields.push((name, field));
	}

	Ok((input, (ver, model::VersionedTypeDefinition {
		fields: fields,
		dummy: PhantomData {},
	})))
}

fn type_param_list(input: &str) -> PResult<&str, Vec<String>> {
	let (input, _) = sym_open_paren(input)?;
	let (input, result) = separated_list1(sym_comma, identifier)(input)?;
	let (input, _) = sym_close_paren(input)?;

	Ok((input, result))
}

// Ex:
// struct Name {
//   version 5 {...}
//   ...
// }
// enum Name {
//   version 5 {...}
//   ...
// }
fn type_definition(input: &str) -> PResult<&str, (String, TopLevelDefinition)> {
	let (input, is_final) = opt(kw_final)(input)?;
	let (input, is_enum) = alt((
		map(kw_enum, |_| true),
		map(kw_struct, |_| false),
	))(input)?;

	let (input, name) = identifier(input)?;
	let (input, type_params) = opt(type_param_list)(input)?;
	let type_params = type_params.unwrap_or(Vec::new());
	
	let (input, _) = sym_open_curly(input)?;
	let (input, versions) = many0(versioned_type)(input)?;
	let (input, _) = sym_close_curly(input)?;

	let mut version_map: HashMap<BigUint, model::VersionedTypeDefinition> = HashMap::new();
	for (ver, ver_type) in versions.into_iter() {
		if version_map.contains_key(&ver) {
			return Err(nom::Err::Failure(PErrorType::DuplicateVersion(input, name, ver)))
		}

		version_map.insert(ver, ver_type);
	}

	
	Ok((input, (name, TopLevelDefinition::Type(
		if is_enum {
			model::TypeDefinition::EnumType(model::TypeDefinitionData {
				imports: HashMap::new(),
				type_params: type_params,
				versions: version_map,
				is_final: is_final.is_some(),
			})
		}
		else {
			model::TypeDefinition::StructType(model::TypeDefinitionData {
				imports: HashMap::new(),
				type_params: type_params,
				versions: version_map,
				is_final: is_final.is_some(),
			})
		}
	))))
	
}


fn top_level_definition(input: &str) -> PResult<&str, (String, TopLevelDefinition)> {
	alt((constant_defn, type_definition))(input)
}


pub fn parse_model(input: &str) -> PResult<&str, model::Verilization> {
	let (input, latest_ver) = version_directive(input)?;
	let (input, package) = opt(package_directive)(input)?;
	let package =
			if let Some(pkg) = package { pkg }
			else { model::PackageName { package: Vec::new() } };


	let mut data = model::Verilization::new(model::VerilizationMetadata {
		latest_version: latest_ver,
	});

	let (input, defs) = many0(top_level_definition)(input)?;

	for (name, def) in defs.into_iter() {
		let qual_name = model::QualifiedName {
			package: package.clone(),
			name: name,
		};

		match def {
			TopLevelDefinition::Constant(constant) => data.add_constant(qual_name, constant).map_err(|qual_name| nom::Err::Failure(PErrorType::DuplicateConstant(qual_name)))?,
			TopLevelDefinition::Type(t) => data.add_type(qual_name, t).map_err(|qual_name| nom::Err::Failure(PErrorType::DuplicateType(qual_name)))?,
		}
	}


	let (input, _) = multispace0(input)?;
	let (input, _) = eof(input)?;

	Ok((input, data))
}
