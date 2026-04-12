use std::{path::Path, process::Command};

fn run_fixture(path: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lex"))
        .arg(Path::new(path))
        .output()
        .expect("interpreter binary should run")
}

#[test]
fn executes_methods_and_field_access() {
    let output = run_fixture("tests/test_files/interpreter/field/method.lox");

    assert!(output.status.success());
    assert_eq!("got method\narg\n", String::from_utf8_lossy(&output.stdout));
}

#[test]
fn binds_this_inside_methods() {
    let output = run_fixture("tests/test_files/interpreter/this/this_in_method.lox");

    assert!(output.status.success());
    assert_eq!("baz\n", String::from_utf8_lossy(&output.stdout));
}

#[test]
fn dispatches_inherited_and_super_methods() {
    let output = run_fixture("tests/test_files/interpreter/super/call_same_method.lox");

    assert!(output.status.success());
    assert_eq!(
        "Derived.foo()\nBase.foo()\n",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn rejects_returning_value_from_initializer() {
    let output = run_fixture("tests/test_files/interpreter/constructor/return_value.lox");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Can't return a value from an initializer.")
    );
}
