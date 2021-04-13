use verilization_compiler::{lang, model};
use lang::{Language, GeneratorError};
use model::PackageName;
use model::Verilization;

use crate::ts_test_gen::TSTestGenerator;

use std::collections::HashMap;
use core::array::IntoIter;
use std::ffi::OsString;
use std::iter::FromIterator;
use std::process::Command;
use rand::Rng;


pub trait TestLanguage: Language {
    type TestGen : TestGenerator;

    fn name() -> String;
    fn test_options() -> Self::Options;
    fn test_command() -> Command;
}

pub trait TestGenerator : Sized {
    fn start() -> Result<Self, GeneratorError>;
    fn generate_tests<'a, R: Rng>(&'a mut self, model: &'a Verilization, random: &'a mut R) -> Result<(), GeneratorError>;
    fn end(self) -> Result<(), GeneratorError>;
}

impl TestLanguage for lang::typescript::TypeScriptLanguage {
    type TestGen = TSTestGenerator;

    fn name() -> String {
        String::from("typescript")
    }
    
    fn test_options() -> Self::Options {
        lang::typescript::TSOptions {
            output_dir: OsString::from("../typescript/src/gen/"),
            package_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&["struct", "versions"]), OsString::from("struct/versions") ),
                ( PackageName::from_parts(&["enum", "versions"]), OsString::from("enum/versions") ),
            ]))
        }
    }
    
    fn test_command() -> Command {
        let mut cmd = Command::new("npm");
        cmd.arg("test");
        cmd.current_dir("../typescript");
        cmd
    }


}


impl TestGenerator for () {
    fn start() -> Result<Self, GeneratorError> {
        Ok(())
    }
    fn generate_tests<R: Rng>(&mut self, model: &Verilization, random: &mut R) -> Result<(), GeneratorError> {
        Ok(())
    }
    fn end(self) -> Result<(), GeneratorError> {
        Ok(())
    }
}


impl TestLanguage for lang::java::JavaLanguage {
    type TestGen = ();

    fn name() -> String {
        String::from("java")
    }
    
    fn test_options() -> Self::Options {
        lang::java::JavaOptions {
            output_dir: OsString::from("../java/gen/"),
            package_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&["struct", "versions"]), PackageName::from_parts(&["struct", "versions"]) ),
                ( PackageName::from_parts(&["enum", "versions"]), PackageName::from_parts(&["enum_", "versions"]) ),
            ]))
        }
    }

    fn test_command() -> Command {
        let mut cmd = Command::new("sbt");
        cmd.arg("test");
        cmd.current_dir("../java");
        cmd
    }
}

impl TestLanguage for lang::scala::ScalaLanguage {
    type TestGen = ();

    fn name() -> String {
        String::from("scala")
    }

    fn test_options() -> Self::Options {
        lang::scala::ScalaOptions {
            output_dir: OsString::from("../scala/gen/"),
            package_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&["struct", "versions"]), PackageName::from_parts(&["struct", "versions"]) ),
                ( PackageName::from_parts(&["enum", "versions"]), PackageName::from_parts(&["enum_", "versions"]) ),
            ]))
        }
    }
    
    fn test_command() -> Command {
        let mut cmd = Command::new("sbt");
        cmd.arg("test");
        cmd.current_dir("../scala");
        cmd
    }
}

