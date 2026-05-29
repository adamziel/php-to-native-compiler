use php_compiler::run_source;

#[test]
fn explode_splits_byte_strings_with_limit_and_value_errors() {
    let execution = run_source(
        r#"<?php
$parts = explode("\0", "one\0two\0three\0four", 3);
echo bin2hex($parts[0]), "|", bin2hex($parts[1]), "|", bin2hex($parts[2]), "\n";
var_dump(explode(":", "a:b:c", 0));
var_dump(explode(":", "a:b:c", -1));
try {
    explode("", "payload");
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "6f6e65|74776f|746872656500666f7572\narray(1) {\n  [0]=>\n  string(5) \"a:b:c\"\n}\narray(2) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(1) \"b\"\n}\nexplode(): Argument #1 ($separator) must not be empty\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn join_alias_reuses_implode_scalar_and_object_coercions() {
    let execution = run_source(
        r#"<?php
class Glue {
    public function __toString() {
        return "::";
    }
}

echo function_exists("join") ? "yes" : "no";
echo "|";
echo is_callable("join") ? "callable" : "missing";
echo "|";
echo join(new Glue(), ["left", 7, true, null, "right"]), "\n";
try {
    join([], ["a"]);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
try {
    implode("glue");
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|left::7::1::::right\njoin(): Argument #1 ($separator) must be of type string, array given\nimplode(): If argument #1 ($separator) is of type string, argument #2 ($array) must be of type array, null given\n"
    );
    assert_eq!(execution.exit_code, 0);
}
