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
fn crypt_invalid_modular_and_des_salts_use_php_fallback_markers() {
    let execution = run_source(
        r#"<?php
$cases = [
    ["test", '$23$04$1234567890123456789012345'],
    ["test", '$2g$04$1234567890123456789012345'],
    ["test", '$2a$4$1234567891234567891234567'],
    ["test", '$2a$32$1234567891234567891234567'],
    ["foo", '$2a$CCCCCCCCCCCCCCCCCCCCC.E5YPO9kmyuRGyh0XouQYb4YMJKvyOeW'],
    ["foo", '$2y$04$000000000000000000000$'],
    ["test", '$:#'],
    ["test", '$:5zd$01' . "\n"],
    ["foo", '$5$' . chr(0) . "abc"],
    ["foo", '*0'],
];

foreach ($cases as $case) {
    echo crypt($case[0], $case[1]), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "*0\n*0\n*0\n*0\n*0\n*0\n*0\n*0\n*0\n*1\n");
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
        "unsupported call crypt(): only invalid salt fallback markers are implemented in the current subset"
    );

    let error =
        run_source("<?php\ncrypt('secret', '$2y$07$usesomesillystringforsalt$');\n").unwrap_err();
    assert_eq!(
        error.message,
        "unsupported call crypt(): only invalid salt fallback markers are implemented in the current subset"
    );
}
