use std::path::PathBuf;


pub const RUNTIME_DIR: &str = "../../runtime/verilization/";

pub const RUNTIME_FILES: &[&str] = &[
    "integral",
    "string",
    "list",
    "option",
];



pub const TEST_CASE_FILES: &[&str] = &[
    "struct_versions",
    "final",
    "generics",
    "enum_versions",
    "interface_example",
];


pub fn test_case_file(name: &str) -> PathBuf {
    PathBuf::from(format!("../verilization/{}.verilization", name))
}

