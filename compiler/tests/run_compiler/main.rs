
use verilization_compiler::{lang, model, file_output_handler};
use lang::{Language, GeneratorError};
use model::PackageName;
use model::Verilization;


use std::collections::HashMap;
use core::array::IntoIter;
use std::ffi::OsString;
use std::iter::FromIterator;
use std::process::Command;
use std::io::Write;

trait TestLanguage: Language {
    fn name() -> String;
    fn test_options() -> Self::Options;
    fn test_command() -> Command;
}

impl TestLanguage for lang::typescript::TypeScriptLanguage {
    fn name() -> String {
        String::from("typescript")
    }
    fn test_options() -> Self::Options {
        lang::typescript::TSOptions {
            output_dir: OsString::from("../tests/typescript/src/gen/"),
            package_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&["struct", "versions"]), OsString::from("struct/versions") ),
                ( PackageName::from_parts(&["enum", "versions"]), OsString::from("enum/versions") ),
            ]))
        }
    }
    fn test_command() -> Command {
        let mut cmd = Command::new("npm");
        cmd.arg("test");
        cmd.current_dir("../tests/typescript");
        cmd
    }
}

impl TestLanguage for lang::java::JavaLanguage {
    fn name() -> String {
        String::from("java")
    }
    fn test_options() -> Self::Options {
        lang::java::JavaOptions {
            output_dir: OsString::from("../tests/java/gen/"),
            package_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&["struct", "versions"]), PackageName::from_parts(&["struct", "versions"]) ),
                ( PackageName::from_parts(&["enum", "versions"]), PackageName::from_parts(&["enum_", "versions"]) ),
            ]))
        }
    }
    fn test_command() -> Command {
        let mut cmd = Command::new("sbt");
        cmd.arg("test");
        cmd.current_dir("../tests/java");
        cmd
    }
}

impl TestLanguage for lang::scala::ScalaLanguage {
    fn name() -> String {
        String::from("scala")
    }
    fn test_options() -> Self::Options {
        lang::scala::ScalaOptions {
            output_dir: OsString::from("../tests/scala/gen/"),
            package_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&["struct", "versions"]), PackageName::from_parts(&["struct", "versions"]) ),
                ( PackageName::from_parts(&["enum", "versions"]), PackageName::from_parts(&["enum_", "versions"]) ),
            ]))
        }
    }
    fn test_command() -> Command {
        let mut cmd = Command::new("sbt");
        cmd.arg("test");
        cmd.current_dir("../tests/scala");
        cmd
    }

}

const TEST_CASE_FILES: &[&str] = &[
    "struct_versions",
    "enum_versions",
];


trait LanguageIterHandler {
    fn run<Lang : TestLanguage>(&self) -> Result<(), GeneratorError>;
}

fn each_language<Handler : LanguageIterHandler>(handler: &Handler) -> Result<(), GeneratorError> {
    handler.run::<lang::typescript::TypeScriptLanguage>()?;
    handler.run::<lang::java::JavaLanguage>()?;
    handler.run::<lang::scala::ScalaLanguage>()?;
    Ok(())
}


struct GenerateCodeHandler<'a> {
    model: &'a Verilization,
}

impl <'a> LanguageIterHandler for GenerateCodeHandler<'a> {
    fn run<Lang : TestLanguage>(&self) -> Result<(), GeneratorError> {
        println!("Generating sources for {}", Lang::name());
        let options = Lang::test_options();
        Lang::generate(self.model, options, &mut file_output_handler::FileOutputHandler {})
    }
}

struct RunTestsHandler {}

impl LanguageIterHandler for RunTestsHandler {
    fn run<Lang : TestLanguage>(&self) -> Result<(), GeneratorError> {
        println!("Executing tests for {}", Lang::name());
        let output = Lang::test_command().output().map_err(|_| GeneratorError::from("Could not run test command."))?;
        
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;

        if !output.status.success() {
            if let Some(code) = output.status.code() {
                Err(GeneratorError::from(format!("Command failed with exit code: {}", code)))?
            }
            else {
                Err(GeneratorError::from(format!("Command failed")))?
            }
        }

        Ok(())
    }
}


#[test]
fn run_compiler() -> Result<(), GeneratorError> {
    for file in TEST_CASE_FILES {
        println!("Generating sources for test case {}", file);
        let input_files = vec!(OsString::from(format!("../tests/verilization/{}.verilization", file)));
        
        let model = verilization_compiler::load_files(input_files)?;

        each_language(&GenerateCodeHandler { model: &model, })?;
    }

    each_language(&RunTestsHandler {})?;


    Ok(())
}
