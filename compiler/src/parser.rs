use crate::model;
use num_bigint::{ BigUint, BigInt, Sign };
use num_traits::Zero;
use std::collections::{HashMap};

use nom::{
	IResult,
	branch::{alt},
	multi::{many0, separated_list1, separated_list0},
	character::complete::{multispace0, multispace1, alphanumeric1, one_of, none_of, char},
	combinator::{map, opt, eof, value, cut},
	bytes::complete::tag,
	sequence::{preceded, terminated},
};

type PResult<I, A> = IResult<I, A>;

type ImportMap = HashMap<String, model::QualifiedName>;
type LazyConstantValue = dyn FnOnce() -> Result<model::ConstantValue, model::ModelError>;
type TopLevelDefinitionAdder = dyn FnOnce(&mut model::Verilization) -> Result<(), model::ModelError>;
type TypeVersionAdder = dyn FnOnce(&mut model::VersionedTypeDefinitionBuilder) -> Result<(), model::ModelError>;
type InterfaceVersionAdder = dyn FnOnce(&mut model::InterfaceTypeDefinitionBuilder) -> Result<(), model::ModelError>;
type InterfaceMethodAdder = dyn FnOnce(&mut model::InterfaceVersionDefinitionBuilder) -> Result<(), model::ModelError>;
type ExternLiteralAdder = dyn FnOnce(&mut model::ExternTypeDefinitionBuilder) -> Result<(), model::ModelError>;
type LazyModel = dyn FnOnce() -> Result<model::Verilization, model::ModelError>;


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

