use verilization_compiler::*;
use lang::generator::*;
use lang::GeneratorError;
use model::{ConstantValue, ConstantValueRecordBuilder};
use verilization_runtime::{FormatWriter, VerilizationCodec};
use rand::Rng;
use num_bigint::{BigInt, BigUint, RandBigInt};
use num_traits::{One, ToPrimitive};
use std::str::FromStr;



pub fn generate_random_value<R: Rng>(random: &mut R, t: LangType) -> Result<ConstantValue, GeneratorError> {
    Ok(match t {
        LangType::Versioned(VersionedTypeKind::Struct, _, _, _, fields) => {
            let mut record = ConstantValueRecordBuilder::new();

            for field in fields.build()? {
                let value = generate_random_value(random, field.field_type)?;
                record.add_field(field.name.clone(), value)?;
            }

            ConstantValue::Record(record.build())
        },

        LangType::Versioned(VersionedTypeKind::Enum, _, _, _, fields) => {
            let mut fields = fields.build()?;
            let index = random.gen_range(0..fields.len());
            let field = fields.remove(index);

            let value = generate_random_value(random, field.field_type)?;

            ConstantValue::Case(field.name.clone(), vec!(value))
        },

        LangType::Extern(_, _, literals) => {
            let mut literals = literals.build()?;
            let literal = literals.remove(random.gen_range(0..literals.len()));

            match literal {
                LangLiteral::Integer(lower_type, lower, upper_type, upper) => {
                    let mut lower = lower.unwrap_or_else(|| BigInt::from_str("-10000000000").unwrap());
                    if lower_type == model::ExternLiteralIntBound::Exclusive {
                        lower += BigInt::one();
                    }

                    let mut upper = upper.unwrap_or_else(|| BigInt::from_str("10000000000").unwrap());
                    if upper_type == model::ExternLiteralIntBound::Inclusive {
                        upper += BigInt::one();
                    }

                    let n = random.gen_bigint_range(&lower, &upper);
                    ConstantValue::Integer(n)
                },

                LangLiteral::String => {
                    let charset = random_string::Charset::new("abcdefABCDEF0123456789").unwrap();
                    let len = random.gen_range(0..2000);
                    let s = random_string::generate(len, &charset).to_string();
                    ConstantValue::String(s)
                },
                LangLiteral::Sequence(element_type) => {
                    let len: u32 = random.gen_range(0..200);
                    let mut values = Vec::new();
                    for _ in 0..len {
                        values.push(generate_random_value(random, element_type.clone())?);
                    }
                    ConstantValue::Sequence(values)
                },
                LangLiteral::Case(case_name, arg_types) => {
                    let mut values = Vec::new();
                    for arg_type in arg_types {
                        values.push(generate_random_value(random, arg_type)?);
                    }
                    ConstantValue::Case(case_name, values)
                },
                LangLiteral::Record(fields) => {
                    let mut record = ConstantValueRecordBuilder::new();
                    for field in fields {
                        record.add_field(field.name.clone(), generate_random_value(random, field.field_type)?)?;
                    }
                    ConstantValue::Record(record.build())
                },
            }
        },
        
        LangType::TypeParameter(_) | LangType::Codec(_) | LangType::Converter(_, _) => return Err(GeneratorError::from("Cannot generate random value for type.")),
    })
}


pub fn write_constant_value<W: FormatWriter<Error = GeneratorError>>(writer: &mut W, value: ConstantValue, t: LangType) -> Result<(), GeneratorError> {
    match (value, t) {
        (ConstantValue::Integer(n), LangType::Extern(name, type_args, _)) if name.package.package.is_empty() => match (name.name.as_ref(), &type_args[..]) {
            ("nat", []) => n.to_biguint().unwrap().write_verilization(writer)?,
            ("int", []) => n.write_verilization(writer)?,
            ("u8", []) => n.to_u8().unwrap().write_verilization(writer)?,
            ("i8", []) => n.to_i8().unwrap().write_verilization(writer)?,
            ("u16", []) => n.to_u16().unwrap().write_verilization(writer)?,
            ("i16", []) => n.to_i16().unwrap().write_verilization(writer)?,
            ("u32", []) => n.to_u32().unwrap().write_verilization(writer)?,
            ("i32", []) => n.to_i32().unwrap().write_verilization(writer)?,
            ("u64", []) => n.to_u64().unwrap().write_verilization(writer)?,
            ("i64", []) => n.to_i64().unwrap().write_verilization(writer)?,
            _ => return Err(GeneratorError::from("Invalid type for integer.")),
        },

        (ConstantValue::String(s), LangType::Extern(name, type_args, _)) if name.package.package.is_empty() && name.name == "string" && type_args.is_empty() =>
            s.write_verilization(writer)?,

            (ConstantValue::Sequence(values), LangType::Extern(name, type_args, _)) if name.package.package.is_empty() && name.name == "list" => match &type_args[..] {
                [element_type] => {
                    BigUint::from(values.len()).write_verilization(writer)?;
                    for value in values {
                        write_constant_value(writer, value, element_type.clone())?;
                    }
                },
                _ => return Err(GeneratorError::from("Invalid type for sequence.")),
            },

        (ConstantValue::Case(case_name, mut values), LangType::Extern(name, mut type_args, _))
                if name.package.package.is_empty() &&
                    name.name == "option" &&
                    type_args.len() == 1 &&
                    case_name == "some" &&
                    values.len() == 1 => {
            let element_type = type_args.remove(0);
            let value = values.remove(0);

            let b: u8 = 1;
            b.write_verilization(writer)?;
            write_constant_value(writer, value, element_type)?;
        },

        (ConstantValue::Case(case_name, values), LangType::Extern(name, type_args, _))
                if name.package.package.is_empty() &&
                    name.name == "option" &&
                    case_name == "none" &&
                    values.is_empty() &&
                    type_args.len() == 1 => {
            let b: u8 = 0;
            b.write_verilization(writer)?;
        },

        (ConstantValue::Case(case_name, mut values), LangType::Versioned(VersionedTypeKind::Enum, _, _, _, fields)) if values.len() == 1 => {
            let value = values.remove(0);

            let (index, field) = fields.build()?.into_iter()
                .enumerate()
                .find(|(_, field)| *field.name == case_name)
                .ok_or("Could not find case for type")?;

            let index = BigUint::from(index);
            index.write_verilization(writer)?;
            write_constant_value(writer, value, field.field_type)?;
        },

        (ConstantValue::Record(record), LangType::Versioned(VersionedTypeKind::Struct, _, _, _, fields)) => {
            let mut values = record.into_field_values();
            for field in fields.build()? {
                let value = values.remove(field.name).ok_or("Field missing from record.")?;
                write_constant_value(writer, value, field.field_type)?;
            }
        },

        (value, t) => return Err(GeneratorError::from(format!("Could not write constant value: {:?} of type {:?}", value, t))),
    }

    Ok(())
}


