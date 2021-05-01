use verilization_test_runner::*;

use verilization_compiler::{lang, FileOutputHandler, VError};
use lang::GeneratorError;

use test_lang::{TestLanguage, TestGenerator};


use std::ffi::OsString;
use rand::SeedableRng;
use hex_literal::hex;
use std::process::Stdio;


const NUM_SAMPLES: i32 = 20;


fn run_tests_for_lang<Lang: TestLanguage>() -> Result<(), VError> {
    println!("Tests for language {}", Lang::name());
    let mut test_gen = Lang::TestGen::start()?;

    for file in test_cases::TEST_CASE_FILES {
        println!("Generating {} sources for test case {}", Lang::name(), file);
        let mut input_files = vec!(OsString::from(format!("../verilization/{}.verilization", file)));

        for rt_file in test_cases::RUNTIME_FILES {
            input_files.push(OsString::from(format!("{}/{}.verilization", test_cases::RUNTIME_DIR, rt_file)));
        }
        
        let model = verilization_compiler::load_files(input_files)?;

        let options = Lang::test_options();
        Lang::generate(&model, options, &mut FileOutputHandler {})?;


        let mut rand = rand_chacha::ChaCha20Rng::from_seed(hex!("
            98 6c 6c 7d e2 57 58 26 a4 04 b5 c1 96 0f bf 18 
            ae b4 35 e7 f4 ae ae 80 82 b1 08 94 4b a4 d9 43
        "));
        test_gen.generate_tests(&model, &mut rand)?;
    }

    test_gen.end()?;


    println!("Executing tests for {}", Lang::name());
    let output = Lang::test_command()
        .stdout(Stdio::piped())
        .output()
        .map_err(|_| GeneratorError::from("Could not run test command."))?;

    let output_text = String::from_utf8_lossy(&output.stdout);
    print!("{}", output_text);


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
fn run_typescript_tests() -> Result<(), VError> {
    run_tests_for_lang::<lang::typescript::TypeScriptLanguage>()
}

#[test]
fn run_java_tests() -> Result<(), VError> {
    run_tests_for_lang::<lang::java::JavaLanguage>()
}

#[test]
fn run_scala_tests() -> Result<(), VError> {
    run_tests_for_lang::<lang::scala::ScalaLanguage>()
}


