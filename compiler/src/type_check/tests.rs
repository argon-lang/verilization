
use crate::TypeCheckError;
use crate::type_check::type_check_verilization;
use crate::parser::parse_model;

fn run_type_check_test(file_data: &[&str]) -> Result<(), TypeCheckError> {
    let (_, model) = parse_model(&file_data[0]).unwrap();
    let mut model = model().unwrap();
    for data in &file_data[1..] {
        let (_, other) = parse_model(data).unwrap();
        model.merge(other().unwrap()).unwrap();
    }

    type_check_verilization(&model)
}

#[test]
fn latest_version_mismatch() {
    let file_data = &[
"
version 2;

struct A {
    version 1 {

    }
}
",

"
version 3;
struct B {
    version 1 {
        a: A;
    }
}
"
    ];

    match run_type_check_test(file_data) {
        Err(TypeCheckError::TypeNotInVersion(_, _)) => (),
        _ => assert!(false)
    }
}

#[test]
fn final_older_latest_version() {
    let file_data = &[
"
version 2;

final struct A {
    version 1 {
        
    }
}
",

"
version 3;
struct B {
    version 1 {
        a: A;
    }
}
"
    ];

    match run_type_check_test(file_data) {
        Ok(()) => (),
        _ => assert!(false)
    }
}

#[test]
fn final_non_final_field() {
    let file_data = &[
"
version 2;

struct A {
    version 1 {
        
    }
}
",

"
version 2;
final struct B {
    version 1 {
        a: A;
    }
}
"
    ];

    match run_type_check_test(file_data) {
        Err(TypeCheckError::TypeNotFinal(_)) => (),
        _ => assert!(false)
    }
}

