use std::path::PathBuf;


pub const TEST_CASE_FILES: &[&str] = &[
    "struct_versions",
    "enum_versions",
];


pub fn test_case_file(name: &str) -> PathBuf {
    PathBuf::from(format!("../verilization/{}.verilization", name))
}

