use verilization_compiler::{lang, model};
use lang::GeneratorError;
use model::Verilization;

use crate::memory_format::MemoryFormatWriter;
use crate::test_lang::{TestLanguage, TestGenerator};

use std::collections::HashSet;
use num_bigint::{ BigUint, BigInt };
use std::fs;
use std::fs::File;
use std::io::Write;
use rand::Rng;
use verilization_runtime::VerilizationCodec;
use num_traits::{Zero, One};
use num_bigint::RandBigInt;

struct JavaTestGenState<'model, F, R> {
    file: &'model mut F,
    random: &'model mut R,
    model: &'model Verilization,
}

fn write_random_value<F: Write, W: verilization_runtime::FormatWriter<Error = GeneratorError>, R: Rng>(f: &mut F, writer: &mut W, random: &mut R, model: &Verilization, version: &BigUint, t: &model::Type) -> Result<(), GeneratorError> {
    match t {
        model::Type::Nat => {
            let n: BigUint = random.gen_biguint(256);
            write!(f, "new java.math.BigInteger(\"{}\")", n)?;
            n.write_verilization(writer)
        },
        model::Type::Int => {
            let n: BigInt = random.gen_bigint(256);
            write!(f, "new java.math.BigInteger(\"{}\")", n)?;
            n.write_verilization(writer)
        },
        model::Type::U8 | model::Type::I8 => {
            let n: i8 = random.gen();
            write!(f, "{}", n)?;
            n.write_verilization(writer)
        },
        model::Type::U16 | model::Type::I16 => {
            let n: i16 = random.gen();
            write!(f, "{}", n)?;
            n.write_verilization(writer)
        },
        model::Type::U32 | model::Type::I32 => {
            let n: i32 = random.gen();
            write!(f, "{}", n)?;
            n.write_verilization(writer)
        },
        model::Type::U64 | model::Type::I64 => {
            let n: i64 = random.gen();
            write!(f, "{}L", n)?;
            n.write_verilization(writer)
        },
        model::Type::String => {
            let charset = random_string::Charset::new("abcdefABCDEF0123456789").unwrap();
            let len = random.gen_range(0..2000);
            let s = random_string::generate(len, &charset).to_string();
            write!(f, "\"{}\"", s)?;
            s.write_verilization(writer)?;

            Ok(())
        },
        model::Type::List(inner) => {
            write!(f, "java.util.List.of(")?;

            let len: u32 = random.gen_range(0..200);
            BigUint::from(len).write_verilization(writer)?;
            for _ in 0..len {
                write_random_value(f, writer, random, model, version, &*inner)?;
                write!(f, ", ")?;
            }

            write!(f, ")")?;

            Ok(())
        },
        model::Type::Option(inner) => {
            let b: bool = random.gen();
            if b {
                BigUint::one().write_verilization(writer)?;
                write!(f, "java.util.Optional.of(")?;
                write_random_value(f, writer, random, model, version, &*inner)?;
                write!(f, ")")?;
            }
            else {
                BigUint::zero().write_verilization(writer)?;
                write!(f, "java.util.Optional.empty()")?;
            }

            Ok(())
        },
        model::Type::Defined(name) => {
            let (t, ver_type) = model.type_in_version(name, version).ok_or("Could not find type in version.")?;
            let options = lang::scala::ScalaLanguage::test_options();

            match t {
                model::TypeDefinition::StructType(_) => {
                    write!(f, "new ")?;
                    lang::java::write_qual_name(f, &options.package_mapping, name)?;
                    write!(f, ".V{}(", version)?;

                    {
                        let mut iter = ver_type.fields.iter();
                        if let Some((_, field)) = iter.next() {
                            write_random_value(f, writer, random, model, version, &field.field_type)?;
                            while let Some((_, field)) = iter.next() {
                                write!(f, ", ")?;
                                write_random_value(f, writer, random, model, version, &field.field_type)?;
                            }
                        }
                    }
                    write!(f, ")")?;

                    Ok(())
                },

                model::TypeDefinition::EnumType(_) => {
                    let index = random.gen_range(0..ver_type.fields.len());
                    let (field_name, field) = &ver_type.fields[index];

                    BigUint::from(index).write_verilization(writer)?;
                    write!(f, "new ")?;
                    lang::java::write_qual_name(f, &options.package_mapping, name)?;
                    write!(f, ".V{}.{}(", version, field_name)?;
                    write_random_value(f, writer, random, model, version, &field.field_type)?;
                    write!(f, ")")?;

                    Ok(())
                },
            }
        },
    }
}

impl <'model, F: Write, R: Rng> model::TypeDefinitionHandler<'model, GeneratorError> for JavaTestGenState<'model, F, R> {
    type StructHandlerState<'state> where 'model : 'state = &'state mut JavaTestGenState<'model, F, R>;
    type EnumHandlerState<'state> where 'model : 'state = &'state mut JavaTestGenState<'model, F, R>;
}

impl <'model, 'state, F: Write, R: Rng> model::TypeDefinitionHandlerState<'model, 'state, JavaTestGenState<'model, F, R>, GeneratorError> for &'state mut JavaTestGenState<'model, F, R> where 'model : 'state {
    fn begin(outer: &'state mut JavaTestGenState<'model, F, R>, _type_name: &'model model::QualifiedName, _referenced_types: HashSet<&'model model::QualifiedName>) -> Result<Self, GeneratorError> {
        Ok(outer)
    }

    fn versioned_type(&mut self, _explicit_version: bool, type_name: &'model model::QualifiedName, version: &BigUint, _type_definition: &'model model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
        write!(self.file, "\t\tsertests.TestCase(")?;

        let options = lang::scala::ScalaLanguage::test_options();

        let current_type = model::Type::Defined(type_name.clone());

        lang::java::write_codec(self.file, &options.package_mapping, version, &current_type)?;
        write!(self.file, ", ")?;
        
        let mut writer = MemoryFormatWriter::new();
        write_random_value(self.file, &mut writer, self.random, self.model, version, &current_type)?;
        
        write!(self.file, ", zio.Chunk[Byte](")?;
        {
            let data = writer.data();
            let mut iter = data.iter();
            if let Some(b) = iter.next() {
                write!(self.file, "{}", *b as i8)?;
                while let Some(b) = iter.next() {
                    write!(self.file, ", {}", *b as i8)?;
                }
            }
        }
        writeln!(self.file, ")),")?;


        Ok(())
    }

    fn end(self, _type_name: &'model model::QualifiedName) -> Result<(), GeneratorError> {
        Ok(())
    }
}

pub struct ScalaTestGenerator {
    file: File,
}

impl TestGenerator for ScalaTestGenerator {
    fn start() -> Result<ScalaTestGenerator, GeneratorError> {
        fs::create_dir_all("../scala/gen-test/")?;
        let mut file = File::create("../scala/gen-test/Tests.scala")?;

        writeln!(file, "object Tests extends sertests.TestsBase {{")?;
        writeln!(file, "\tprotected override def testCases: Seq[sertests.TestCase[_]] = Seq(")?;
        

        Ok(ScalaTestGenerator {
            file: file,
        })
    }

    fn generate_tests<'a, R: Rng>(&'a mut self, model: &'a Verilization, random: &'a mut R) -> Result<(), GeneratorError> {
        let mut state = JavaTestGenState {
            file: &mut self.file,
            random: random,
            model: model,
        };

        model.iter_types(&mut state)
    }
    
    fn end(mut self) -> Result<(), GeneratorError> {
        writeln!(self.file, "\t)")?;
        writeln!(self.file, "}}")?;
        Ok(())
    }
}