use verilization_compiler::{lang, model, VError};
use lang::GeneratorError;
use verilization_lang_typescript::{TSGenerator, TSOptions};
use lang::generator::*;
use model::{Verilization, Named};
use crate::value_generator::{generate_random_value, write_constant_value};


use crate::memory_format::MemoryFormatWriter;
use crate::test_lang::{TestLanguage, TestGenerator};

use std::collections::HashSet;
use num_bigint::BigUint;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use rand::Rng;

struct TSTestCaseGen<'model, 'opt, 'state, 'output, F, R> {
    file: &'output mut F,
    options: &'opt TSOptions,
    imported_types: &'state mut HashSet<model::QualifiedName>,
    random: &'model mut R,
    model: &'model Verilization,
	type_def: Named<'model, model::VersionedTypeDefinitionData>,
	scope: model::Scope<'model>,
}

impl <'model, 'opt, 'state, 'output, F: Write, R> GeneratorWithFile for TSTestCaseGen<'model, 'opt, 'state, 'output, F, R> {
	type GeneratorFile = F;
	fn file(&mut self) -> &mut Self::GeneratorFile {
		&mut self.file
	}
}

impl <'model, 'opt, 'state, 'output, F: Write, R> Generator<'model> for TSTestCaseGen<'model, 'opt, 'state, 'output, F, R> {
    type Lang = verilization_lang_typescript::TypeScriptLanguage;

	fn model(&self) -> &'model model::Verilization {
		self.model
	}

	fn scope(&self) -> &model::Scope<'model> {
		&self.scope
	}
}

impl <'model, 'opt, 'state, 'output, F: Write, R> TSGenerator<'model> for TSTestCaseGen<'model, 'opt, 'state, 'output, F, R> {

	fn generator_element_name(&self) -> Option<&'model model::QualifiedName> {
		None
	}

	fn options(&self) -> &TSOptions {
		self.options
	}

	fn referenced_types(&self) -> model::ReferencedTypeIterator<'model> {
		self.type_def.referenced_types()
	}

	fn current_dir(&self) -> Result<PathBuf, GeneratorError> {
		Ok(PathBuf::from(&self.options.output_dir))
	}

	fn add_user_converter(&mut self, _name: String) {}
}

impl <'model, 'opt, 'state, 'output, F: Write, R: Rng> TSTestCaseGen<'model, 'opt, 'state, 'output, F, R> {

    fn generate(&mut self) -> Result<(), VError> {
        self.add_imported_type(&model::QualifiedName::from_parts(&[], "u32"))?;
        self.add_imported_type(self.type_def.name())?;

        for t in self.type_def.referenced_types() {
            self.add_imported_type(&t)?;
        }
        
        for ver in self.type_def.versions() {
            self.versioned_type(&ver.version)?;
        }

        Ok(())
    }

    fn add_imported_type(&mut self, t: &model::QualifiedName) -> Result<(), GeneratorError> {
        let t = match self.type_def.scope().lookup(t.clone()) {
            model::ScopeLookup::TypeParameter(_) => return Ok(()),
            model::ScopeLookup::NamedType(t) => t,
        };

        if !self.imported_types.contains(&t) {
            self.write_import(&t, &self.options().output_dir.clone())?;
            self.imported_types.insert(t.clone());
        }

        Ok(())
    }

    fn versioned_type(&mut self, version: &BigUint) -> Result<(), VError> {
        write!(self.file, "await check(")?;

        let type_args: Vec<_> = self.type_def.type_params().iter().map(|_| model::Type { name: model::QualifiedName { package: model::PackageName::new(), name: String::from("u32") }, args: Vec::new() }).collect();
        let current_type = model::Type { name: self.type_def.name().clone(), args: type_args };
        let current_lang_type = self.build_type(version, &current_type)?;

        self.write_expr(&self.build_codec(current_lang_type.clone())?)?;
        write!(self.file, ", ")?;
        
        let value = generate_random_value(self.random, current_lang_type.clone())?;

        self.write_expr(&self.build_value(version, current_lang_type.clone(), value.clone())?)?;
        
        let mut writer = MemoryFormatWriter::new();
        write_constant_value(&mut writer, value, current_lang_type)?;
        
        write!(self.file, ", Uint8Array.of(")?;
        for b in writer.data() {
            write!(self.file, "{},", b)?;
        }
        writeln!(self.file, "));")?;


        Ok(())
    }

}


pub struct TSTestGenerator {
    file: File,
    imported_types: HashSet<model::QualifiedName>,
}

impl TestGenerator for TSTestGenerator {
    fn start() -> Result<TSTestGenerator, VError> {
        let mut file = File::create("../typescript/src/gen/tests.ts")?;

        writeln!(file, "import {{check}} from \"../check.js\";")?;
        

        Ok(TSTestGenerator {
            file: file,
            imported_types: HashSet::new(),
        })
    }

    fn generate_tests<'a, R: Rng>(&'a mut self, model: &'a Verilization, random: &'a mut R) -> Result<(), VError> {
        let options = verilization_lang_typescript::TypeScriptLanguage::test_options();

        for t in model.types() {
            let t = match t {
                model::NamedTypeDefinition::StructType(t) => t,
                model::NamedTypeDefinition::EnumType(t) => t,
                model::NamedTypeDefinition::ExternType(_) => continue,
                model::NamedTypeDefinition::InterfaceType(_) => continue,
            };

            let mut gen = TSTestCaseGen {
                file: &mut self.file,
                options: &options,
                imported_types: &mut self.imported_types,
                random: random,
                model: model,
                type_def: t,
                scope: model::Scope::empty(model),
            };

            gen.generate()?;
        }

        Ok(())
    }
    
    fn end(self) -> Result<(), VError> {
        Ok(())
    }
}