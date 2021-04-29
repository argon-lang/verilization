use verilization_compiler::{lang, model};
use lang::GeneratorError;
use lang::scala::{ScalaGenerator, ScalaOptions};
use model::{Verilization, Named};
use lang::generator::*;

use crate::memory_format::MemoryFormatWriter;
use crate::test_lang::{TestLanguage, TestGenerator};

use std::collections::HashMap;
use num_bigint::BigUint;
use std::fs;
use std::fs::File;
use std::io::Write;
use rand::Rng;
use crate::value_generator::{generate_random_value, write_constant_value};

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

impl <'model, 'opt, 'state, 'output, F: Write, R> Generator<'model> for ScalaTestCaseGen<'model, 'opt, 'output, F, R> {
    type Lang = lang::scala::ScalaLanguage;

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