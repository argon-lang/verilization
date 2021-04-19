use verilization_compiler::{lang, model};
use lang::GeneratorError;
use model::Verilization;

use crate::memory_format::MemoryFormatWriter;
use crate::test_lang::{TestLanguage, TestGenerator};

use std::collections::HashSet;
use num_bigint::{ BigUint, BigInt };
use std::fs::File;
use std::io::Write;
use rand::Rng;
use verilization_runtime::VerilizationCodec;
use num_traits::{Zero, One};
use num_bigint::RandBigInt;

struct TSTestGenInfo<'model, F, R> {
    file: &'model mut F,
    imported_types: &'model mut HashSet<model::QualifiedName>,
    random: &'model mut R,
    model: &'model Verilization,
}

struct TSTestGenState<'model, 'state, 'scope, F, R> {
    outer: &'state mut TSTestGenInfo<'model, F, R>,
    type_name: &'model model::QualifiedName,
    type_params: &'model Vec<String>,
    scope: &'scope model::Scope<'model>,
}

fn write_random_value<F: Write, W: verilization_runtime::FormatWriter<Error = GeneratorError>, R: Rng>(f: &mut F, writer: &mut W, random: &mut R, model: &Verilization, version: &BigUint, scope: &model::Scope, t: &model::Type) -> Result<(), GeneratorError> {
    match t {
        model::Type::Nat => {
            let n: BigUint = random.gen_biguint(256);
            write!(f, "{}n", n)?;
            n.write_verilization(writer)
        },
        model::Type::Int => {
            let n: BigInt = random.gen_bigint(256);
            write!(f, "{}n", n)?;
            n.write_verilization(writer)
        },
        model::Type::U8 => {
            let n: u8 = random.gen();
            write!(f, "{}", n)?;
            n.write_verilization(writer)
        },
        model::Type::I8 => {
            let n: i8 = random.gen();
            write!(f, "{}", n)?;
            n.write_verilization(writer)
        },
        model::Type::U16 => {
            let n: u16 = random.gen();
            write!(f, "{}", n)?;
            n.write_verilization(writer)
        },
        model::Type::I16 => {
            let n: i16 = random.gen();
            write!(f, "{}", n)?;
            n.write_verilization(writer)
        },
        model::Type::U32 => {
            let n: u32 = random.gen();
            write!(f, "{}", n)?;
            n.write_verilization(writer)
        },
        model::Type::I32 => {
            let n: i32 = random.gen();
            write!(f, "{}", n)?;
            n.write_verilization(writer)
        },
        model::Type::U64 => {
            let n: u64 = random.gen();
            write!(f, "{}n", n)?;
            n.write_verilization(writer)
        },
        model::Type::I64 => {
            let n: i64 = random.gen();
            write!(f, "{}n", n)?;
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
            write!(f, "[ ")?;

            let len: u32 = random.gen_range(0..200);
            BigUint::from(len).write_verilization(writer)?;
            for _ in 0..len {
                write_random_value(f, writer, random, model, version, scope, &*inner)?;
                write!(f, ", ")?;
            }

            write!(f, "]")?;

            Ok(())
        },
        model::Type::Option(inner) => {
            let b: bool = random.gen();
            if b {
                BigUint::one().write_verilization(writer)?;
                write!(f, "{{ value: ")?;
                write_random_value(f, writer, random, model, version, scope, &*inner)?;
                write!(f, "}}")?;
            }
            else {
                BigUint::zero().write_verilization(writer)?;
                write!(f, "null")?;
            }

            Ok(())
        },
        model::Type::Defined(name, _) => match scope.lookup(name.clone()) {
            model::ScopeLookup::NamedType(name) => {
                let t = model.get_type(&name).ok_or("Could not find type")?;
                
    
                match t {
                    model::TypeDefinition::StructType(t) => {
                        let ver_type = t.versioned(version).ok_or("Could not find version of type")?;
                        write!(f, "{{ ")?;
    
                        for (field_name, field) in &ver_type.fields {
                            write!(f, "{}: ", field_name)?;
                            write_random_value(f, writer, random, model, version, scope, &field.field_type)?;
                            write!(f, ", ")?;
                        }
    
                        write!(f, "}}")?;
    
                        Ok(())
                    },
    
                    model::TypeDefinition::EnumType(t) => {
                        let ver_type = t.versioned(version).ok_or("Could not find version of type")?;
                        let index = random.gen_range(0..ver_type.fields.len());
                        let (field_name, field) = &ver_type.fields[index];
    
                        BigUint::from(index).write_verilization(writer)?;
                        write!(f, "{{ tag: \"{}\", {}: ", field_name, field_name)?;
                        write_random_value(f, writer, random, model, version, scope, &field.field_type)?;
                        write!(f, "}}")?;
    
                        Ok(())
                    },
                }
            },

            // Hardcode type parameters as u32
            model::ScopeLookup::TypeParameter(_) => write_random_value(f, writer, random, model, version, scope, &model::Type::U32),
        },
    }
}

