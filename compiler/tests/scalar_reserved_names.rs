use php_compiler::run_source_with_source_file;

fn fatal_for(source: &str, file: &str) -> String {
    let execution =
        run_source_with_source_file(source, file).expect("source should produce PHP fatal");
    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 255);
    execution.stderr
}

#[test]
fn reserved_scalar_class_name_emits_php_startup_fatal() {
    let stderr = fatal_for(
        r#"<?php
namespace foo;

class int {}
"#,
        "tests/type_declarations/scalar_reserved_class.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use \"int\" as a class name as it is reserved in tests/type_declarations/scalar_reserved_class.php on line 4"
    );
}

#[test]
fn reserved_scalar_use_alias_emits_php_startup_fatal() {
    let stderr = fatal_for(
        r#"<?php
use foobar as float;
"#,
        "tests/type_declarations/scalar_reserved_use.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use foobar as float because 'float' is a special class name in tests/type_declarations/scalar_reserved_use.php on line 2"
    );
}

#[test]
fn reserved_scalar_class_alias_literal_emits_php_startup_fatal() {
    let stderr = fatal_for(
        r#"<?php
class foobar {}
class_alias("foobar", "string");
"#,
        "tests/type_declarations/scalar_reserved_class_alias.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Cannot use \"string\" as a class alias as it is reserved in tests/type_declarations/scalar_reserved_class_alias.php on line 3"
    );
}
