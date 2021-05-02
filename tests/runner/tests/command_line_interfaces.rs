use verilization_compiler::{lang, FileOutputHandler, VError};

use verilization_test_runner::*;

use test_lang::TestLanguage;
use output_comparison::{run_generator, print_file_map};
use std::ffi::OsString;
use std::process::Command;
use std::process::Stdio;

use test_generator::test_resources;

struct GeneratorCommand {
    build_cmd: Command,
    run_cmd: Command,
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


fn run_test_case<Lang: TestLanguage>(model_file: &str) -> Result<(), VError> {
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
    {
        let mut build = Command::new("cargo");
        build.arg("build");
        build.current_dir("../../compiler-cli");

        
        let mut run = Command::new("cargo");
        run.arg("run");
        run.arg("-p");
        run.arg("verilization-compiler-cli");
        run.arg("--");

        commands.push(GeneratorCommand {
            build_cmd: build,
            run_cmd: run,
        });
    }
    {
        let mut build = Command::new("npm");
        build.arg("run");
        build.arg("build");
        build.current_dir("../../bindings/typescript");

        
        let mut run = Command::new("node");
        run.arg("--experimental-wasm-modules");
        run.arg("../../bindings/typescript/bin/cli.js");

        commands.push(GeneratorCommand {
            build_cmd: build,
            run_cmd: run,
        });
    }


    for cmd in commands {
        run_command_check_exit(cmd.build_cmd)?;

        let mut run = cmd.run_cmd;
        let gen_files = run_generator(|path| {
            let options = Lang::test_options_dir(OsString::from(path));
            
            run.arg("generate");
            run.arg(Lang::name());
            run.arg("-i");
            run.arg(model_file.clone());
            for rt_file in test_cases::RUNTIME_FILES {
                run.arg("-i");
                run.arg(format!("{}/{}.verilization", test_cases::RUNTIME_DIR, rt_file));
            }
            Lang::append_options(&mut run, &options);
            run_command_check_exit(run)
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
    run_test_case::<lang::typescript::TypeScriptLanguage>(file).unwrap()
}

#[test_resources("tests/verilization/*.verilization")]
fn run_cli_java(file: &str) {
    run_test_case::<lang::java::JavaLanguage>(file).unwrap()
}

#[test_resources("tests/verilization/*.verilization")]
fn run_cli_scala(file: &str) {
    run_test_case::<lang::scala::ScalaLanguage>(file).unwrap()
}
