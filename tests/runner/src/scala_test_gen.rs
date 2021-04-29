use verilization_compiler::{lang, model, for_sep};
use lang::GeneratorError;
use lang::scala::{ScalaGenerator, ScalaOptions, make_type_name};
use model::{Verilization, Named};
use lang::generator::*;

use crate::memory_format::MemoryFormatWriter;
use crate::test_lang::{TestLanguage, TestGenerator};

use std::collections::HashMap;
use num_bigint::{ BigUint, BigInt };
use std::fs;
use std::fs::File;
use std::io::Write;
use rand::Rng;
use verilization_runtime::VerilizationCodec;
use num_traits::{Zero, One};
use num_bigint::RandBigInt;

struct ScalaTestCaseGen<'model, 'opt, 'output, F, R> {
    file: &'output mut F,
    options: &'opt ScalaOptions,
    random: &'model mut R,
    model: &'model Verilization,
	type_def: Named<'model, model::VersionedTypeDefinitionData>,
	scope: model::Scope<'model>,
}


impl <'model, 'opt, 'state, 'output, F: Write, R> GeneratorWithFile for ScalaTestCaseGen<'model, 'opt, 'output, F, R> {
	type GeneratorFile = F;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'model, 'opt, 'state, 'output, F: Write, R> Generator<'model, lang::scala::ScalaLanguage> for ScalaTestCaseGen<'model, 'opt, 'output, F, R> {
	fn model(&self) -> &'model model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}
}

impl <'model, 'opt, 'output, F: Write, R> ScalaGenerator<'model, 'opt> for ScalaTestCaseGen<'model, 'opt, 'output, F, R> {
	fn options(&self) -> &'opt ScalaOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.type_def.referenced_types()
	}
}

impl <'model, 'opt, 'output, F: Write, R: Rng> ScalaTestCaseGen<'model, 'opt, 'output, F, R> {

    fn generate(&mut self) -> Result<(), GeneratorError> {
        for ver in self.type_def.versions() {
            self.versioned_type(&ver.version)?;
        }

        Ok(())
    }

