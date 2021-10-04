use verilization_compiler::{lang, model, VError};
use verilization_lang_scala::{ScalaGenerator, ScalaOptions};
use model::{Verilization, Named};
use lang::generator::*;

use crate::memory_format::MemoryFormatWriter;
use crate::test_lang::{TestLanguage, TestGenerator};

use num_bigint::BigUint;
use std::fs;
use std::fs::File;
use std::io::Write;
use rand::Rng;
use crate::value_generator::{generate_random_value, write_constant_value};

struct ScalaTestCaseGen<'a, F, R> {
    file: &'a mut F,
    options: &'a ScalaOptions,
    random: &'a mut R,
    model: &'a Verilization,
	type_def: Named<'a, model::VersionedTypeDefinitionData>,
	scope: model::Scope<'a>,
}


impl <'a, F: Write, R> GeneratorWithFile for ScalaTestCaseGen<'a, F, R> {
	type GeneratorFile = F;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'a, F: Write, R> Generator<'a> for ScalaTestCaseGen<'a, F, R> {
    type Lang = verilization_lang_scala::ScalaLanguage;

	fn model(&self) -> &'a model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'a> {
		&self.scope
	}
}

impl <'a, F: Write, R> ScalaGenerator<'a> for ScalaTestCaseGen<'a, F, R> {
	fn options(&self) -> &'a ScalaOptions {
		self.options
	}
}

impl <'a, F: Write, R: Rng> ScalaTestCaseGen<'a, F, R> {

    fn generate(&mut self) -> Result<(), VError> {
        for ver in self.type_def.versions() {
            self.versioned_type(&ver.version)?;
        }

        Ok(())
    }

    fn versioned_type(&mut self, version: &BigUint) -> Result<(), VError> {

        let type_args: Vec<_> = self.type_def.type_params().iter().map(|_| model::Type { name: model::QualifiedName { package: model::PackageName::new(), name: String::from("u32") }, args: Vec::new() }).collect();
        let current_type = model::Type { name: self.type_def.name().clone(), args: type_args };
        let current_lang_type = self.build_type(version, &current_type)?;

        write!(self.file, "\t\tsertests.TestCase[")?;
        self.write_type(&self.build_type(version, &current_type)?)?;
        write!(self.file, "](")?;

        self.write_expr(&self.build_codec(current_lang_type.clone())?)?;
        write!(self.file, ", ")?;
        
        let value = generate_random_value(self.random, current_lang_type.clone())?;

        self.write_expr(&self.build_value(version, current_lang_type.clone(), value.clone())?)?;
        
        let mut writer = MemoryFormatWriter::new();
        write_constant_value(&mut writer, value, current_lang_type)?;
        
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
}

pub struct ScalaTestGenerator {
    file: File,
}

impl TestGenerator for ScalaTestGenerator {
    fn start() -> Result<ScalaTestGenerator, VError> {
        fs::create_dir_all("../scala/gen-test/")?;
        let mut file = File::create("../scala/gen-test/Tests.scala")?;

        writeln!(file, "object Tests extends sertests.TestsBase {{")?;
        writeln!(file, "\tprotected override def testCases: Seq[sertests.TestCase[_]] = Seq(")?;
        

        Ok(ScalaTestGenerator {
            file: file,
        })
    }

    fn generate_tests<'a, R: Rng>(&'a mut self, model: &'a Verilization, random: &'a mut R) -> Result<(), VError> {
        let options = verilization_lang_scala::ScalaLanguage::test_options();

        for t in model.types() {
            let t = match t {
                model::NamedTypeDefinition::StructType(t) => t,
                model::NamedTypeDefinition::EnumType(t) => t,
                model::NamedTypeDefinition::ExternType(_) => continue,
                model::NamedTypeDefinition::InterfaceType(_) => continue,
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
    
    fn end(mut self) -> Result<(), VError> {
        writeln!(self.file, "\t)")?;
        writeln!(self.file, "}}")?;
        Ok(())
    }
}