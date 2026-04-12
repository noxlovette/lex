use std::{path::Path, process::Command};

fn run_fixture(path: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lex"))
        .arg(Path::new(path))
        .output()
        .expect("interpreter binary should run")
}

fn assert_success(path: &str, expected_stdout: &str) {
    let output = run_fixture(path);

    assert!(
        output.status.success(),
        "expected success for {path}, stderr was:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(expected_stdout, String::from_utf8_lossy(&output.stdout));
}

fn assert_error(path: &str, expected_stderr_fragment: &str) {
    let output = run_fixture(path);

    assert!(
        !output.status.success(),
        "expected failure for {path}, stdout was:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected_stderr_fragment),
        "stderr for {path} did not contain {expected_stderr_fragment:?}. actual stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn executes_block_scope() {
    assert_success(
        "tests/test_files/interpreter/block/scope.lox",
        "inner\nouter\n",
    );
}

#[test]
fn executes_global_assignment() {
    assert_success(
        "tests/test_files/interpreter/assignment/global.lox",
        "before\nafter\narg\narg\n",
    );
}

#[test]
fn executes_if_else_statements() {
    assert_success(
        "tests/test_files/interpreter/if/else.lox",
        "good\ngood\nblock\n",
    );
}

#[test]
fn executes_while_loops() {
    assert_success(
        "tests/test_files/interpreter/while/syntax.lox",
        "1\n2\n3\n0\n1\n2\n",
    );
}

#[test]
fn executes_for_loop_desugaring() {
    assert_success(
        "tests/test_files/interpreter/for/scope.lox",
        "0\n-1\nafter\n0\n",
    );
}

#[test]
fn executes_recursive_functions() {
    assert_success("tests/test_files/interpreter/function/recursion.lox", "21\n");
}

#[test]
fn executes_logical_operators() {
    assert_success(
        "tests/test_files/interpreter/logical_operator/or.lox",
        "1\n1\ntrue\nfalse\nfalse\nfalse\ntrue\n",
    );
}

#[test]
fn instantiates_and_prints_classes() {
    assert_success("tests/test_files/interpreter/class/empty.lox", "Foo\n");
}

#[test]
fn executes_methods_and_field_access() {
    assert_success("tests/test_files/interpreter/field/method.lox", "got method\narg\n");
}

#[test]
fn stores_fields_on_instances() {
    assert_success(
        "tests/test_files/interpreter/field/on_instance.lox",
        "bar value\nbaz value\nbar value\nbaz value\n",
    );
}

#[test]
fn binds_this_inside_methods() {
    assert_success("tests/test_files/interpreter/this/this_in_method.lox", "baz\n");
}

#[test]
fn dispatches_inherited_and_super_methods() {
    assert_success(
        "tests/test_files/interpreter/super/call_same_method.lox",
        "Derived.foo()\nBase.foo()\n",
    );
}

#[test]
fn inherits_fields_initialized_in_base_class() {
    assert_success(
        "tests/test_files/interpreter/inheritance/set_fields_from_base_class.lox",
        "foo 1\nfoo 2\nbar 1\nbar 2\nbar 1\nbar 2\n",
    );
}

#[test]
fn reports_top_level_return_error() {
    assert_error(
        "tests/test_files/interpreter/return/at_top_level.lox",
        "Can't return from top-level code.",
    );
}

#[test]
fn reports_this_outside_class_error() {
    assert_error(
        "tests/test_files/interpreter/this/this_at_top_level.lox",
        "Can't use 'this' outside of a class.",
    );
}

#[test]
fn reports_super_outside_class_error() {
    assert_error(
        "tests/test_files/interpreter/super/super_at_top_level.lox",
        "Can't use 'super' outside of a class.",
    );
}

#[test]
fn rejects_non_class_superclasses() {
    assert_error(
        "tests/test_files/interpreter/inheritance/inherit_from_number.lox",
        "Superclass must be a class.",
    );
}

#[test]
fn rejects_field_access_on_non_instances() {
    assert_error(
        "tests/test_files/interpreter/field/set_on_num.lox",
        "Only instances have fields.",
    );
}

#[test]
fn rejects_missing_super_methods() {
    assert_error(
        "tests/test_files/interpreter/super/no_superclass_method.lox",
        "Undefined variable \"doesNotExist\"",
    );
}

#[test]
fn rejects_returning_value_from_initializer() {
    assert_error(
        "tests/test_files/interpreter/constructor/return_value.lox",
        "Can't return a value from an initializer.",
    );
}
