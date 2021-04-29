use verilization_compiler::{lang, model};
use lang::GeneratorError;
use lang::typescript::{TSGenerator, TSOptions, make_field_name};
use lang::generator::*;
use model::{Verilization, Named};


use crate::memory_format::MemoryFormatWriter;
use crate::test_lang::{TestLanguage, TestGenerator};

use std::collections::{HashSet, HashMap};
use num_bigint::{ BigUint, BigInt };
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use rand::Rng;
use verilization_runtime::VerilizationCodec;
use num_traits::{Zero, One};
use num_bigint::RandBigInt;

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
    type Lang = lang::typescript::TypeScriptLanguage;
    
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

    fn generate(&mut self) -> Result<(), GeneratorError> {
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

    fn versioned_type(&mut self, version: &BigUint) -> Result<(), GeneratorError> {
        write!(self.file, "await check(")?;

        let type_arg_map: HashMap<_, _> = self.type_def.type_params().iter().map(|param| (param.clone(), model::Type::Defined(model::QualifiedName { package: model::PackageName::new(), name: String::from("u32") }, Vec::new()))).collect();
        let type_args: Vec<_> = type_arg_map.values().map(|arg| arg.clone()).collect();
        let current_type = model::Type::Defined(self.type_def.name().clone(), type_args);
        let current_lang_type = self.build_type(version, &current_type)?;

        self.write_expr(&self.build_codec(current_lang_type.clone())?)?;
        write!(self.file, ", ")?;
        
        let mut writer = MemoryFormatWriter::new();
        self.write_random_value(&mut writer, version, &current_type, &type_arg_map)?;
        
        write!(self.file, ", Uint8Array.of(")?;
        for b in writer.data() {
            write!(self.file, "{},", b)?;
        }
        writeln!(self.file, "));")?;


        Ok(())
    }


    fn with_scope<'a, 'model2, 'opt2, 'state2, 'output2>(&'a mut self, scope: model::Scope<'model>) -> TSTestCaseGen<'model2, 'opt2, 'state2, 'output2, F, R>
        where
            'a : 'model2 + 'opt2 + 'state2 + 'output2,
            'model : 'model2,
            'opt : 'opt2,
            'state : 'state2,
            'output : 'output2
    {
        TSTestCaseGen {
            file: self.file,
            options: self.options,
            imported_types: self.imported_types,
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
                            write!(self.file, "{}n", n)?;
                            n.write_verilization(writer)
                        },
                        ("int", []) => {
                            let n: BigInt = self.random.gen_bigint(256);
                            write!(self.file, "{}n", n)?;
                            n.write_verilization(writer)
                        },
                        ("u8", []) => {
                            let n: u8 = self.random.gen();
                            write!(self.file, "{}", n)?;
                            n.write_verilization(writer)
                        },
                        ("i8", []) => {
                            let n: i8 = self.random.gen();
                            write!(self.file, "{}", n)?;
                            n.write_verilization(writer)
                        },
                        ("u16", []) => {
                            let n: u16 = self.random.gen();
                            write!(self.file, "{}", n)?;
                            n.write_verilization(writer)
                        },
                        ("i16", []) => {
                            let n: i16 = self.random.gen();
                            write!(self.file, "{}", n)?;
                            n.write_verilization(writer)
                        },
                        ("u32", []) => {
                            let n: u32 = self.random.gen();
                            write!(self.file, "{}", n)?;
                            n.write_verilization(writer)
                        },
                        ("i32", []) => {
                            let n: i32 = self.random.gen();
                            write!(self.file, "{}", n)?;
                            n.write_verilization(writer)
                        },
                        ("u64", []) => {
                            let n: u64 = self.random.gen();
                            write!(self.file, "{}n", n)?;
                            n.write_verilization(writer)
                        },
                        ("i64", []) => {
                            let n: i64 = self.random.gen();
                            write!(self.file, "{}n", n)?;
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
                            write!(self.file, "[ ")?;
                
                            let len: u32 = self.random.gen_range(0..200);
                            BigUint::from(len).write_verilization(writer)?;
                            for _ in 0..len {
                                self.write_random_value(writer, version, &*inner, type_args)?;
                                write!(self.file, ", ")?;
                            }
                
                            write!(self.file, "]")?;
                
                            Ok(())
                        },
                        ("option", [inner]) => {
                            let b: bool = self.random.gen();
                            if b {
                                BigUint::one().write_verilization(writer)?;
                                write!(self.file, "{{ value: ")?;
                                self.write_random_value(writer, version, &*inner, type_args)?;
                                write!(self.file, "}}")?;
                            }
                            else {
                                BigUint::zero().write_verilization(writer)?;
                                write!(self.file, "null")?;
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
                                    let ver_type = t.versioned(version).ok_or("Could not find version of type")?.ver_type;
                                    write!(self.file, "{{ ")?;
                
                                    for (field_name, field) in &ver_type.fields {
                                        write!(self.file, "{}: ", make_field_name(&field_name))?;
                                        self.with_scope(t.scope()).write_random_value(writer, version, &field.field_type, &resolved_args)?;
                                        write!(self.file, ", ")?;
                                    }
                
                                    write!(self.file, "}}")?;
                
                                    Ok(())
                                },
                
                                model::NamedTypeDefinition::EnumType(t) => {
                                    let ver_type = t.versioned(version).ok_or("Could not find version of type")?.ver_type;
                                    let index = self.random.gen_range(0..ver_type.fields.len());
                                    let (field_name, field) = &ver_type.fields[index];
                
                                    BigUint::from(index).write_verilization(writer)?;
                                    write!(self.file, "{{ tag: \"{}\", {}: ", field_name, make_field_name(&field_name))?;
                                    self.with_scope(t.scope()).write_random_value(writer, version, &field.field_type, &resolved_args)?;
                                    write!(self.file, "}}")?;
                
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


pub struct TSTestGenerator {
    file: File,
    imported_types: HashSet<model::QualifiedName>,
}

impl TestGenerator for TSTestGenerator {
    fn start() -> Result<TSTestGenerator, GeneratorError> {
        let mut file = File::create("../typescript/src/gen/tests.ts")?;

        writeln!(file, "import {{check}} from \"../check.js\";")?;
        

        Ok(TSTestGenerator {
            file: file,
            imported_types: HashSet::new(),
        })
    }

    fn generate_tests<'a, R: Rng>(&'a mut self, model: &'a Verilization, random: &'a mut R) -> Result<(), GeneratorError> {
        let options = lang::typescript::TypeScriptLanguage::test_options();

        for t in model.types() {
            let t = match t {
                model::NamedTypeDefinition::StructType(t) => t,
                model::NamedTypeDefinition::EnumType(t) => t,
                model::NamedTypeDefinition::ExternType(_) => continue,
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
    
    fn end(self) -> Result<(), GeneratorError> {
        Ok(())
    }
}