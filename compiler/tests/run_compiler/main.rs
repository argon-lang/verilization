#![feature(generic_associated_types)]

mod memory_format;
mod test_lang;
mod ts_test_gen;

use verilization_compiler::{lang, file_output_handler};
use lang::GeneratorError;

use test_lang::{TestLanguage, TestGenerator};


use std::ffi::OsString;
use std::io::Write;
use rand::SeedableRng;
use hex_literal::hex;


const NUM_SAMPLES: i32 = 20;


const TEST_CASE_FILES: &[&str] = &[
    "struct_versions",
    "enum_versions",
];


fn run_compiler_for_lang<Lang: TestLanguage>() -> Result<(), GeneratorError> {
    let mut test_gen = Lang::TestGen::start()?;

    for file in TEST_CASE_FILES {
        println!("Generating {} sources for test case {}", Lang::name(), file);
        let input_files = vec!(OsString::from(format!("../tests/verilization/{}.verilization", file)));
        
        let model = verilization_compiler::load_files(input_files)?;

        let options = Lang::test_options();
        Lang::generate(&model, options, &mut file_output_handler::FileOutputHandler {})?;


        let mut rand = rand_chacha::ChaCha20Rng::from_seed(hex!("
            98 6c 6c 7d e2 57 58 26 a4 04 b5 c1 96 0f bf 18 
            ae b4 35 e7 f4 ae ae 80 82 b1 08 94 4b a4 d9 43
        "));
        test_gen.generate_tests(&model, &mut rand)?;
    }

    test_gen.end()?;


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





#[test]
fn run_tests_typescript() -> Result<(), GeneratorError> {
    run_compiler_for_lang::<lang::typescript::TypeScriptLanguage>()
}

#[test]
fn run_tests_java() -> Result<(), GeneratorError> {
    run_compiler_for_lang::<lang::java::JavaLanguage>()
}

#[test]
fn run_tests_scala() -> Result<(), GeneratorError> {
    run_compiler_for_lang::<lang::scala::ScalaLanguage>()
}

