use php_compiler::run_source;

#[test]
fn strlen_uses_php_internal_string_argument_coercions() {
    let execution = run_source(
        r#"<?php
ini_set("precision", "12");

set_error_handler(function($_, $message) {
    throw new Exception($message);
});
try {
    strlen(null);
} catch (Exception $e) {
    echo $e->getMessage(), "\n";
}

class Box {
    public $value = "hello";
    public function __toString() {
        return $this->value;
    }
}

var_dump(strlen(new Box));
echo strlen(10.55555555555555555555555555), "|";
echo strlen(10.55555555595555555555555555), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "strlen(): Passing null to parameter #1 ($string) of type string is deprecated\nint(5)\n13|12\n"
    );
    assert_eq!(execution.exit_code, 0);
}
