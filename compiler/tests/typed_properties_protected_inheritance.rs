use php_compiler::run_source_with_source_file;

fn fatal_for(source: &str, file: &str) -> String {
    let execution =
        run_source_with_source_file(source, file).expect("source should produce startup fatal");
    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 255);
    execution.stderr
}

#[test]
fn protected_property_inherited_type_mismatch_emits_php_startup_fatal() {
    let stderr = fatal_for(
        r#"<?php

class A { protected int $x; }
class B extends A { protected $x; }
"#,
        "Zend/tests/type_declarations/typed_properties_protected_inheritance_mismatch.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type of B::$x must be int (as in class A) in Zend/tests/type_declarations/typed_properties_protected_inheritance_mismatch.php on line 4"
    );
}

#[test]
fn inherited_untyped_property_rejects_typed_child_without_shape_specific_names() {
    let stderr = fatal_for(
        r#"<?php
class ParentBox { protected $value; }
class ChildBox extends ParentBox { protected string $value; }
"#,
        "tests/type_declarations/typed_property_untyped_parent.php",
    );

    assert_eq!(
        stderr,
        "Fatal error: Type of ChildBox::$value must be omitted to match the parent definition in class ParentBox in tests/type_declarations/typed_property_untyped_parent.php on line 3"
    );
}

#[test]
fn private_parent_property_type_is_not_inherited_for_startup_diagnostics() {
    let execution = run_source_with_source_file(
        r#"<?php
class PrivateParent { private int $value; }
class PrivateChild extends PrivateParent { protected string $value = "ok"; }
echo "ready";
"#,
        "tests/type_declarations/typed_property_private_parent.php",
    )
    .expect("private parent property should not constrain child property type");

    assert_eq!(execution.stdout, "ready");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
