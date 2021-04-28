use crate::model;
use num_bigint::{ BigUint, BigInt, Sign };
use num_traits::{Zero, One};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use nom::{
	IResult,
	branch::{alt},
	multi::{many0, separated_list1, separated_list0},
	character::complete::{multispace0, multispace1, alphanumeric1, one_of, none_of, char},
	combinator::{map, opt, eof, value},
	bytes::complete::tag,
	sequence::{preceded, terminated},
	error::{ParseError, ErrorKind},
};

#[derive(Debug)]
pub enum PErrorType<I> {
	ParseError(I, ErrorKind),
	DuplicateVersion(I, String, BigUint),
	DuplicateField(I, BigUint, String),
	DuplicateFieldValue,
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

fn kw_extern(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("extern")(input)?;
	let (input, _) = multispace1(input)?;
	Ok((input, ()))
}

fn kw_final(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("final")(input)?;
	let (input, _) = multispace1(input)?;
	Ok((input, ()))
}

fn kw_literal(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("literal")(input)?;
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

fn string_literal(input: &str) -> PResult<&str, String> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char('\"')(input)?;

	let (input, chars) = many0(
		alt((
			none_of("\"\\\r\n"),
			preceded(char('\\'),
				alt((
					value('\\', char('\\')),
					value('\"', char('\"')),
					value('n', char('\n')),
					value('\r', char('\r')),
				))
			)
		))
	)(input)?;

	let (input, _) = char('\"')(input)?;

	Ok((input, chars.into_iter().collect()))
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
	
	Ok((input, model::Type::Defined(qual_name, args)))
}

fn case_literal(input: &str) -> PResult<&str, model::ConstantValue> {
	let (input, name) = identifier(input)?;
	let (input, _) = sym_open_paren(input)?;
	let (input, args) = opt(separated_list1(sym_comma, constant_value))(input)?;
	let (input, _) = sym_close_paren(input)?;

	let args = args.unwrap_or_else(|| Vec::new());

	Ok((input, model::ConstantValue::Case(name, args)))
}

fn record_field_literal(input: &str) -> PResult<&str, (String, model::ConstantValue)> {
	let (input, name) = identifier(input)?;
	let (input, _) = sym_eq(input)?;
	let (input, value) = constant_value(input)?;
	Ok((input, (name, value)))
}

fn record_literal(input: &str) -> PResult<&str, model::ConstantValue> {
	let (input, _) = sym_open_curly(input)?;

	let (input, fields) = many0(record_field_literal)(input)?;

	let (input, _) = sym_close_curly(input)?;


	let mut field_map = HashMap::new();
	for (name, value) in fields {
		if field_map.contains_key(&name) {
			return Err(nom::Err::Failure(PErrorType::DuplicateFieldValue));
		}

		field_map.insert(name, value);
	}

	Ok((input, model::ConstantValue::Record(field_map)))
}

fn other_constant(input: &str) -> PResult<&str, model::ConstantValue> {
	let (input, parts) = separated_list1(sym_dot, identifier)(input)?;

	let mut iter = parts.into_iter();

	let mut package = Vec::new();
	let mut name = iter.next().unwrap();

	while let Some(part) = iter.next() {
		package.push(name);
		name = part;
	}

	Ok((input, model::ConstantValue::Constant(
		model::QualifiedName {
			package: model::PackageName {
				package: package,
			},
			name: name
		}
	)))
}


fn constant_value(input: &str) -> PResult<&str, model::ConstantValue> {
	alt((
		map(bigint, model::ConstantValue::Integer),
		map(string_literal, model::ConstantValue::String),
		case_literal,
		record_literal,
		other_constant,
	))(input)
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
fn type_version_definition(input: &str) -> PResult<&str, (BigUint, model::TypeVersionDefinition)> {
	let (input, _) = kw_version(input)?;
	let (input, ver) = biguint(input)?;
	let (input, _) = sym_open_curly(input)?;
	let (input, fields_orig) = many0(field_definition)(input)?;
	let (input, _) = sym_close_curly(input)?;

	let mut field_names: HashSet<String> = HashSet::new();
	let mut fields: Vec<(String, model::FieldInfo)> = Vec::new();
	for (name, field) in fields_orig {
		let mut case_name = name.clone();
		case_name.make_ascii_uppercase();
		if !field_names.insert(case_name) {
			return Err(nom::Err::Failure(PErrorType::DuplicateField(input, ver, name)))
		}

		fields.push((name, field));
	}

	Ok((input, (ver, model::TypeVersionDefinition {
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
fn versioned_type_definition(input: &str) -> PResult<&str, (String, TopLevelDefinition)> {
	let (input, is_final) = opt(kw_final)(input)?;
	let (input, is_enum) = alt((
		map(kw_enum, |_| true),
		map(kw_struct, |_| false),
	))(input)?;

	let (input, name) = identifier(input)?;
	let (input, type_params) = opt(type_param_list)(input)?;
	let type_params = type_params.unwrap_or(Vec::new());
	
	let (input, _) = sym_open_curly(input)?;
	let (input, versions) = many0(type_version_definition)(input)?;
	let (input, _) = sym_close_curly(input)?;

	let mut version_map: HashMap<BigUint, model::TypeVersionDefinition> = HashMap::new();
	for (ver, ver_type) in versions.into_iter() {
		if version_map.contains_key(&ver) {
			return Err(nom::Err::Failure(PErrorType::DuplicateVersion(input, name, ver)))
		}

		version_map.insert(ver, ver_type);
	}

	
	Ok((input, (name, TopLevelDefinition::Type(
		if is_enum {
			model::TypeDefinition::EnumType(model::VersionedTypeDefinitionData {
				imports: HashMap::new(),
				type_params: type_params,
				versions: version_map,
				is_final: is_final.is_some(),
			})
		}
		else {
			model::TypeDefinition::StructType(model::VersionedTypeDefinitionData {
				imports: HashMap::new(),
				type_params: type_params,
				versions: version_map,
				is_final: is_final.is_some(),
			})
		}
	))))
	
}



fn extern_literal_integer(input: &str) -> PResult<&str, model::ExternLiteralSpecifier> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("integer")(input)?;
	let (input, _) = multispace0(input)?;
	let (input, open) = one_of("[(")(input)?;
	let (input, _) = multispace0(input)?;
	let (input, lower) = bigint(input)?;
	let (input, _) = sym_comma(input)?;
	let (input, upper) = bigint(input)?;
	let (input, _) = multispace0(input)?;
	let (input, close) = one_of("])")(input)?;

	let bound = |ch: char| if ch == '(' { model::ExternLiteralIntBound::Exclusive } else { model::ExternLiteralIntBound::Inclusive };

	Ok((input, model::ExternLiteralSpecifier::Integer(bound(open), lower, bound(close), upper)))
}

fn extern_literal_string(input: &str) -> PResult<&str, model::ExternLiteralSpecifier> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("string")(input)?;

	Ok((input, model::ExternLiteralSpecifier::String))
}


fn extern_literal_case(input: &str) -> PResult<&str, model::ExternLiteralSpecifier> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("case")(input)?;
	let (input, name) = identifier(input)?;
	let (input, _) = sym_open_paren(input)?;
	let (input, params) = separated_list0(sym_comma, type_expr)(input)?;
	let (input, _) = sym_close_paren(input)?;

	Ok((input, model::ExternLiteralSpecifier::Case(name, params)))
}


fn extern_literal_record(input: &str) -> PResult<&str, model::ExternLiteralSpecifier> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("record")(input)?;
	let (input, _) = sym_open_curly(input)?;
	let (input, fields) = many0(field_definition)(input)?;
	let (input, _) = sym_close_curly(input)?;

	Ok((input, model::ExternLiteralSpecifier::Record(fields)))
}

fn extern_literal(input: &str) -> PResult<&str, model::ExternLiteralSpecifier> {
	let (input, literal) = alt((extern_literal_integer, extern_literal_string, extern_literal_case, extern_literal_record))(input)?;

	Ok((input, literal))
}


// Ex:
// literal {
//   ...
// }
fn extern_literal_block(input: &str) -> PResult<&str, Vec<model::ExternLiteralSpecifier>> {
	let (input, _) = kw_literal(input)?;
	let (input, _) = sym_open_curly(input)?;
	
	let (input, literals) = many0(extern_literal)(input)?;
	
	let (input, _) = sym_close_curly(input)?;

	Ok((input, literals))
}


// Ex:
// extern Name {
//   ...
// }
fn extern_type_definition(input: &str) -> PResult<&str, (String, TopLevelDefinition)> {
	let (input, _) = kw_extern(input)?;
	let (input, name) = identifier(input)?;
	let (input, type_params) = opt(type_param_list)(input)?;
	let type_params = type_params.unwrap_or(Vec::new());

	let (input, _) = sym_open_curly(input)?;
	
	let (input, literals) = opt(extern_literal_block)(input)?;
	let literals = literals.unwrap_or_else(|| Vec::new());
	
	let (input, _) = sym_close_curly(input)?;

	Ok((input, (name, TopLevelDefinition::Type(model::TypeDefinition::ExternType(model::ExternTypeDefinitionData {
		imports: HashMap::new(),
		type_params: type_params,
		literals: literals,
	})))))
}


fn top_level_definition(input: &str) -> PResult<&str, (String, TopLevelDefinition)> {
	alt((constant_defn, versioned_type_definition, extern_type_definition))(input)
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
