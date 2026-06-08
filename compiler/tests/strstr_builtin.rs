use php_compiler::run_source;

#[test]
fn strstr_and_stristr_cover_before_needle_empty_and_binary_rows() {
    let execution = run_source(
        r#"<?php
$binary = chr(0).chr(128).chr(129).chr(234).chr(235).chr(254).chr(255);
echo bin2hex(strstr($binary, chr(128))), "\n";
echo bin2hex(strstr($binary, chr(0))), "\n";
var_dump(stristr("tEsT sTrInG", ""));
var_dump(strstr("a@example.com", "@", 1));
var_dump(strstr("eE@fF", "E", ""));
var_dump(stristr("AbcCdEfGh", "c", 1));

class sample {
    public function __toString() {
        return "sample object";
    }
}
var_dump(stristr("Hello World", new sample()));
try {
    var_dump(stristr("Hello World", []));
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "8081eaebfeff\n008081eaebfeff\nstring(11) \"tEsT sTrInG\"\nstring(1) \"a\"\nstring(4) \"E@fF\"\nstring(2) \"Ab\"\nbool(false)\nstristr(): Argument #2 ($needle) must be of type string, array given\n"
    );
    assert_eq!(execution.exit_code, 0);
}
