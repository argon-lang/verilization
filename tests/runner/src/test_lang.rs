use verilization_compiler::{lang, model};
use lang::{Language, GeneratorError};
use model::PackageName;
use model::Verilization;

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
    fn test_options_dir(dir: OsString) -> Self::Options;
    fn append_options(command: &mut Command, options: &Self::Options);
    fn test_command() -> Command;

}

pub trait TestGenerator : Sized {
    fn start() -> Result<Self, GeneratorError>;
    fn generate_tests<'a, R: Rng>(&'a mut self, model: &'a Verilization, random: &'a mut R) -> Result<(), GeneratorError>;
    fn end(self) -> Result<(), GeneratorError>;
}

impl TestLanguage for lang::typescript::TypeScriptLanguage {
    type TestGen = crate::ts_test_gen::TSTestGenerator;

    fn name() -> String {
        String::from("typescript")
    }
    
    fn test_options() -> Self::Options {
        Self::test_options_dir(OsString::from("../typescript/src/gen/"))
    }
    
    fn test_options_dir(dir: OsString) -> Self::Options {
        lang::typescript::TSOptions {
            output_dir: dir,
            package_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&["struct", "versions"]), OsString::from("struct/versions") ),
                ( PackageName::from_parts(&["enum", "versions"]), OsString::from("enum/versions") ),
            ]))
        }
    }
    
    fn append_options(command: &mut Command, options: &Self::Options) {
        command.arg("-o:out_dir");
        command.arg(&options.output_dir);
        for (pkg, dir) in &options.package_mapping {
            command.arg(format!("-o:pkg:{}", pkg));
            command.arg(dir);
        }
    }
    
    fn test_command() -> Command {
        let mut cmd = Command::new("npm");
        cmd.arg("test");
        cmd.current_dir("../typescript");
        cmd
    }


}

impl TestLanguage for lang::java::JavaLanguage {
    type TestGen = crate::java_test_gen::JavaTestGenerator;

    fn name() -> String {
        String::from("java")
    }
    
    fn test_options() -> Self::Options {
        Self::test_options_dir(OsString::from("../java/gen/"))
    }
    
    fn test_options_dir(dir: OsString) -> Self::Options {
        lang::java::JavaOptions {
            output_dir: dir,
            package_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&["struct", "versions"]), PackageName::from_parts(&["struct", "versions"]) ),
                ( PackageName::from_parts(&["enum", "versions"]), PackageName::from_parts(&["enum_", "versions"]) ),
            ]))
        }
    }
    
    fn append_options(command: &mut Command, options: &Self::Options) {
        command.arg("-o:out_dir");
        command.arg(&options.output_dir);
        for (pkg, java_pkg) in &options.package_mapping {
            command.arg(format!("-o:pkg:{}", pkg));
            command.arg(format!("{}", java_pkg));
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
    type TestGen = crate::scala_test_gen::ScalaTestGenerator;

    fn name() -> String {
        String::from("scala")
    }

    fn test_options() -> Self::Options {
        Self::test_options_dir(OsString::from("../scala/gen/"))
    }

    fn test_options_dir(dir: OsString) -> Self::Options {
        lang::scala::ScalaOptions {
            output_dir: dir,
            package_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&["struct", "versions"]), PackageName::from_parts(&["struct", "versions"]) ),
                ( PackageName::from_parts(&["enum", "versions"]), PackageName::from_parts(&["enum_", "versions"]) ),
            ]))
        }
    }
    
    fn append_options(command: &mut Command, options: &Self::Options) {
        command.arg("-o:out_dir");
        command.arg(&options.output_dir);
        for (pkg, scala_pkg) in &options.package_mapping {
            command.arg(format!("-o:pkg:{}", pkg));
            command.arg(format!("{}", scala_pkg));
        }
    }
    
    fn test_command() -> Command {
        let mut cmd = Command::new("sbt");
        cmd.arg("test");
        cmd.current_dir("../scala");
        cmd
    }
}