fn kw_interface(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("interface")(input)?;
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

fn sym_open_bracket(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char('[')(input)?;
	Ok((input, ()))
}

fn sym_close_bracket(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char(']')(input)?;
	Ok((input, ()))
}

fn sym_open_angle(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char('<')(input)?;
	Ok((input, ()))
}

fn sym_close_angle(input: &str) -> PResult<&str, ()> {
	let (input, _) = multispace0(input)?;
	let (input, _) = char('>')(input)?;
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
	
	Ok((input, model::Type { name: qual_name, args: args }))
}

fn constant_integer_literal(input: &str) -> PResult<&str, Box<LazyConstantValue>> {
	let (input, n) = bigint(input)?;
	Ok((input, Box::new(move || Ok(model::ConstantValue::Integer(n)))))
}

fn constant_string_literal(input: &str) -> PResult<&str, Box<LazyConstantValue>> {
	let (input, s) = string_literal(input)?;
	Ok((input, Box::new(move || Ok(model::ConstantValue::String(s)))))
}

fn sequence_literal(input: &str) -> PResult<&str, Box<LazyConstantValue>> {
	let (input, _) = sym_open_bracket(input)?;
	let (input, values) = opt(
		terminated(
			separated_list1(sym_comma, constant_value),
			opt(sym_comma)
		)
	)(input)?;
	let (input, _) = sym_close_bracket(input)?;

	Ok((input, Box::new(move || {
		let values = values
			.unwrap_or_else(|| Vec::new())
			.into_iter()
			.map(|lazy_const| lazy_const())
			.collect::<Result<Vec<_>, _>>()?;
		Ok(model::ConstantValue::Sequence(values))
	})))
}

fn case_literal(input: &str) -> PResult<&str, Box<LazyConstantValue>> {
	let (input, name) = identifier(input)?;
	let (input, _) = sym_open_paren(input)?;
	let (input, args) = opt(separated_list1(sym_comma, constant_value))(input)?;
	let (input, _) = sym_close_paren(input)?;

	Ok((input, Box::new(move || {
		let args = args
			.unwrap_or_else(|| Vec::new())
			.into_iter()
			.map(|lazy_const| lazy_const())
			.collect::<Result<Vec<_>, _>>()?;

		Ok(model::ConstantValue::Case(name, args))
	})))
}

fn record_field_literal(input: &str) -> PResult<&str, (String, Box<LazyConstantValue>)> {
	let (input, name) = identifier(input)?;
	let (input, _) = sym_eq(input)?;
	let (input, value) = constant_value(input)?;
	let (input, _) = sym_semicolon(input)?;
	Ok((input, (name, value)))
}

fn record_literal(input: &str) -> PResult<&str, Box<LazyConstantValue>> {
	let (input, _) = sym_open_curly(input)?;

	let (input, fields) = many0(record_field_literal)(input)?;

	let (input, _) = sym_close_curly(input)?;

	Ok((input, Box::new(move || {
		let mut record = model::ConstantValueRecordBuilder::new();
		for (name, value) in fields {
			record.add_field(name, value()?)?;
		}

		Ok(model::ConstantValue::Record(record.build()))
	})))
}

fn other_constant(input: &str) -> PResult<&str, Box<LazyConstantValue>> {
	let (input, parts) = separated_list1(sym_dot, identifier)(input)?;

	let mut iter = parts.into_iter();

	let mut package = Vec::new();
	let mut name = iter.next().unwrap();

	while let Some(part) = iter.next() {
		package.push(name);
		name = part;
	}

	Ok((input, Box::new(move || Ok(model::ConstantValue::Constant(
		model::QualifiedName {
			package: model::PackageName {
				package: package,
			},
			name: name
		}
	)))))
}


fn constant_value(input: &str) -> PResult<&str, Box<LazyConstantValue>> {
	alt((
		constant_integer_literal,
		constant_string_literal,
		sequence_literal,
		case_literal,
		record_literal,
		other_constant,
	))(input)
}

fn versioned_constant(input: &str) -> PResult<&str, (BigUint, Box<LazyConstantValue>)> {
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
fn constant_defn(latest_version: BigUint, current_package: model::PackageName, imports: ImportMap) -> impl Fn(&str) -> PResult<&str, Box<TopLevelDefinitionAdder>> {
	move |input| {
		let (input, _) = kw_const(input)?;
		let (input, name) = identifier(input)?;
		let (input, _) = sym_colon(input)?;
		let (input, t) = type_expr(input)?;
		let (input, _) = multispace0(input)?;
		let (input, _) = sym_open_curly(input)?;
		let (input, versions) = many0(versioned_constant)(input)?;
		let (input, _) = sym_close_curly(input)?;
	
		let name = model::QualifiedName { package: current_package.clone(), name: name, };
		let imports = imports.clone();

		let latest_version = latest_version.clone();
	
		Ok((input, Box::new(move |model| {
			let mut constant = model::ConstantBuilder::new(latest_version.clone(), name, t, imports);
			for (ver, value) in versions {
				constant.add_version(ver, value()?)?;
			}
			
			model.add_constant(constant)
		})))
	}
}

fn variable_declaration_part(input: &str) -> PResult<&str, (String, model::Type)> {
	let (input, name) = identifier(input)?;
	let (input, _) = cut(sym_colon)(input)?;
	let (input, t) = cut(type_expr)(input)?;

	Ok((input, (name, t)))
}

// Ex: name: Type;
fn field_definition(input: &str) -> PResult<&str, (String, model::FieldInfo)> {
	let (input, (name, t)) = variable_declaration_part(input)?;
	let (input, _) = cut(sym_semicolon)(input)?;

	Ok((input, (name, model::FieldInfo {
		field_type: t,
	})))
}

fn param_definition(input: &str) -> PResult<&str, model::ParameterInfo> {
	let (input, (name, t)) = variable_declaration_part(input)?;

	Ok((input, model::ParameterInfo {
		name: name,
		param_type: t,
	}))
}

// Ex:
// version 5 {
//   ...	
// }
fn type_version_definition(input: &str) -> PResult<&str, Box<TypeVersionAdder>> {
	let (input, _) = kw_version(input)?;
	let (input, ver) = cut(biguint)(input)?;
	let (input, _) = cut(sym_open_curly)(input)?;
	let (input, fields_orig) = many0(field_definition)(input)?;
	let (input, _) = cut(sym_close_curly)(input)?;

	Ok((input, Box::new(|type_def| {
		let mut ver_type = type_def.add_version(ver)?;
		for (name, field) in fields_orig {	
			ver_type.add_field(name, field)?;
		}
		Ok(())
	})))
}

// Ex:
// funcName<T1, T2>(arg1: A1, arg2: A2): R;
fn method_definition(input: &str) -> PResult<&str, Box<InterfaceMethodAdder>> {
	let (input, name) = identifier(input)?;
	let (input, type_params) = opt(type_param_list)(input)?;
	let type_params = type_params.unwrap_or(Vec::new());

	let (input, _) = cut(sym_open_paren)(input)?;
	let (input, params) = separated_list1(sym_comma, param_definition)(input)?;
	let (input, _) = cut(sym_close_paren)(input)?;

	let (input, _) = cut(sym_colon)(input)?;
	let (input, t) = cut(type_expr)(input)?;
	let (input, _) = cut(sym_semicolon)(input)?;

	Ok((input, Box::new(|ver_builder| {
		let mut method = ver_builder.add_method(name, t)?;
		type_params.into_iter().try_for_each(|p| method.add_type_param(p))?;
		params.into_iter().try_for_each(|p| method.add_param(p))?;
		Ok(())
	})))
}

// Ex:
// version 5 {
//   ...	
// }
fn interface_version_definition(input: &str) -> PResult<&str, Box<InterfaceVersionAdder>> {
	let (input, _) = kw_version(input)?;
	let (input, ver) = cut(biguint)(input)?;
	let (input, _) = cut(sym_open_curly)(input)?;
	let (input, methods_adders) = many0(method_definition)(input)?;
	let (input, _) = cut(sym_close_curly)(input)?;

	Ok((input, Box::new(|type_def| {
		let mut ver_type = type_def.add_version(ver)?;
		for method_adder in methods_adders {	
			method_adder(&mut ver_type)?;
		}
		Ok(())
	})))
}

fn type_param_list(input: &str) -> PResult<&str, Vec<String>> {
	let (input, _) = sym_open_angle(input)?;
	let (input, result) = separated_list1(sym_comma, identifier)(input)?;
	let (input, _) = sym_close_angle(input)?;

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
fn versioned_type_definition(latest_version: BigUint, current_package: model::PackageName, imports: ImportMap) -> impl Fn(&str) -> PResult<&str, Box<TopLevelDefinitionAdder>> {
	move |input| {
		let (input, is_final) = opt(kw_final)(input)?;
		let is_final = is_final.is_some();

		let (input, is_enum) = alt((
			map(kw_enum, |_| true),
			map(kw_struct, |_| false),
		))(input)?;
	
		let (input, name) = cut(identifier)(input)?;
		let (input, type_params) = opt(type_param_list)(input)?;
		let type_params = type_params.unwrap_or(Vec::new());
		
		let (input, _) = cut(sym_open_curly)(input)?;
		let (input, versions) = many0(type_version_definition)(input)?;
		let (input, _) = cut(sym_close_curly)(input)?;
	
		let name = model::QualifiedName { package: current_package.clone(), name: name, };
		let imports = imports.clone();
	
		let latest_version = latest_version.clone();
		
		Ok((input, Box::new(move |model| {
			let mut type_def = model::VersionedTypeDefinitionBuilder::new(latest_version, name, type_params, is_final, imports);
			for adder in versions {
				adder(&mut type_def)?;
			}

			if is_enum {
				model.add_enum_type(type_def)
			}
			else {
				model.add_struct_type(type_def)
			}
		})))
	}
}

fn extern_literal_integer(input: &str) -> PResult<&str, Box<ExternLiteralAdder>> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("integer")(input)?;
	let (input, _) = multispace0(input)?;
	let (input, open) = one_of("[(")(input)?;
	let (input, _) = multispace0(input)?;
	let (input, lower) = opt(bigint)(input)?;
	let (input, _) = sym_comma(input)?;
	let (input, upper) = opt(bigint)(input)?;
	let (input, _) = multispace0(input)?;
	let (input, close) = one_of("])")(input)?;

	let bound = |ch: char| if ch == '(' { model::ExternLiteralIntBound::Exclusive } else { model::ExternLiteralIntBound::Inclusive };

	Ok((input, Box::new(move |type_def| type_def.add_integer_literal(bound(open), lower, bound(close), upper))))
}

fn extern_literal_string(input: &str) -> PResult<&str, Box<ExternLiteralAdder>> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("string")(input)?;

	Ok((input, Box::new(model::ExternTypeDefinitionBuilder::add_string_literal)))
}

fn extern_literal_sequence(input: &str) -> PResult<&str, Box<ExternLiteralAdder>> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("sequence")(input)?;
	let (input, _) = multispace1(input)?;
	let (input, element_type) = type_expr(input)?;

	Ok((input, Box::new(|type_def| type_def.add_sequence_literal(element_type))))
}


