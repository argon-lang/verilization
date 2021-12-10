use verilization_compiler::{FileOutputHandler, VError};

use verilization_test_runner::*;

use test_lang::TestLanguage;
use output_comparison::{run_generator, print_file_map};
use std::ffi::OsString;
use std::process::Command;
use std::process::Stdio;
use std::sync::Mutex;
use lazy_static::lazy_static;

use test_generator::test_resources;

lazy_static! {
	static ref BUILD_FLAG: Mutex<bool> = Mutex::default();
}


fn run_command_check_exit(mut command: Command) -> Result<(), VError> {

    println!("Command: {:?}", command);

    let output = command
        .stdout(Stdio::piped())
        .output()
        .expect("Could not run test command.");

    let output_text = String::from_utf8_lossy(&output.stdout);
    print!("{}", output_text);

    let error_text = String::from_utf8_lossy(&output.stderr);
    print!("{}", error_text);

    if !output.status.success() {
        if let Some(code) = output.status.code() {
            panic!("Command failed with exit code: {}", code);
        }
        else {
            panic!("Command failed");
        }
    }

    Ok(())
}


fn build_bindings() -> Result<(), VError> {
    let mut build_flag = BUILD_FLAG.lock().unwrap();

    if !*build_flag {
        let mut build = Command::new("cargo");
        build.arg("build");
        build.current_dir("../../compiler-cli");
        run_command_check_exit(build)?;
        
        let mut build = Command::new("npm");
        build.arg("run");
        build.arg("build");
        build.current_dir("../../bindings/typescript");
        run_command_check_exit(build)?;
        
        *build_flag = true;
    }

    Ok(())
}


fn run_test_case<Lang: TestLanguage>(model_file: &str) -> Result<(), VError> {
    build_bindings()?;

    let model_file = format!("../../{}", model_file);
    let expected_files = run_generator(|path| -> Result<(), VError> {
        let mut input_files = vec!(model_file.clone());
        for rt_file in test_cases::RUNTIME_FILES {
            input_files.push(format!("{}/{}.verilization", test_cases::RUNTIME_DIR, rt_file));
        }

        let model = verilization_compiler::load_files(input_files)?;
        let options = Lang::test_options_dir(OsString::from(path));
        Lang::generate(&model, options, &mut FileOutputHandler {})?;
        Ok(())
    })?;

    let mut commands = Vec::new();
    
    let mut run = Command::new("cargo");
    run.arg("run");
    run.arg("-p");
    run.arg("verilization-compiler-cli");
    run.arg("--");
    commands.push(run);


    let mut run = Command::new("node");
    run.arg("../../bindings/typescript/lib/cli.js");
    commands.push(run);


    for mut cmd in commands {
        let gen_files = run_generator(|path| {
            let options = Lang::test_options_dir(OsString::from(path));
            
            cmd.arg("generate");
            cmd.arg(Lang::name());
            cmd.arg("-i");
            cmd.arg(model_file.clone());
            for rt_file in test_cases::RUNTIME_FILES {
                cmd.arg("-i");
                cmd.arg(format!("{}/{}.verilization", test_cases::RUNTIME_DIR, rt_file));
            }
            Lang::append_options(&mut cmd, &options);
            run_command_check_exit(cmd)
        })?;

        
        if gen_files != expected_files {
            println!("Generated:");
            print_file_map(&gen_files);
            println!("Expected:");
            print_file_map(&expected_files);

            panic!("Generated files did not match the expected files.");
        }
    }
    

    Ok(())
}

#[test_resources("tests/verilization/*.verilization")]
fn run_cli_typescript(file: &str) {
    run_test_case::<verilization_lang_typescript::TypeScriptLanguage>(file).unwrap()
}

#[test_resources("tests/verilization/*.verilization")]
fn run_cli_java(file: &str) {
    run_test_case::<verilization_lang_java::JavaLanguage>(file).unwrap()
}

#[test_resources("tests/verilization/*.verilization")]
fn run_cli_scala(file: &str) {
    run_test_case::<verilization_lang_scala::ScalaLanguage>(file).unwrap()
}
