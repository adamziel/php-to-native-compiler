use php_compiler::error::Phase;
use php_compiler::run_source;

#[test]
fn crypt_invalid_underscore_salt_uses_php_fallback() {
    let execution = run_source(
        r#"<?php
echo function_exists("crypt") ? "fn" : "missing";
echo "|";
echo is_callable("crypt") ? "callable" : "not";
echo "|";
$call = "crypt";
$result = $call("a", "_");
echo $result === "*0" || $result === "*1" ? "OK" : "Not OK";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|OK");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn crypt_rejects_algorithm_salts_outside_current_boundary() {
    let error = run_source("<?php\ncrypt('a', 'ab');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call crypt(): only the invalid '_' salt fallback is implemented in the current subset"
    );
}