fn extern_literal_case(input: &str) -> PResult<&str, Box<ExternLiteralAdder>> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("case")(input)?;
	let (input, name) = identifier(input)?;
	let (input, _) = sym_open_paren(input)?;
	let (input, params) = separated_list0(sym_comma, type_expr)(input)?;
	let (input, _) = sym_close_paren(input)?;

	Ok((input, Box::new(|type_def| type_def.add_case_literal(name, params))))
}


fn extern_literal_record(input: &str) -> PResult<&str, Box<ExternLiteralAdder>> {
	let (input, _) = multispace0(input)?;
	let (input, _) = tag("record")(input)?;
	let (input, _) = sym_open_curly(input)?;
	let (input, fields) = many0(field_definition)(input)?;
	let (input, _) = sym_close_curly(input)?;

	Ok((input, Box::new(|type_def| {
		let mut record = type_def.add_record_literal()?;
		for (name, field) in fields {
			record.add_field(name, field)?;
		}
		Ok(())
	})))
}

fn extern_literal(input: &str) -> PResult<&str, Box<ExternLiteralAdder>> {
	let (input, literal) = alt((extern_literal_integer, extern_literal_string, extern_literal_sequence, extern_literal_case, extern_literal_record))(input)?;
	let (input, _) = sym_semicolon(input)?;

	Ok((input, literal))
}


