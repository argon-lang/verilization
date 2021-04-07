use crate::model;
use num_bigint::{ BigUint, BigInt, Sign };
use num_traits::Zero;
use std::collections::{HashMap, HashSet};

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


fn type_expr(input: &str) -> PResult<&str, model::Type> {
	let (input, name) = identifier(input)?;

	Ok(match name.as_str() {
		"nat" => (input, model::Type::Nat),
		"int" => (input, model::Type::Int),
		"u8" => (input, model::Type::U8),
		"i8" => (input, model::Type::I8),
		"u16" => (input, model::Type::U16),
		"i16" => (input, model::Type::I16),
		"u32" => (input, model::Type::U32),
		"i32" => (input, model::Type::I32),
		"u64" => (input, model::Type::U64),
		"i64" => (input, model::Type::I64),
		"string" => (input, model::Type::String),
		"list" => {
			let (input, elem_type) = type_expr(input)?;
			(input, model::Type::List(Box::new(elem_type)))
		},
		"option" => {
			let (input, elem_type) = type_expr(input)?;
			(input, model::Type::Option(Box::new(elem_type)))
		},
		_ => {
			let (input, parts) = many0(preceded(sym_dot, identifier))(input)?;

			let mut name = name;
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

			(input, model::Type::Defined(qual_name))
		},
	})
}

fn constant_value(input: &str) -> PResult<&str, model::ConstantValue> {
	map(bigint, model::ConstantValue::Integer)(input)
}


enum TopLevelDefinition {
	Constant(model::Constant),
	Type(model::TypeDefinition),
}

// Ex: const name: Type = value;
fn constant_defn(input: &str) -> PResult<&str, (String, TopLevelDefinition)> {
	let (input, _) = kw_const(input)?;
	let (input, name) = identifier(input)?;
	let (input, _) = sym_colon(input)?;
	let (input, t) = type_expr(input)?;
	let (input, _) = multispace0(input)?;
	let (input, _) = sym_eq(input)?;
	let (input, value) = constant_value(input)?;
	let (input, _) = sym_semicolon(input)?;

	Ok((input, (name, TopLevelDefinition::Constant(model::Constant {
		value_type: t,
		value: value,
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
		fields: fields
	})))
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
	let (input, is_enum) = alt((
		map(kw_enum, |_| true),
		map(kw_struct, |_| false),
	))(input)?;

	let (input, name) = identifier(input)?;
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
			model::TypeDefinition::EnumType(model::EnumDefinition {
				versions: version_map,
			})
		}
		else {
			model::TypeDefinition::StructType(model::StructDefinition {
				versions: version_map,
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
