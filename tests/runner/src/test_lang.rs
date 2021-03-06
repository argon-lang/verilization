use verilization_compiler::{lang, model, VError};
use lang::Language;
use model::{PackageName, QualifiedName, Verilization};

use std::collections::HashMap;
use core::array::IntoIter;
use std::ffi::OsString;
use std::iter::FromIterator;
use std::process::Command;
use rand::Rng;


pub trait TestLanguage: Language {
    type TestGen : TestGenerator;

    fn test_options() -> Self::Options;
    fn test_options_dir(dir: OsString) -> Self::Options;
    fn append_options(command: &mut Command, options: &Self::Options);
    fn test_command() -> Command;

}

pub trait TestGenerator : Sized {
    fn start() -> Result<Self, VError>;
    fn generate_tests<'a, R: Rng>(&'a mut self, model: &'a Verilization, random: &'a mut R) -> Result<(), VError>;
    fn end(self) -> Result<(), VError>;
}

impl TestLanguage for verilization_lang_typescript::TypeScriptLanguage {
    type TestGen = crate::ts_test_gen::TSTestGenerator;
    
    fn test_options() -> Self::Options {
        Self::test_options_dir(OsString::from("../typescript/src/gen/"))
    }
    
    fn test_options_dir(dir: OsString) -> Self::Options {
        verilization_lang_typescript::TSOptions {
            output_dir: dir,
            package_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&["struct", "versions"]), OsString::from("struct/versions") ),
                ( PackageName::from_parts(&["enum", "versions"]), OsString::from("enum/versions") ),
                ( PackageName::from_parts(&["genericsTest"]), OsString::from("genericsTest") ),
                ( PackageName::from_parts(&["finalTest"]), OsString::from("finalTest") ),
                ( PackageName::from_parts(&["interfaceExample"]), OsString::from("interfaceExample") ),
            ])),
            library_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&[]), OsString::from("@verilization/runtime") ),
            ])),
        }
    }
    
    fn append_options(command: &mut Command, options: &Self::Options) {
        command.arg("-o:out_dir");
        command.arg(&options.output_dir);
        for (pkg, dir) in &options.package_mapping {
            command.arg(format!("-o:pkg:{}", pkg));
            command.arg(dir);
        }
        for (pkg, dir) in &options.library_mapping {
            command.arg(format!("-o:lib:{}", pkg));
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

impl TestLanguage for verilization_lang_java::JavaLanguage {
    type TestGen = crate::java_test_gen::JavaTestGenerator;
    
    fn test_options() -> Self::Options {
        Self::test_options_dir(OsString::from("../java/gen/"))
    }
    
    fn test_options_dir(dir: OsString) -> Self::Options {
        verilization_lang_java::JavaOptions {
            output_dir: dir,
            package_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&["struct", "versions"]), PackageName::from_parts(&["struct", "versions"]) ),
                ( PackageName::from_parts(&["enum", "versions"]), PackageName::from_parts(&["enum_", "versions"]) ),
                ( PackageName::from_parts(&["genericsTest"]), PackageName::from_parts(&["genericsTest"]) ),
                ( PackageName::from_parts(&["finalTest"]), PackageName::from_parts(&["finalTest"]) ),
                ( PackageName::from_parts(&["interfaceExample"]), PackageName::from_parts(&["interfaceExample"]) ),
            ])),
            library_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&[]), PackageName::from_parts(&["dev", "argon", "verilization", "runtime"]) ),
            ])),
            extern_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( QualifiedName::from_parts(&[], "nat"), QualifiedName::from_parts(&["java", "math"], "BigInteger") ),
                ( QualifiedName::from_parts(&[], "int"), QualifiedName::from_parts(&["java", "math"], "BigInteger") ),
                ( QualifiedName::from_parts(&[], "u8"), QualifiedName::from_parts(&[], "byte") ),
                ( QualifiedName::from_parts(&[], "i8"), QualifiedName::from_parts(&[], "byte") ),
                ( QualifiedName::from_parts(&[], "u16"), QualifiedName::from_parts(&[], "short") ),
                ( QualifiedName::from_parts(&[], "i16"), QualifiedName::from_parts(&[], "short") ),
                ( QualifiedName::from_parts(&[], "u32"), QualifiedName::from_parts(&[], "int") ),
                ( QualifiedName::from_parts(&[], "i32"), QualifiedName::from_parts(&[], "int") ),
                ( QualifiedName::from_parts(&[], "u64"), QualifiedName::from_parts(&[], "long") ),
                ( QualifiedName::from_parts(&[], "i64"), QualifiedName::from_parts(&[], "long") ),
                ( QualifiedName::from_parts(&[], "string"), QualifiedName::from_parts(&["java", "lang"], "String") ),
                ( QualifiedName::from_parts(&[], "option"), QualifiedName::from_parts(&["java", "util"], "Optional") ),
            ])),
        }
    }
    
    fn append_options(command: &mut Command, options: &Self::Options) {
        command.arg("-o:out_dir");
        command.arg(&options.output_dir);
        for (pkg, java_pkg) in &options.package_mapping {
            command.arg(format!("-o:pkg:{}", pkg));
            command.arg(format!("{}", java_pkg));
        }
        for (pkg, java_pkg) in &options.library_mapping {
            command.arg(format!("-o:lib:{}", pkg));
            command.arg(format!("{}", java_pkg));
        }
        for (extern_name, mapped) in &options.extern_mapping {
            command.arg(format!("-o:extern:{}", extern_name));
            command.arg(format!("{}", mapped));
        }
    }

    fn test_command() -> Command {
        let mut cmd = Command::new("sbt");
        cmd.arg("-J--enable-preview");
        cmd.arg("test");
        cmd.current_dir("../java");
        cmd
    }
}

impl TestLanguage for verilization_lang_scala::ScalaLanguage {
    type TestGen = crate::scala_test_gen::ScalaTestGenerator;

    fn test_options() -> Self::Options {
        Self::test_options_dir(OsString::from("../scala/gen/"))
    }

    fn test_options_dir(dir: OsString) -> Self::Options {
        verilization_lang_scala::ScalaOptions {
            output_dir: dir,
            package_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&["struct", "versions"]), PackageName::from_parts(&["struct", "versions"]) ),
                ( PackageName::from_parts(&["enum", "versions"]), PackageName::from_parts(&["enum_", "versions"]) ),
                ( PackageName::from_parts(&["genericsTest"]), PackageName::from_parts(&["genericsTest"]) ),
                ( PackageName::from_parts(&["finalTest"]), PackageName::from_parts(&["finalTest"]) ),
                ( PackageName::from_parts(&["interfaceExample"]), PackageName::from_parts(&["interfaceExample"]) ),
            ])),
            library_mapping: HashMap::<_, _>::from_iter(IntoIter::new([
                ( PackageName::from_parts(&[]), PackageName::from_parts(&["dev", "argon", "verilization", "scala_runtime"]) ),
            ])),
        }
    }
    
    fn append_options(command: &mut Command, options: &Self::Options) {
        command.arg("-o:out_dir");
        command.arg(&options.output_dir);
        for (pkg, scala_pkg) in &options.package_mapping {
            command.arg(format!("-o:pkg:{}", pkg));
            command.arg(format!("{}", scala_pkg));
        }
        for (pkg, scala_pkg) in &options.library_mapping {
            command.arg(format!("-o:lib:{}", pkg));
            command.arg(format!("{}", scala_pkg));
        }
    }
    
    fn test_command() -> Command {
        let mut cmd = Command::new("sbt");
        cmd.arg("+test");
        cmd.current_dir("../scala");
        cmd
    }
}

