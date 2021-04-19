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

struct JavaTestGenInfo<'model, F, R> {
    file: &'model mut F,
    random: &'model mut R,
    model: &'model Verilization,
}

struct JavaTestGenState<'model, 'state, 'scope, F, R> {
    info: &'state mut JavaTestGenInfo<'model, F, R>,
    type_name: &'model model::QualifiedName,
    type_params: &'model Vec<String>,
    scope: &'scope model::Scope<'model>,
}

fn write_random_value<F: Write, W: verilization_runtime::FormatWriter<Error = GeneratorError>, R: Rng>(f: &mut F, writer: &mut W, random: &mut R, model: &Verilization, version: &BigUint, scope: &model::Scope, t: &model::Type) -> Result<(), GeneratorError> {
    match t {
        model::Type::Nat => {
            let n: BigUint = random.gen_biguint(256);
            write!(f, "new java.lang.BigInteger(\"{}\")", n)?;
            n.write_verilization(writer)
        },
        model::Type::Int => {
            let n: BigInt = random.gen_bigint(256);
            write!(f, "new java.lang.BigInteger(\"{}\")", n)?;
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
                write_random_value(f, writer, random, model, version, scope, &*inner)?;
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
                write_random_value(f, writer, random, model, version, scope, &*inner)?;
                write!(f, ")")?;
            }
            else {
                BigUint::zero().write_verilization(writer)?;
                write!(f, "java.util.Optional.empty()")?;
            }

            Ok(())
        },
        model::Type::Defined(name, _) => match scope.lookup(name.clone()) {
            model::ScopeLookup::NamedType(name) => {
                let t = model.get_type(&name).ok_or("Could not find type")?;
                let options = lang::java::JavaLanguage::test_options();
                
    
                match t {
                    model::TypeDefinition::StructType(t) => {
                        let ver_type = t.versioned(version).ok_or("Could not find version of type")?;
                        write!(f, "new ")?;
                        lang::java::write_qual_name(f, &options.package_mapping, &name)?;
                        write!(f, ".V{}(", version)?;
    
                        {
                            let mut iter = ver_type.fields.iter();
                            if let Some((_, field)) = iter.next() {
                                write_random_value(f, writer, random, model, version, scope, &field.field_type)?;
                                while let Some((_, field)) = iter.next() {
                                    write!(f, ", ")?;
                                    write_random_value(f, writer, random, model, version, scope, &field.field_type)?;
                                }
                            }
                        }
                        write!(f, ")")?;
    
                        Ok(())
                    },
    
                    model::TypeDefinition::EnumType(t) => {
                        let ver_type = t.versioned(version).ok_or("Could not find version of type")?;
                        let index = random.gen_range(0..ver_type.fields.len());
                        let (field_name, field) = &ver_type.fields[index];
    
                        BigUint::from(index).write_verilization(writer)?;
                        write!(f, "new ")?;
                        lang::java::write_qual_name(f, &options.package_mapping, &name)?;
                        write!(f, ".V{}.{}(", version, field_name)?;
                        write_random_value(f, writer, random, model, version, scope, &field.field_type)?;
                        write!(f, ")")?;
    
                        Ok(())
                    },
                }

            },

            // Hardcode type parameters as u32
            model::ScopeLookup::TypeParameter(_) => write_random_value(f, writer, random, model, version, scope, &model::Type::U32),
        },
    }
}

impl <'model, F: Write, R: Rng> model::TypeDefinitionHandler<'model, GeneratorError> for JavaTestGenInfo<'model, F, R> {
    type StructHandlerState<'state, 'scope> where 'model : 'scope, 'scope : 'state = JavaTestGenState<'model, 'state, 'scope, F, R>;
    type EnumHandlerState<'state, 'scope> where 'model : 'scope, 'scope : 'state = JavaTestGenState<'model, 'state, 'scope, F, R>;
}

impl <'model, 'state, 'scope, F: Write, R: Rng> model::TypeDefinitionHandlerState<'model, 'state, 'scope, JavaTestGenInfo<'model, F, R>, GeneratorError> for JavaTestGenState<'model, 'state, 'scope, F, R> where 'model : 'state {
    fn begin(outer: &'state mut JavaTestGenInfo<'model, F, R>, type_name: &'model model::QualifiedName, type_params: &'model Vec<String>, scope: &'scope model::Scope<'model>, _referenced_types: HashSet<&'model model::QualifiedName>) -> Result<Self, GeneratorError> {
        Ok(JavaTestGenState {
            info: outer,
            type_name: type_name,
            type_params: type_params,
            scope: scope,
        })
    }

    fn versioned_type(&mut self, _explicit_version: bool, version: &BigUint, _type_definition: &'model model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
        write!(self.info.file, "\t\tcheck(")?;

        let options = lang::java::JavaLanguage::test_options();

        let type_args: Vec<_> = std::iter::repeat(model::Type::U32).take(self.type_params.len()).collect();
        let current_type = model::Type::Defined(self.type_name.clone(), type_args);

        lang::java::write_codec(self.info.file, &options.package_mapping, version, self.scope, &current_type)?;
        write!(self.info.file, ", ")?;
        
        let mut writer = MemoryFormatWriter::new();
        write_random_value(self.info.file, &mut writer, self.info.random, self.info.model, version, self.scope, &current_type)?;
        
        write!(self.info.file, ", new byte[] {{")?;
        for b in writer.data() {
            write!(self.info.file, "{},", b as i8)?;
        }
        writeln!(self.info.file, "}});")?;


        Ok(())
    }

    fn end(self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

pub struct JavaTestGenerator {
    file: File,
}

impl TestGenerator for JavaTestGenerator {
    fn start() -> Result<JavaTestGenerator, GeneratorError> {
        fs::create_dir_all("../java/gen-test/")?;
        let mut file = File::create("../java/gen-test/Tests.java")?;

        writeln!(file, "class Tests extends sertests.TestsBase {{")?;
        writeln!(file, "\t@org.junit.jupiter.api.Test")?;
        writeln!(file, "\tvoid test() throws java.io.IOException {{")?;
        

        Ok(JavaTestGenerator {
            file: file,
        })
    }

    fn generate_tests<'a, R: Rng>(&'a mut self, model: &'a Verilization, random: &'a mut R) -> Result<(), GeneratorError> {
        let mut state = JavaTestGenInfo {
            file: &mut self.file,
            random: random,
            model: model,
        };

        model.iter_types(&mut state)
    }
    
    fn end(mut self) -> Result<(), GeneratorError> {
        writeln!(self.file, "\t}}")?;
        writeln!(self.file, "}}")?;
        Ok(())
    }
}