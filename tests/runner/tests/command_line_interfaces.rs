use verilization_compiler::{lang, FileOutputHandler};

use verilization_test_runner::*;

use lang::GeneratorError;
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


fn run_command_check_exit(mut command: Command) -> Result<(), GeneratorError> {
    let output = command
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


fn run_test_case<Lang: TestLanguage>(model_file: &str) -> Result<(), GeneratorError> {
    let expected_files = run_generator(|path| {
        let mut input_files = vec!(String::from(model_file));
        for rt_file in test_cases::RUNTIME_FILES {
            input_files.push(format!("{}/{}.verilization", test_cases::RUNTIME_DIR, rt_file));
        }

        let model = verilization_compiler::load_files(input_files)?;
        let options = Lang::test_options_dir(OsString::from(path));
        Lang::generate(&model, options, &mut FileOutputHandler {})
    })?;

    let mut commands = Vec::new();
    {
        let mut build = Command::new("cargo");
        build.arg("build");
        build.current_dir("../../compiler");

        
        let mut run = Command::new("cargo");
        run.arg("run");
        run.arg("--manifest-path");
        run.arg("../../compiler/Cargo.toml");
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
            run.arg(model_file);
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

            Err("Generated files did not match the expected files.")?;
        }
    }
    

    Ok(())
}

#[test_resources("../verilization/*.verilization")]
fn run_cli_typescript(file: &str) {
    run_test_case::<lang::typescript::TypeScriptLanguage>(file).unwrap()
}

#[test_resources("../verilization/*.verilization")]
fn run_cli_java(file: &str) {
    run_test_case::<lang::java::JavaLanguage>(file).unwrap()
}

#[test_resources("../verilization/*.verilization")]
fn run_cli_scala(file: &str) {
    run_test_case::<lang::scala::ScalaLanguage>(file).unwrap()
}