// Ex:
// literal {
//   ...
// }
fn extern_literal_block(input: &str) -> PResult<&str, Vec<Box<ExternLiteralAdder>>> {
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
fn extern_type_definition(current_package: model::PackageName, imports: ImportMap) -> impl Fn(&str) -> PResult<&str, Box<TopLevelDefinitionAdder>> {
	move |input| {
		let (input, _) = kw_extern(input)?;
		let (input, name) = identifier(input)?;
		let (input, type_params) = opt(type_param_list)(input)?;
		let type_params = type_params.unwrap_or(Vec::new());
	
		let (input, _) = sym_open_curly(input)?;
		
		let (input, literals) = opt(extern_literal_block)(input)?;
		let literals = literals.unwrap_or_else(|| Vec::new());
		
		let (input, _) = sym_close_curly(input)?;
		
		let name = model::QualifiedName { package: current_package.clone(), name: name, };
		let imports = imports.clone();
	
	
		Ok((input, Box::new(|model| {
			let mut type_def = model::ExternTypeDefinitionBuilder::new(name, type_params, imports);
			for literal_adder in literals {
				literal_adder(&mut type_def)?;
			}
			model.add_extern_type(type_def)
		})))
	}
}


// Ex:
// interface Name {
//     ...
// }
fn interface_type_definition(latest_version: BigUint, current_package: model::PackageName, imports: ImportMap) -> impl Fn(&str) -> PResult<&str, Box<TopLevelDefinitionAdder>> {
	move |input| {
		let (input, is_final) = opt(kw_final)(input)?;
		let is_final = is_final.is_some();

		let (input, _) = kw_interface(input)?;
	
		let (input, name) = cut(identifier)(input)?;
		let (input, type_params) = opt(type_param_list)(input)?;
		let type_params = type_params.unwrap_or(Vec::new());
		
		let (input, _) = cut(sym_open_curly)(input)?;
		let (input, versions) = many0(interface_version_definition)(input)?;
		let (input, _) = cut(sym_close_curly)(input)?;
	
		let name = model::QualifiedName { package: current_package.clone(), name: name, };
		let imports = imports.clone();
	
		let latest_version = latest_version.clone();
		
		Ok((input, Box::new(move |model| {
			let mut type_def = model::InterfaceTypeDefinitionBuilder::new(latest_version, name, type_params, is_final, imports);
			for adder in versions {
				adder(&mut type_def)?;
			}

			model.add_interface(type_def)
		})))
	}
}



fn top_level_definition(latest_version: BigUint, current_package: model::PackageName, imports: ImportMap) -> impl Fn(&str) -> PResult<&str, Box<TopLevelDefinitionAdder>> {
	move |input| alt((
		constant_defn(latest_version.clone(), current_package.clone(), imports.clone()),
		versioned_type_definition(latest_version.clone(), current_package.clone(), imports.clone()),
		extern_type_definition(current_package.clone(), imports.clone()),
		interface_type_definition(latest_version.clone(), current_package.clone(), imports.clone())
	))(input)
}


pub fn parse_model(input: &str) -> PResult<&str, Box<LazyModel>> {
	let (input, latest_ver) = version_directive(input)?;
	let (input, package) = opt(package_directive)(input)?;
	let package =
			if let Some(pkg) = package { pkg }
			else { model::PackageName { package: Vec::new() } };



	let (input, defs) = many0(top_level_definition(latest_ver, package, HashMap::new()))(input)?;
	let (input, _) = multispace0(input)?;
	let (input, _) = eof(input)?;



	Ok((input, Box::new(move || {
		let mut model = model::Verilization::new();
	
		for def_adder in defs.into_iter() {
			def_adder(&mut model)?;
		}

		Ok(model)
	})))
}
