use verilization_compiler::{lang, model, for_sep};
use lang::GeneratorError;
use lang::scala::{ScalaGenerator, ScalaOptions};
use model::{Verilization, Named};

use crate::memory_format::MemoryFormatWriter;
use crate::test_lang::{TestLanguage, TestGenerator};

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
	type_def: Named<'model, model::TypeDefinitionData>,
	scope: model::Scope<'model>,
}


impl <'model, 'opt, 'output, F: Write, R> ScalaGenerator<'model, 'opt> for ScalaTestCaseGen<'model, 'opt, 'output, F, R> {
	type GeneratorFile = F;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}

	fn options(&self) -> &'opt ScalaOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.type_def.referenced_types()
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
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

        let type_args: Vec<_> = std::iter::repeat(model::Type::U32).take(self.type_def.type_params().len()).collect();
        let current_type = model::Type::Defined(self.type_def.name().clone(), type_args);

        write!(self.file, "\t\tsertests.TestCase[")?;
        self.write_type(version, &current_type)?;
        write!(self.file, "](")?;

        self.write_codec(version, &current_type)?;
        write!(self.file, ", ")?;
        
        let mut writer = MemoryFormatWriter::new();
        self.write_random_value(&mut writer, version, &current_type)?;
        
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

    fn write_random_value<W: verilization_runtime::FormatWriter<Error = GeneratorError>>(&mut self, writer: &mut W, version: &BigUint, t: &model::Type) -> Result<(), GeneratorError> {
        match t {
            model::Type::Nat => {
                let n: BigUint = self.random.gen_biguint(256);
                write!(self.file, "new java.math.BigInteger(\"{}\")", n)?;
                n.write_verilization(writer)
            },
            model::Type::Int => {
                let n: BigInt = self.random.gen_bigint(256);
                write!(self.file, "new java.math.BigInteger(\"{}\")", n)?;
                n.write_verilization(writer)
            },
            model::Type::U8 | model::Type::I8 => {
                let n: i8 = self.random.gen();
                write!(self.file, "{}", n)?;
                n.write_verilization(writer)
            },
            model::Type::U16 | model::Type::I16 => {
                let n: i16 = self.random.gen();
                write!(self.file, "{}", n)?;
                n.write_verilization(writer)
            },
            model::Type::U32 | model::Type::I32 => {
                let n: i32 = self.random.gen();
                write!(self.file, "{}", n)?;
                n.write_verilization(writer)
            },
            model::Type::U64 | model::Type::I64 => {
                let n: i64 = self.random.gen();
                write!(self.file, "{}L", n)?;
                n.write_verilization(writer)
            },
            model::Type::String => {
                let charset = random_string::Charset::new("abcdefABCDEF0123456789").unwrap();
                let len = self.random.gen_range(0..2000);
                let s = random_string::generate(len, &charset).to_string();
                write!(self.file, "\"{}\"", s)?;
                s.write_verilization(writer)?;
    
                Ok(())
            },
            model::Type::List(inner) => {
                write!(self.file, "java.util.List.of(")?;
    
                let len: u32 = self.random.gen_range(0..200);
                BigUint::from(len).write_verilization(writer)?;
                for _ in 0..len {
                    self.write_random_value(writer, version, &*inner)?;
                    write!(self.file, ", ")?;
                }
    
                write!(self.file, ")")?;
    
                Ok(())
            },
            model::Type::Option(inner) => {
                let b: bool = self.random.gen();
                if b {
                    BigUint::one().write_verilization(writer)?;
                    write!(self.file, "java.util.Optional.of(")?;
                    self.write_random_value(writer, version, &*inner)?;
                    write!(self.file, ")")?;
                }
                else {
                    BigUint::zero().write_verilization(writer)?;
                    write!(self.file, "java.util.Optional.empty()")?;
                }
    
                Ok(())
            },
            model::Type::Defined(name, _) => match self.scope.lookup(name.clone()) {
                model::ScopeLookup::NamedType(name) => {
                    let t = self.model.get_type(&name).ok_or("Could not find type")?;
        
                    match t {
                        model::NamedTypeDefinition::StructType(t) => {
                            let ver_type = t.versioned(version).ok_or("Could not find version of type")?.ver_type;
                            write!(self.file, "new ")?;
                            self.write_qual_name(&name)?;
                            write!(self.file, ".V{}(", version)?;
        
                            for_sep!((_, field), &ver_type.fields, { write!(self.file, ", ")?; }, {
                                self.with_scope(t.scope()).write_random_value(writer, version, &field.field_type)?;
                            });

                            write!(self.file, ")")?;
        
                            Ok(())
                        },
        
                        model::NamedTypeDefinition::EnumType(t) => {
                            let ver_type = t.versioned(version).ok_or("Could not find version of type")?.ver_type;
                            let index = self.random.gen_range(0..ver_type.fields.len());
                            let (field_name, field) = &ver_type.fields[index];
        
                            BigUint::from(index).write_verilization(writer)?;
                            write!(self.file, "new ")?;
                            self.write_qual_name(&name)?;
                            write!(self.file, ".V{}.{}(", version, field_name)?;
                            self.with_scope(t.scope()).write_random_value(writer, version, &field.field_type)?;
                            write!(self.file, ")")?;
        
                            Ok(())
                        },
                    }
                },
    
                // Hardcode type parameters as u32
                model::ScopeLookup::TypeParameter(_) => self.write_random_value(writer, version, &model::Type::U32),
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