    fn versioned_type(&mut self, version: &BigUint) -> Result<(), GeneratorError> {

        let type_arg_map: HashMap<_, _> = self.type_def.type_params().iter().map(|param| (param.clone(), model::Type::Defined(model::QualifiedName { package: model::PackageName::new(), name: String::from("u32") }, Vec::new()))).collect();
        let type_args: Vec<_> = type_arg_map.values().map(|arg| arg.clone()).collect();
        let current_type = model::Type::Defined(self.type_def.name().clone(), type_args);
        let current_lang_type = self.build_type(version, &current_type)?;

        write!(self.file, "\t\tsertests.TestCase[")?;
        self.write_type(&self.build_type(version, &current_type)?)?;
        write!(self.file, "](")?;

        self.write_expr(&self.build_codec(current_lang_type.clone())?)?;
        write!(self.file, ", ")?;
        
        let mut writer = MemoryFormatWriter::new();
        self.write_random_value(&mut writer, version, &current_type, &type_arg_map)?;
        
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

    fn with_scope<'a, 'model2, 'opt2, 'output2>(&'a mut self, scope: model::Scope<'model>) -> ScalaTestCaseGen<'model2, 'opt2, 'output2, F, R>
        where
            'a : 'model2 + 'opt2 + 'output2,
            'model : 'model2,
            'opt : 'opt2,
            'output : 'output2
    {
        ScalaTestCaseGen {
            file: self.file,
            options: self.options,
            random: self.random,
            model: self.model,
            type_def: self.type_def,
            scope: scope,
        }
    }

    fn write_random_value<W: verilization_runtime::FormatWriter<Error = GeneratorError>>(&mut self, writer: &mut W, version: &BigUint, t: &model::Type, type_args: &HashMap<String, model::Type>) -> Result<(), GeneratorError> {
        match t {
            model::Type::Defined(name, args) => match self.scope.lookup(name.clone()) {
                model::ScopeLookup::NamedType(name) => {
                    match (name.name.as_ref(), &args[..]) {
                        ("nat", []) => {
                            let n: BigUint = self.random.gen_biguint(256);
                            write!(self.file, "new java.math.BigInteger(\"{}\")", n)?;
                            n.write_verilization(writer)
                        },
                        ("int", []) => {
                            let n: BigInt = self.random.gen_bigint(256);
                            write!(self.file, "new java.math.BigInteger(\"{}\")", n)?;
                            n.write_verilization(writer)
                        },
                        ("u8", []) | ("i8", []) => {
                            let n: i8 = self.random.gen();
                            write!(self.file, "{}", n)?;
                            n.write_verilization(writer)
                        },
                        ("u16", []) | ("i16", []) => {
                            let n: i16 = self.random.gen();
                            write!(self.file, "{}", n)?;
                            n.write_verilization(writer)
                        },
                        ("u32", []) | ("i32", []) => {
                            let n: i32 = self.random.gen();
                            write!(self.file, "{}", n)?;
                            n.write_verilization(writer)
                        },
                        ("u64", []) | ("i64", []) => {
                            let n: i64 = self.random.gen();
                            write!(self.file, "{}L", n)?;
                            n.write_verilization(writer)
                        },
                        ("string", []) => {
                            let charset = random_string::Charset::new("abcdefABCDEF0123456789").unwrap();
                            let len = self.random.gen_range(0..2000);
                            let s = random_string::generate(len, &charset).to_string();
                            write!(self.file, "\"{}\"", s)?;
                            s.write_verilization(writer)?;
                
                            Ok(())
                        },
                        ("list", [inner]) => {
                            write!(self.file, "java.util.List.of(")?;
                
                            let len: u32 = self.random.gen_range(0..200);
                            BigUint::from(len).write_verilization(writer)?;
                            for _ in 0..len {
                                self.write_random_value(writer, version, &inner, type_args)?;
                                write!(self.file, ", ")?;
                            }
                
                            write!(self.file, ")")?;
                
                            Ok(())
                        },
                        ("option", [inner]) => {
                            let b: bool = self.random.gen();
                            if b {
                                BigUint::one().write_verilization(writer)?;
                                write!(self.file, "java.util.Optional.of(")?;
                                self.write_random_value(writer, version, &inner, type_args)?;
                                write!(self.file, ")")?;
                            }
                            else {
                                BigUint::zero().write_verilization(writer)?;
                                write!(self.file, "java.util.Optional.empty()")?;
                            }
                
                            Ok(())
                        },
                        _ => {
                            let t = self.model.get_type(&name).ok_or("Could not find type")?;
                            
                            let resolved_args = t.type_params().iter().zip(args.iter())
                                .map(|(param, arg)| Some((param.clone(), self.scope.resolve(arg.clone(), type_args)?)))
                                .collect::<Option<HashMap<_, _>>>()
                                .ok_or("Could not resolve args")?;
                
                            match t {
                                model::NamedTypeDefinition::StructType(t) => {
                                    let ver_type = t.versioned(version).ok_or("Could not find version of type")?;
                                    write!(self.file, "new ")?;
                                    self.write_qual_name(&name)?;
                                    write!(self.file, ".V{}", ver_type.version)?;
                                    self.write_type_args(&args.iter().map(|arg| self.build_type(version, arg)).collect::<Result<Vec<_>, _>>()?)?;
                                    write!(self.file, "(")?;
                
                                    for_sep!((_, field), &ver_type.ver_type.fields, { write!(self.file, ", ")?; }, {
                                        self.with_scope(t.scope()).write_random_value(writer, version, &field.field_type, &resolved_args)?;
                                    });

                                    write!(self.file, ")")?;
                
                                    Ok(())
                                },
                
                                model::NamedTypeDefinition::EnumType(t) => {
                                    let ver_type = t.versioned(version).ok_or("Could not find version of type")?;
                                    let index = self.random.gen_range(0..ver_type.ver_type.fields.len());
                                    let (field_name, field) = &ver_type.ver_type.fields[index];
                
                                    BigUint::from(index).write_verilization(writer)?;
                                    write!(self.file, "new ")?;
                                    self.write_qual_name(&name)?;
                                    write!(self.file, ".V{}.{}", ver_type.version, make_type_name(field_name))?;
                                    self.write_type_args(&args.iter().map(|arg| self.build_type(version, arg)).collect::<Result<Vec<_>, _>>()?)?;
                                    write!(self.file, "(")?;
                                    self.with_scope(t.scope()).write_random_value(writer, version, &field.field_type, &resolved_args)?;
                                    write!(self.file, ")")?;
                
                                    Ok(())
                                },

                                model::NamedTypeDefinition::ExternType(_) => {
                                    Err(GeneratorError::from("Extern types not implemented"))
                                },
                            }
                        },
                    }
                },
    
                model::ScopeLookup::TypeParameter(name) => {
                    let t = type_args.get(&name).ok_or("Unknown type parameter")?;
                    self.with_scope(model::Scope::empty(self.model)).write_random_value(writer, version, t, &HashMap::new())
                },
            },
        }
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
        let options = lang::scala::ScalaLanguage::test_options();

        for t in model.types() {
            let t = match t {
                model::NamedTypeDefinition::StructType(t) => t,
                model::NamedTypeDefinition::EnumType(t) => t,
                model::NamedTypeDefinition::ExternType(_) => continue,
            };

            let mut gen = ScalaTestCaseGen {
                file: &mut self.file,
                options: &options,
                random: random,
                model: model,
                type_def: t,
                scope: model::Scope::empty(model),
            };
    
            gen.generate()?;
        }

        Ok(())
    }
    
    fn end(mut self) -> Result<(), GeneratorError> {
        writeln!(self.file, "\t)")?;
        writeln!(self.file, "}}")?;
        Ok(())
    }
}