impl <'model, F: Write, R: Rng> model::TypeDefinitionHandler<'model, GeneratorError> for TSTestGenInfo<'model, F, R> {
    type StructHandlerState<'state, 'scope> where 'model : 'scope, 'scope : 'state = TSTestGenState<'model, 'state, 'scope, F, R>;
    type EnumHandlerState<'state, 'scope> where 'model : 'scope, 'scope : 'state = TSTestGenState<'model, 'state, 'scope, F, R>;
}

impl <'model, 'state, 'scope, F: Write, R: Rng> model::TypeDefinitionHandlerState<'model, 'state, 'scope, TSTestGenInfo<'model, F, R>, GeneratorError> for TSTestGenState<'model, 'state, 'scope, F, R> where 'model : 'scope, 'scope : 'state {
    fn begin(outer: &'state mut TSTestGenInfo<'model, F, R>, type_name: &'model model::QualifiedName, type_params: &'model Vec<String>, scope: &'scope model::Scope<'model>, referenced_types: HashSet<&'model model::QualifiedName>) -> Result<Self, GeneratorError> {
        let options = lang::typescript::TypeScriptLanguage::test_options();

        let mut add_type = |t: &model::QualifiedName| -> Result<(), GeneratorError> {
            if !outer.imported_types.contains(&t) {
                let pkg_dir = options.package_mapping.get(&t.package).ok_or(format!("Unmapped package: {}", t.package))?;

                write!(outer.file, "import * as ")?;
                lang::typescript::write_import_name(outer.file, t)?;
                writeln!(outer.file, " from \"./{}/{}.js\";", pkg_dir.to_str().unwrap(), t.name)?;
                outer.imported_types.insert(t.clone());
            }

            Ok(())
        };

        add_type(type_name)?;

        for t in referenced_types {
            add_type(&t)?;
        }

        Ok(TSTestGenState {
            outer: outer,
            type_name: type_name,
            type_params: type_params,
            scope: scope,
        })
    }

    fn versioned_type(&mut self, _explicit_version: bool, version: &BigUint, _type_definition: &'model model::VersionedTypeDefinition) -> Result<(), GeneratorError> {
        write!(self.outer.file, "await check(")?;

        let type_args: Vec<_> = std::iter::repeat(model::Type::U32).take(self.type_params.len()).collect();
        let current_type = model::Type::Defined(self.type_name.clone(), type_args);

        lang::typescript::write_codec(self.outer.file, version, None, self.scope, &current_type)?;
        write!(self.outer.file, ", ")?;
        
        let mut writer = MemoryFormatWriter::new();
        write_random_value(self.outer.file, &mut writer, self.outer.random, self.outer.model, version, self.scope, &current_type)?;
        
        write!(self.outer.file, ", Uint8Array.of(")?;
        for b in writer.data() {
            write!(self.outer.file, "{},", b)?;
        }
        writeln!(self.outer.file, "));")?;


        Ok(())
    }

    fn end(self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

pub struct TSTestGenerator {
    file: File,
    imported_types: HashSet<model::QualifiedName>,
}

impl TestGenerator for TSTestGenerator {
    fn start() -> Result<TSTestGenerator, GeneratorError> {
        let mut file = File::create("../typescript/src/gen/tests.ts")?;

        writeln!(file, "import {{StandardCodecs}} from \"@verilization/runtime\";")?;
        writeln!(file, "import {{check}} from \"../check.js\";")?;
        

        Ok(TSTestGenerator {
            file: file,
            imported_types: HashSet::new(),
        })
    }

    fn generate_tests<'a, R: Rng>(&'a mut self, model: &'a Verilization, random: &'a mut R) -> Result<(), GeneratorError> {
        let mut state = crate::ts_test_gen::TSTestGenInfo {
            file: &mut self.file,
            imported_types: &mut self.imported_types,
            random: random,
            model: model,
        };

        model.iter_types(&mut state)
    }
    
    fn end(self) -> Result<(), GeneratorError> {
        Ok(())
    }
}